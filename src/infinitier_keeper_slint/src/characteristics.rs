//! Characteristics tab for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/characteristics/`. The identity fields
//! (gender/race/alignment/class/kit/enemy-ally/state) are stored as raw IDS
//! values and resolved here against the game's `*.IDS` / `*.2DA` files; the
//! kill stats come from the GAM party slot; the Miscellaneous checkboxes come
//! from `CreatureFlags::MISC`. The data-extraction logic (`CharData` +
//! `resolve` + helpers) is duplicated verbatim from the egui `data.rs` per
//! the port decision. The egui view resolves `dialog.tlk` strrefs at render
//! time (memoised per frame); here they're resolved during `build`, which
//! runs once per refresh (tab switch / selection / commit), so no cache is
//! needed. The whole tab is display-only — no `commit`.

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::cre::ImportedCre;
use infinitier_core::imported_resource::gam::NpcCre;
use infinitier_core::resource::Game;
use infinitier_core::resource::cre::{Cre, CreHeader, CreHeaderV22, CreatureFlags};
use infinitier_core::resource::gam::NpcCharStats;
use infinitier_core::resource::two_da::TwoDA;

use crate::FieldCard;
use crate::state::Model;
use crate::ui_model::{card, check_row, readonly_row};

// ── Resolved, display-ready data (duplicated from the egui data.rs) ───

#[derive(Debug, Default, Clone)]
struct CharData {
    gender: String,
    race: String,
    sub_race: String,
    alignment: String,
    class: String,
    original_class: String,
    specialization: Specialization,
    racial_enemy: TlkLabel,
    enemy_ally: String,
    state: String,
    movement: i64,
    fallen: bool,
    dual_class: bool,
    kill: KillStats,
    flags: CreatureFlags,
}

#[derive(Debug, Clone)]
enum Specialization {
    Kit(TlkLabel),
    PstReligion { deity: String, mage_type: String },
}

impl Default for Specialization {
    fn default() -> Self {
        Specialization::Kit(TlkLabel::default())
    }
}

/// A display label whose text comes from a `dialog.tlk` strref, with a
/// `fallback` used when the strref is absent or doesn't resolve.
#[derive(Debug, Default, Clone)]
struct TlkLabel {
    strref: u32,
    fallback: String,
}

#[derive(Debug, Default, Clone)]
struct KillStats {
    strongest_name_strref: u32,
    strongest_xp: u32,
    chapter_kills: u32,
    chapter_kills_xp: u32,
    game_kills: u32,
    game_kills_xp: u32,
}

struct RawChar {
    gender: u8,
    race: u8,
    subrace: u8,
    alignment: u8,
    class: u8,
    ea: u8,
    kit: u32,
    state_flags: u32,
}

