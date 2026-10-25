//! The Spell Browser's index and its add-to-character glue. The type names,
//! `SPELL.IDS` script names, IWD2 book names and the add paths are
//! duplicated from the egui keeper's `ui/spell_browser.rs` / `game_view.rs`.

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::spl::Iwd2Placement;
use infinitier_core::resource::ResourceType;
use infinitier_core::resource::cre::{Iwd2Spellbook, SpellType};
use infinitier_core::resource::ids::Ids;
use slint::SharedString;

use super::{Entry, first_strref};
use crate::state::Model;

/// Build the spell index: parse every `.spl`, resolve its name, description
/// strref and `SPELL.IDS` script name. Sorted by type, then level, then
/// name.
pub(super) fn build_index(game_data: &GameData) -> Vec<Entry> {
    let tlk = game_data.dialog_tlk().ok();
    let spell_ids = game_data.import_ids_by_name("SPELL").ok();
    // Collect the resrefs first so the borrow of the resource index ends
    // before each spell is imported.
    let resrefs = {
        let mut resrefs: Vec<String> = game_data
            .get_all_resources_by_type(ResourceType::Spl)
            .map(|r| r.name.clone())
            .collect();
        resrefs.sort();
        resrefs.dedup();
        resrefs
    };

    let mut out: Vec<Entry> = resrefs
        .into_iter()
        .filter_map(|resref| {
            let spl = game_data.import_spl_by_name(&resref).ok()?;
            // Spells keep their real name/description in the *unidentified*
            // strref, so prefer it and fall back to the identified one.
            let name_ref = first_strref(
                spl.header.name_strref(),
                spl.header.name_identified_strref(),
            );
            Some(Entry {
                name: tlk
                    .as_deref()
                    .and_then(|t| t.get(name_ref))
                    .unwrap_or_default(),
                type_name: spell_type_name(spl.header.spell_type()),
                level: Some(spl.header.spell_level()),
                script_name: script_name(spell_ids.as_deref(), &resref),
                icon: spl.header.spellbook_icon().to_owned(),
                description_strref: first_strref(
                    spl.header.description_strref(),
                    spl.header.description_identified_strref(),
                ),
                resref,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.type_name
            .cmp(b.type_name)
            .then(a.level.cmp(&b.level))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Whether the selected party member can take a spell: an embedded CRE is
/// selected (both the AD&D and the IWD2 spell models are supported).
pub(super) fn can_add(model: &Model) -> bool {
    model.selected_cre().is_some()
}

/// The Add button's caption, naming the selected character.
pub(super) fn add_label(model: Option<&Model>) -> String {
    match model.map(Model::character_name) {
        Some(name) if !name.is_empty() => format!("Add to {name}"),
        Some(_) | None => "Add to character".to_owned(),
    }
}

/// An IWD2 placement's entry in the spellbook menu.
pub(super) fn placement_label(placement: &Iwd2Placement) -> SharedString {
    format!("{} — Lvl {}", book_name(placement.book), placement.level).into()
}

/// AD&D: add `resref` to the selected character as a fresh, un-memorised
/// spell, in the spellbook and at the level its SPL header gives.
pub(super) fn add_adnd(model: &mut Model, resref: &str) {
    let Ok(spl) = model.game_data.import_spl_by_name(resref) else {
        return;
    };
    let spell_type = match spl.header.spell_type() {
        1 => SpellType::Wizard,
        2 => SpellType::Priest,
        // Innate / special / psionic / bard / other → the Innate book.
        _ => SpellType::Innate,
    };
    // SPL levels are 1-based; known spells store them 0-based.
    let level = (spl.header.spell_level() as u16).saturating_sub(1);
    if let Some(cre) = model.selected_cre_mut() {
        cre.add_known_spell(spell_type, level, resref);
    }
}

/// IWD2: add the spell to the book, level and list index the user picked,
/// with no memorised copies.
pub(super) fn add_iwd2(model: &mut Model, placement: Iwd2Placement) {
    let Iwd2Placement { book, level, index } = placement;
    if let Some(cre) = model.selected_cre_mut() {
        cre.add_iwd2_spell(book, level as usize, index, 0);
    }
}

/// The "Script Name" for a spell: its `SPELL.IDS` symbol title-cased
/// (`SPPR101` → `1101` → `CLERIC_BLESS` → `"Cleric Bless"`), or empty when
/// the resref isn't a spell-code resref or has no `SPELL.IDS` entry.
fn script_name(spell_ids: Option<&Ids>, resref: &str) -> String {
    spell_ids
        .zip(spell_ids_value(resref))
        .and_then(|(ids, value)| ids.of_value(value))
        .map(title_case)
        .unwrap_or_default()
}

/// Map a spell resref to its `SPELL.IDS` numeric value: the two-letter
/// school code after `SP` selects a thousands digit (PR→1 priest, WI→2
/// wizard, IN→3 innate, CL→4 special) and the trailing number is added
/// (e.g. `SPWI923` → 2923). `None` for non-spell resrefs.
fn spell_ids_value(resref: &str) -> Option<i32> {
    let body = resref
        .get(..2)
        .filter(|p| p.eq_ignore_ascii_case("SP"))
        .map(|_| &resref[2..])?;
    let (code, number) = body.split_at_checked(2)?;
    let thousands = match code.to_ascii_uppercase().as_str() {
        "PR" => 1,
        "WI" => 2,
        "IN" => 3,
        "CL" => 4,
        _ => return None,
    };
    let n: i32 = number.parse().ok()?;
    Some(thousands * 1000 + n)
}

/// Title-case a `SPELL.IDS` symbol: `CLERIC_ARMOR_OF_FAITH` →
/// `Cleric Armor Of Faith`.
fn title_case(symbol: &str) -> String {
    symbol
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Display name of an IWD2 spellbook, for the spellbook menu.
fn book_name(book: Iwd2Spellbook) -> &'static str {
    match book {
        Iwd2Spellbook::Bard => "Bard",
        Iwd2Spellbook::Cleric => "Cleric",
        Iwd2Spellbook::Druid => "Druid",
        Iwd2Spellbook::Paladin => "Paladin",
        Iwd2Spellbook::Ranger => "Ranger",
        Iwd2Spellbook::Sorcerer => "Sorcerer",
        Iwd2Spellbook::Wizard => "Wizard",
        Iwd2Spellbook::Domain => "Domain",
        Iwd2Spellbook::Innate => "Innate",
        Iwd2Spellbook::Song => "Song",
        Iwd2Spellbook::ShapeChange => "Shape",
    }
}

/// Map an SPL spell-type id to a display "Type". Mirrors EEKeeper, which
/// shows priest spells as "Cleric". Unknown values fall back to "Other".
fn spell_type_name(spell_type: u16) -> &'static str {
    match spell_type {
        1 => "Wizard",
        2 => "Cleric",
        3 => "Psionic",
        4 => "Innate",
        5 => "Bard",
        0 => "Special",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_ids_value_maps_school_codes() {
        assert_eq!(spell_ids_value("SPPR101"), Some(1101));
        assert_eq!(spell_ids_value("spwi923"), Some(2923));
        assert_eq!(spell_ids_value("SPIN101"), Some(3101));
        assert_eq!(spell_ids_value("SPCL311"), Some(4311));
        assert_eq!(spell_ids_value("SPXX101"), None);
        assert_eq!(spell_ids_value("RING01"), None);
        assert_eq!(spell_ids_value("SP"), None);
    }

    #[test]
    fn title_case_symbols() {
        assert_eq!(title_case("CLERIC_ARMOR_OF_FAITH"), "Cleric Armor Of Faith");
        assert_eq!(title_case("WIZARD__X"), "Wizard  X");
    }
}
