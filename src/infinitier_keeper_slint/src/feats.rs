//! Feats tab for the Slint keeper (editable, IWD2 only).
//!
//! Ports the egui keeper's `ui/tabs/feats.rs`. One row per IWD2 feat
//! (`FEATS.IDS`, ~75): an editable **Known** checkbox, plus — for the 26
//! stackable feats and only while known — an editable **Level** row. Known is
//! a bit (`Cre::iwd2_feat_known`); Level is a per-feat count byte
//! (`Cre::iwd2_feat_count`) addressed by slot. Feat names resolve via
//! `FEATS.IDS` → `feats.2da` NAMEREF → `dialog.tlk`, shown alphabetically.
//! `COUNT_SLOTS` + the name resolution are duplicated from the egui source.
//!
//! Field-id encoding: `feat_index` for the Known toggle; `LEVEL_ID_OFFSET +
//! slot` for the Level input.

use infinitier_core::game::GameData;
use infinitier_core::resource::tlk::Tlk;
use infinitier_core::resource::two_da::TwoDA;

use crate::FieldCard;
use crate::state::Model;
use crate::ui_model::{card, editable_check_row, editable_row};

/// Level-input ids start here, clear of the feat-index space.
const LEVEL_ID_OFFSET: i32 = 1000;

/// `FEATS.IDS` symbol → per-feat count-byte slot, for the 26 stackable feats.
const COUNT_SLOTS: &[(&str, u8)] = &[
    ("MARTIAL_BOW", 0),
    ("SIMPLE_CROSSBOW", 1),
    ("SIMPLE_MISSILE", 2),
    ("MARTIAL_AXE", 3),
    ("SIMPLE_MACE", 4),
    ("MARTIAL_FLAIL", 5),
    ("MARTIAL_POLEARM", 6),
    ("MARTIAL_HAMMER", 7),
    ("SIMPLE_QUARTERSTAFF", 8),
    ("MARTIAL_GREATSWORD", 9),
    ("MARTIAL_LARGESWORD", 10),
    ("SIMPLE_SMALLBLADE", 11),
    ("TOUGHNESS", 12),
    ("ARMORED_ARCANA", 13),
    ("CLEAVE", 14),
    ("ARMOR_PROF", 15),
    ("SPELL_FOCUS_ENCHANTMENT", 16),
    ("SPELL_FOCUS_EVOCATION", 17),
    ("SPELL_FOCUS_NECROMANCY", 18),
    ("SPELL_FOCUS_TRANSMUTE", 19),
    ("SPELL_PENETRATION", 20),
    ("EXTRA_RAGE", 21),
    ("EXTRA_SHAPESHIFTING", 22),
    ("EXTRA_SMITING", 23),
    ("EXTRA_TURNING", 24),
    ("EXOTIC_BASTARD", 25),
];

fn count_slot_for(symbol: &str) -> Option<u8> {
    COUNT_SLOTS
        .iter()
        .find(|(s, _)| *s == symbol)
        .map(|(_, slot)| *slot)
}

struct FeatRow {
    feat_index: u8,
    name: String,
    count_slot: Option<u8>,
}

fn build_feat_list(game_data: &GameData) -> Vec<FeatRow> {
    let Ok(ids) = game_data.import_ids_by_name("feats") else {
        return Vec::new();
    };
    let two_da = game_data.import_2da_by_name("feats").ok();
    let nameref_col = two_da
        .as_ref()
        .and_then(|t| t.headers.iter().position(|h| h == "NAMEREF"));
    let tlk = game_data.dialog_tlk().ok();

    let mut rows: Vec<FeatRow> = ids
        .entries
        .iter()
        .map(|entry| {
            let symbol = entry.name.as_str();
            let feat_index = entry.value as u8;
            let name = resolve_name(symbol, two_da.as_deref(), nameref_col, tlk.as_deref())
                .unwrap_or_else(|| prettify(symbol));
            FeatRow {
                feat_index,
                name,
                count_slot: count_slot_for(symbol),
            }
        })
        .collect();
    rows.sort_by_key(|r| r.name.to_lowercase());
    rows
}

fn resolve_name(
    symbol: &str,
    two_da: Option<&TwoDA>,
    nameref_col: Option<usize>,
    tlk: Option<&Tlk>,
) -> Option<String> {
    let strref: u32 = two_da?.rows.get(symbol)?.get(nameref_col?)?.parse().ok()?;
    tlk?.get(strref).filter(|s| !s.is_empty())
}

fn prettify(symbol: &str) -> String {
    symbol
        .split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    first.to_ascii_uppercase().to_string() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build the Feats-tab cards for the selected IWD2 creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(cre) = model.selected_cre() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    // Not an IWD2 (V2.2) creature → no feats block.
    if cre.iwd2_feat_known(0).is_none() {
        return [Vec::new(), Vec::new(), Vec::new()];
    }
    let feats = build_feat_list(&model.game_data);
    let mut rows = Vec::new();
    for f in &feats {
        let known = cre.iwd2_feat_known(f.feat_index).unwrap_or(false);
        rows.push(editable_check_row(i32::from(f.feat_index), &f.name, known));
        // Level only exists for stackable feats, and only once known.
        if known && let Some(slot) = f.count_slot {
            let level = cre.iwd2_feat_count(slot).unwrap_or(0);
            rows.push(editable_row(
                LEVEL_ID_OFFSET + i32::from(slot),
                &format!("{} (level)", f.name),
                &level.to_string(),
            ));
        }
    }
    [vec![card("Feats", 0, rows)], Vec::new(), Vec::new()]
}

/// Commit an edited feat cell: a Level input (id ≥ offset) or a Known toggle.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    if id >= LEVEL_ID_OFFSET {
        let slot = (id - LEVEL_ID_OFFSET) as u8;
        let level = value.trim().parse::<u32>().unwrap_or(0).min(u8::MAX as u32) as u8;
        if let Some(cre) = model.selected_cre_mut() {
            cre.set_iwd2_feat_count(slot, level);
        }
    } else if id >= 0 {
        let feat_index = id as u8;
        let known = value == "1";
        if let Some(cre) = model.selected_cre_mut() {
            cre.set_iwd2_feat_known(feat_index, known);
        }
    }
}