impl CharData {
    fn resolve(
        imported: &ImportedCre,
        char_stats: &NpcCharStats,
        game_data: &GameData,
    ) -> CharData {
        let cre: &Cre = imported.cre();
        let kill = KillStats {
            strongest_name_strref: char_stats.most_powerful_vanquished_name,
            strongest_xp: char_stats.most_powerful_vanquished_xp,
            chapter_kills: char_stats.kills_number_chapter,
            chapter_kills_xp: char_stats.kills_xp_chapter,
            game_kills: char_stats.kills_number_game,
            game_kills_xp: char_stats.kills_xp_game,
        };
        let flags = cre.creature_flags();
        let original_class = cre.dual_class_original_class();
        let dual_class = original_class.is_some();
        let fallen = cre.is_fallen();
        let original_class_label = original_class
            .map(|c| pretty(c.symbol(), " "))
            .unwrap_or_default();

        let Some(raw) = raw_char(cre) else {
            return CharData {
                kill,
                flags,
                dual_class,
                fallen,
                original_class: original_class_label,
                ..Default::default()
            };
        };

        let specialization = if game_data.game() == Game::Pstee {
            Specialization::PstReligion {
                deity: resolve_deity(game_data, (raw.kit & 0xFFFF) as i32),
                mage_type: resolve_mage_type(game_data, (raw.kit >> 16) as i32),
            }
        } else {
            Specialization::Kit(resolve_kit(game_data, imported))
        };

        let is_iwd2 = matches!(cre.header, CreHeader::V22(_));
        let class = match &cre.header {
            CreHeader::V22(h) => {
                let by_levels = iwd2_class_label(h);
                if by_levels.is_empty() {
                    ids_pretty(game_data, "class", raw.class as i32, " / ")
                } else {
                    by_levels
                }
            }
            _ => ids_pretty(game_data, "class", raw.class as i32, " / "),
        };
        let alignment = if is_iwd2 {
            iwd2_alignment(game_data, raw.alignment)
        } else {
            ids_pretty(game_data, "alignmen", raw.alignment as i32, " ")
        };
        let sub_race = if is_iwd2 {
            iwd2_sub_race(game_data, raw.race, raw.subrace)
        } else {
            String::new()
        };

        CharData {
            gender: ids_pretty(game_data, "gender", raw.gender as i32, " "),
            race: ids_pretty(game_data, "race", raw.race as i32, " "),
            sub_race,
            alignment,
            class,
            original_class: original_class_label,
            specialization,
            racial_enemy: resolve_racial_enemy(game_data, imported),
            enemy_ally: ids_pretty(game_data, "ea", raw.ea as i32, " "),
            state: resolve_state(game_data, raw.state_flags),
            movement: 0,
            fallen,
            dual_class,
            kill,
            flags,
        }
    }
}

fn raw_char(cre: &Cre) -> Option<RawChar> {
    match &cre.header {
        CreHeader::V10(h) => Some(RawChar {
            gender: h.gender_gender_ids_dictates_the_casting,
            race: h.race_race_ids,
            subrace: 0,
            alignment: h.alignment_alignmen_ids,
            class: h.class_class_ids,
            ea: h.enemy_ally_ea_ids,
            kit: h.kit_information_none_0x00000000_kit_barbarian,
            state_flags: h.permanent_status_flags_state_ids,
        }),
        CreHeader::V12(h) => Some(RawChar {
            gender: h.gender_gender_ids,
            race: h.race_race_ids,
            subrace: 0,
            alignment: h.alignment_alignmen_ids,
            class: h.class_class_ids,
            ea: h.enemy_ally_ea_ids,
            kit: h.kit_information_none_0x00000000_abjurer_0x00400000,
            state_flags: h.permanent_status_flags_state_ids,
        }),
        CreHeader::V90(h) => Some(RawChar {
            gender: h.gender_gender_ids,
            race: h.race_race_ids,
            subrace: 0,
            alignment: h.alignment_alignmen_ids,
            class: h.class_class_ids,
            ea: h.enemy_ally_ea_ids,
            kit: h.kit_information_none_abjurer_0x00400000_conjurer,
            state_flags: h.permanent_status_flags_state_ids,
        }),
        CreHeader::V22(h) => Some(RawChar {
            gender: h.sex_gender_ids,
            race: h.race_race_ids,
            subrace: h.subrace_subrace_ids,
            alignment: h.alignment_alignmen_ids,
            class: h.class_class_ids_not_updated_when,
            ea: h.enemy_ally_ea_ids,
            kit: h.kit_bitfield,
            state_flags: h.permanent_status_flags_state_ids,
        }),
    }
}

fn iwd2_class_label(h: &CreHeaderV22) -> String {
    class_from_levels(&[
        (h.barbarian_levels, "Barbarian"),
        (h.bard_levels, "Bard"),
        (h.cleric_levels, "Cleric"),
        (h.druid_levels, "Druid"),
        (h.fighter_levels, "Fighter"),
        (h.monk, "Monk"),
        (h.paladin_levels, "Paladin"),
        (h.ranger_levels, "Ranger"),
        (h.rogue_levels, "Rogue"),
        (h.sorcerer_levels, "Sorcerer"),
        (h.wizard_levels, "Wizard"),
    ])
}

fn class_from_levels(levels: &[(u8, &str)]) -> String {
    levels
        .iter()
        .filter(|(level, _)| *level > 0)
        .map(|(_, name)| *name)
        .collect::<Vec<_>>()
        .join(" / ")
}

fn ids_pretty(game_data: &GameData, ids_file: &str, value: i32, sep: &str) -> String {
    ids_symbol(game_data, ids_file, value)
        .map(|s| pretty(&s, sep))
        .unwrap_or_default()
}

fn ids_symbol(game_data: &GameData, ids_file: &str, value: i32) -> Option<String> {
    let ids = game_data.import_ids_by_name(ids_file).ok()?;
    ids.entries
        .iter()
        .find(|e| e.value == value)
        .map(|e| e.name.clone())
}

fn iwd2_alignment(game_data: &GameData, value: u8) -> String {
    let Ok(aligns) = game_data.import_2da_by_name("aligns") else {
        return String::new();
    };
    alignment_from_2da(&aligns, value)
}

fn alignment_from_2da(aligns: &TwoDA, value: u8) -> String {
    let Some(value_col) = aligns
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case("VALUE"))
    else {
        return String::new();
    };
    aligns
        .rows
        .iter()
        .find(|(_, cells)| {
            cells.get(value_col).and_then(|c| parse_2da_int(c)) == Some(i64::from(value))
        })
        .map(|(name, _)| pretty(name, " "))
        .unwrap_or_default()
}

fn iwd2_sub_race(game_data: &GameData, race: u8, subrace: u8) -> String {
    ids_symbol(game_data, "subrace", subrace_packed(race, subrace))
        .map(|s| pretty(&s, " "))
        .unwrap_or_default()
}

fn subrace_packed(race: u8, subrace: u8) -> i32 {
    if subrace == 0 {
        0
    } else {
        (i32::from(race) << 16) | i32::from(subrace)
    }
}

fn parse_2da_int(cell: &str) -> Option<i64> {
    let cell = cell.trim();
    match cell.strip_prefix("0x").or_else(|| cell.strip_prefix("0X")) {
        Some(hex) => i64::from_str_radix(hex, 16).ok(),
        None => cell.parse().ok(),
    }
}

fn resolve_kit(game_data: &GameData, imported: &ImportedCre) -> TlkLabel {
    let Some(kit) = imported.cre().kit() else {
        return TlkLabel::default();
    };
    let swapped = ((kit & 0xFFFF) << 16) | (kit >> 16);
    if swapped == 0 {
        return TlkLabel::default();
    }
    if ids_symbol(game_data, "kit", swapped as i32).as_deref() == Some("TRUECLASS") {
        return TlkLabel {
            strref: 0,
            fallback: "Base Class".to_string(),
        };
    }
    let fallback = ids_symbol(game_data, "kit", swapped as i32)
        .map(|s| pretty(&s, " "))
        .unwrap_or_default();
    let strref = imported.kit_strref(game_data).unwrap_or(0);
    TlkLabel { strref, fallback }
}

fn resolve_deity(game_data: &GameData, value: i32) -> String {
    if value == 0 {
        return String::new();
    }
    ids_symbol(game_data, "deity", value)
        .or_else(|| ids_symbol(game_data, "diety", value))
        .map(|s| pretty(&s, " "))
        .unwrap_or_default()
}

fn resolve_mage_type(game_data: &GameData, value: i32) -> String {
    if value == 0 {
        return String::new();
    }
    if let Some(s) = ids_symbol(game_data, "magespec", value) {
        return pretty(&s, " ");
    }
    match value {
        0x0040 => "Abjurer",
        0x0080 => "Conjurer",
        0x0100 => "Diviner",
        0x0200 => "Enchanter",
        0x0400 => "Illusionist",
        0x0800 => "Invoker",
        0x1000 => "Necromancer",
        0x2000 => "Transmuter",
        0x4000 => "Generalist",
        _ => "",
    }
    .to_string()
}

fn resolve_racial_enemy(game_data: &GameData, imported: &ImportedCre) -> TlkLabel {
    let Some(value) = imported.cre().racial_enemy() else {
        return TlkLabel::default();
    };
    if value == 0 {
        return TlkLabel::default();
    }
    let fallback = match ids_symbol(game_data, "race", value as i32) {
        Some(s) if s == "NO_RACE" || s == "ANYTHING" => String::new(),
        Some(s) => pretty(&s, " "),
        None => String::new(),
    };
    let strref = imported.racial_enemy_strref(game_data).unwrap_or(0);
    TlkLabel { strref, fallback }
}

fn resolve_state(game_data: &GameData, flags: u32) -> String {
    match ids_symbol(game_data, "state", flags as i32) {
        Some(s) => pretty(&s, " "),
        None => format!("0x{flags:08X}"),
    }
}

fn pretty(symbol: &str, sep: &str) -> String {
    symbol
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

// ── TLK resolution (done at build time, not per frame) ────────────────

fn resolve_strref(game_data: &GameData, strref: u32) -> String {
    if strref == 0 || strref == 0xFFFF_FFFF {
        return String::new();
    }
    game_data
        .dialog_tlk()
        .ok()
        .and_then(|tlk| tlk.get(strref))
        .unwrap_or_default()
}

fn tlk_label(game_data: &GameData, label: &TlkLabel) -> String {
    let resolved = resolve_strref(game_data, label.strref);
    if resolved.is_empty() {
        label.fallback.clone()
    } else {
        resolved
    }
}

// ── Build ────────────────────────────────────────────────────────────

/// Build the Characteristics-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(npc) = model.selected_npc() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let Some(NpcCre::Cre(imported)) = npc.cre.as_ref() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let gd = &model.game_data;
    let data = CharData::resolve(imported, &npc.char_stats, gd);

    // Identity column.
    let mut ident = vec![
        readonly_row("Gender", &data.gender),
        readonly_row("Race", &data.race),
    ];
    if !data.sub_race.is_empty() {
        ident.push(readonly_row("Sub Race", &data.sub_race));
    }
    ident.push(readonly_row("Alignment", &data.alignment));
    ident.push(readonly_row("Class", &data.class));
    ident.push(check_row("Fallen", data.fallen));
    ident.push(readonly_row("Original Class", &data.original_class));
    ident.push(check_row("Dual Class", data.dual_class));
    match &data.specialization {
        Specialization::Kit(kit) => ident.push(readonly_row("Kit", &tlk_label(gd, kit))),
        Specialization::PstReligion { deity, mage_type } => {
            ident.push(readonly_row("Deity", deity));
            ident.push(readonly_row("Mage Type", mage_type));
        }
    }
    ident.push(readonly_row("Racial", &tlk_label(gd, &data.racial_enemy)));
    ident.push(readonly_row("Enemy/Ally", &data.enemy_ally));
    ident.push(readonly_row("State", &data.state));
    // Movement is always 0 ("normal speed"); the egui keeper notes this
    // inline, but the value column is too narrow for the note here.
    ident.push(readonly_row("Movement", &data.movement.to_string()));

    // Kill stats.
    let k = &data.kill;
    let kill_rows = vec![
        readonly_row(
            "Strongest Kill Name",
            &resolve_strref(gd, k.strongest_name_strref),
        ),
        readonly_row("Strongest Kill XP", &k.strongest_xp.to_string()),
        readonly_row("Chapter Kills", &k.chapter_kills.to_string()),
        readonly_row("Chapter Kills XP", &k.chapter_kills_xp.to_string()),
        readonly_row("Game Kills", &k.game_kills.to_string()),
        readonly_row("Game Kills XP", &k.game_kills_xp.to_string()),
    ];

    // Miscellaneous creature flags.
    let misc_rows = CreatureFlags::MISC
        .iter()
        .map(|(label, bit)| check_row(label, data.flags.contains(*bit)))
        .collect();

    [
        vec![card("Identity", 0, ident)],
        vec![
            card("Kill Stats", 1, kill_rows),
            card("Miscellaneous", 1, misc_rows),
        ],
        Vec::new(),
    ]
}
