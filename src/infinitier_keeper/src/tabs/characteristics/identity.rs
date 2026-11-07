//! The Characteristics tab's Identity card, whose fields the player edits.
//!
//! Each field holds a number the game looks up in one of its tables: an IDS
//! file (gender, race, class…), `ALIGNS.2DA` for IWD2's alignment,
//! `KITLIST.2DA` for a kit. It's picked here by name from a dropdown of the
//! table's entries: each value once, in the table's order. The creature's
//! own value comes first, by number, when the table lacks it, so it's never
//! lost. A commit names the option by its index, resolved against the same
//! choices, rebuilt: the creature hasn't changed since they were shown.
//!
//! Fallen is a creature flag, a checkbox. Dual Class follows Original Class. IWD2's class
//! follows its levels (the Levels tab), and it has no kit, racial enemy or
//! dual-classing; those rows stay read-only there.

use std::collections::HashSet;

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::cre::{kits, racial_enemy_strref};
use infinitier_core::resource::Game;
use infinitier_core::resource::cre::{Cre, CreHeader, CreHeaderV22, CreatureFlags, OriginalClass};

use super::resolve_strref;
use crate::FieldRow;
use crate::components::field_card::{check_row, editable_check_row, readonly_row, select_row};
use crate::state::Model;

/// An Identity field the player edits. Its place in [`Field::ALL`] is its
/// row's field id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Gender,
    Race,
    SubRace,
    Alignment,
    Class,
    Fallen,
    OriginalClass,
    Kit,
    Deity,
    MageType,
    RacialEnemy,
    EnemyAlly,
    State,
}

impl Field {
    const ALL: [Self; 13] = [
        Self::Gender,
        Self::Race,
        Self::SubRace,
        Self::Alignment,
        Self::Class,
        Self::Fallen,
        Self::OriginalClass,
        Self::Kit,
        Self::Deity,
        Self::MageType,
        Self::RacialEnemy,
        Self::EnemyAlly,
        Self::State,
    ];

    fn id(self) -> i32 {
        Self::ALL
            .iter()
            .position(|&field| field == self)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    fn from_id(id: i32) -> Option<Self> {
        usize::try_from(id)
            .ok()
            .and_then(|index| Self::ALL.get(index))
            .copied()
    }

    fn label(self) -> &'static str {
        match self {
            Self::Gender => "Gender",
            Self::Race => "Race",
            Self::SubRace => "Sub Race",
            Self::Alignment => "Alignment",
            Self::Class => "Class",
            Self::Fallen => "Fallen",
            Self::OriginalClass => "Original Class",
            Self::Kit => "Kit",
            Self::Deity => "Deity",
            Self::MageType => "Mage Type",
            Self::RacialEnemy => "Racial",
            Self::EnemyAlly => "Enemy/Ally",
            Self::State => "State",
        }
    }
}

/// A dropdown's choices: each value a field can take, and its name.
type Choices = Vec<(u32, String)>;

/// The kit dword of a character without a kit, its class's own:
/// `KITLIST.2DA`'s first row.
const TRUE_CLASS: u32 = 0x4000_0000;

// ── The card ──────────────────────────────────────────────────────────

/// The Identity card's rows.
pub fn rows(game_data: &GameData, cre: &Cre) -> Vec<FieldRow> {
    // A field's dropdown, or its value read-only where it can't be edited.
    let row = |field: Field, fallback: &str| {
        dropdown(game_data, cre, field).unwrap_or_else(|| readonly_row(field.label(), fallback))
    };
    let mut rows = vec![row(Field::Gender, ""), row(Field::Race, "")];
    // IWD2 only.
    rows.extend(dropdown(game_data, cre, Field::SubRace));
    rows.push(row(Field::Alignment, ""));
    rows.push(row(Field::Class, &iwd2_class(game_data, cre)));
    rows.push(fallen_row(game_data, cre));
    rows.push(row(Field::OriginalClass, ""));
    rows.push(check_row("Dual Class", cre.is_dual_classed()));
    if game_data.game() == Game::Pstee {
        rows.push(row(Field::Deity, ""));
        rows.push(row(Field::MageType, ""));
    } else {
        rows.push(row(Field::Kit, ""));
    }
    rows.push(row(Field::RacialEnemy, ""));
    rows.push(row(Field::EnemyAlly, ""));
    rows.push(row(Field::State, ""));
    rows
}

/// `field`'s dropdown row, `None` where the creature has no such field to
/// edit.
fn dropdown(game_data: &GameData, cre: &Cre, field: Field) -> Option<FieldRow> {
    let (choices, index) = choices(game_data, cre, field)?;
    let names = choices.into_iter().map(|(_, name)| name.into()).collect();
    Some(select_row(
        field.id(),
        field.label(),
        names,
        i32::try_from(index).unwrap_or(-1),
    ))
}

/// Fallen: a checkbox, which can make a paladin or a ranger fallen, and
/// clear it on anyone.
fn fallen_row(game_data: &GameData, cre: &Cre) -> FieldRow {
    let fallen = cre.is_fallen();
    if fallen || !fallen_flags(game_data, cre).is_empty() {
        editable_check_row(Field::Fallen.id(), Field::Fallen.label(), fallen)
    } else {
        check_row(Field::Fallen.label(), fallen)
    }
}

/// IWD2's class, by its levels (those it has any of); else by its class
/// field, which IWD2 doesn't keep up to date.
fn iwd2_class(game_data: &GameData, cre: &Cre) -> String {
    let CreHeader::V22(h) = &cre.header else {
        return String::new();
    };
    let by_levels = iwd2_classes(h);
    if by_levels.is_empty() {
        ids_symbol(game_data, "class", cre.class().into())
            .map(|symbol| pretty(&symbol, " / "))
            .unwrap_or_default()
    } else {
        by_levels
    }
}

fn iwd2_classes(h: &CreHeaderV22) -> String {
    [
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
    ]
    .iter()
    .filter(|(levels, _)| *levels > 0)
    .map(|(_, name)| *name)
    .collect::<Vec<_>>()
    .join(" / ")
}

// ── Editing ───────────────────────────────────────────────────────────

/// An edit to make: a field's new value, or (Fallen) the creature flags.
enum Edit {
    Value(u32),
    Flags(CreatureFlags),
}

/// Apply an edit of the Identity row `id`: `value` is the index picked in
/// its dropdown, or Fallen's `"1"` / `"0"`.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    let Some(field) = Field::from_id(id) else {
        return;
    };
    let Some(cre) = model.selected_cre() else {
        return;
    };
    let game_data = &model.game_data;
    let edit = if field == Field::Fallen {
        let mut flags = cre.creature_flags();
        flags.remove(CreatureFlags::FallenPaladin | CreatureFlags::FallenRanger);
        if value == "1" {
            flags.insert(fallen_flags(game_data, cre));
        }
        Edit::Flags(flags)
    } else {
        let Some((choices, _)) = choices(game_data, cre, field) else {
            return;
        };
        let Some(&(picked, _)) = value
            .parse::<usize>()
            .ok()
            .and_then(|index| choices.get(index))
        else {
            return;
        };
        Edit::Value(picked)
    };
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    match edit {
        Edit::Flags(flags) => cre.set_creature_flags(flags),
        Edit::Value(value) => set_value(cre, field, value),
    }
}

/// The flags that make the creature fallen, by its class: a paladin's, a
/// ranger's, both, or none.
fn fallen_flags(game_data: &GameData, cre: &Cre) -> CreatureFlags {
    let (paladin, ranger) = if matches!(cre.header, CreHeader::V22(_)) {
        (
            cre.paladin_level().is_some_and(|levels| levels > 0),
            cre.ranger_level().is_some_and(|levels| levels > 0),
        )
    } else {
        let class = ids_symbol(game_data, "class", cre.class().into()).unwrap_or_default();
        (class.contains("PALADIN"), class.contains("RANGER"))
    };
    let mut flags = CreatureFlags::empty();
    flags.set(CreatureFlags::FallenPaladin, paladin);
    flags.set(CreatureFlags::FallenRanger, ranger);
    flags
}

/// The number `field` holds; `None` where the creature has no such field to
/// edit: IWD2's class, original class, kit and racial enemy; the kit in
/// PSTEE, whose kit dword is its deity and mage type, and those elsewhere.
fn value(game: Game, cre: &Cre, field: Field) -> Option<u32> {
    let iwd2 = matches!(cre.header, CreHeader::V22(_));
    let religion = game == Game::Pstee;
    Some(match field {
        Field::Gender => cre.gender().into(),
        Field::Race => cre.race().into(),
        Field::SubRace => cre.subrace()?.into(),
        Field::Alignment => cre.alignment().into(),
        Field::Class if !iwd2 => cre.class().into(),
        Field::Fallen => cre.is_fallen().into(),
        Field::OriginalClass if !iwd2 => cre
            .dual_class_original_class()
            .map_or(0, original_class_value),
        Field::Kit if !religion => cre.kit()?,
        Field::Deity if religion => cre.kit()? & 0xFFFF,
        Field::MageType if religion => cre.kit()? >> 16,
        Field::RacialEnemy => cre.racial_enemy()?.into(),
        Field::EnemyAlly => cre.enemy_ally().into(),
        Field::State => cre.state_flags(),
        Field::Class | Field::OriginalClass | Field::Kit | Field::Deity | Field::MageType => {
            return None;
        }
    })
}

/// Write `value` into the creature's `field`.
fn set_value(cre: &mut Cre, field: Field, value: u32) {
    match (field, u8::try_from(value).ok()) {
        (Field::Gender, Some(byte)) => cre.set_gender(byte),
        (Field::Race, Some(byte)) => cre.set_race(byte),
        (Field::SubRace, Some(byte)) => cre.set_subrace(byte),
        (Field::Alignment, Some(byte)) => cre.set_alignment(byte),
        (Field::Class, Some(byte)) => cre.set_class(byte),
        (Field::RacialEnemy, Some(byte)) => cre.set_racial_enemy(byte),
        (Field::EnemyAlly, Some(byte)) => cre.set_enemy_ally(byte),
        (Field::OriginalClass, _) => cre.set_dual_class_original_class(original_class(value)),
        (Field::Kit, _) => cre.set_kit(value),
        (Field::Deity, _) => {
            if let Some(kit) = cre.kit() {
                cre.set_kit((kit & 0xFFFF_0000) | (value & 0xFFFF));
            }
        }
        (Field::MageType, _) => {
            if let Some(kit) = cre.kit() {
                cre.set_kit((kit & 0xFFFF) | (value << 16));
            }
        }
        (Field::State, _) => cre.set_state_flags(value),
        // A checkbox (see `commit`), or too large for its byte.
        (Field::Fallen, _) | (_, None) => {}
    }
}

/// An original class as a choice's value: 0 is none.
fn original_class_value(class: OriginalClass) -> u32 {
    OriginalClass::ALL
        .iter()
        .position(|&each| each == class)
        .and_then(|index| u32::try_from(index + 1).ok())
        .unwrap_or(0)
}

/// The original class a choice's value stands for.
fn original_class(value: u32) -> Option<OriginalClass> {
    let index = usize::try_from(value.checked_sub(1)?).ok()?;
    OriginalClass::ALL.get(index).copied()
}

// ── Choices ───────────────────────────────────────────────────────────

/// `field`'s dropdown: its choices, the creature's value first (by number)
/// when they lack it, and that value's place. `None` where the creature has
/// no such field to edit.
fn choices(game_data: &GameData, cre: &Cre, field: Field) -> Option<(Choices, usize)> {
    let current = value(game_data.game(), cre, field)?;
    let mut choices = game_choices(game_data, cre, field);
    let index = choices
        .iter()
        .position(|&(value, _)| value == current)
        .unwrap_or_else(|| {
            choices.insert(0, (current, unlisted(field, current)));
            0
        });
    Some((choices, index))
}

/// How a value the game's tables lack is named: by its number.
fn unlisted(field: Field, value: u32) -> String {
    match field {
        Field::State => format!("0x{value:08X}"),
        Field::Kit | Field::Deity | Field::MageType => format!("0x{value:X}"),
        _ => value.to_string(),
    }
}

/// Every value `field` can take, by name, as the game's tables list them.
fn game_choices(game_data: &GameData, cre: &Cre, field: Field) -> Choices {
    const BYTE: u32 = 0xFF;
    match field {
        Field::Gender => ids_choices(game_data, "gender", " ", BYTE),
        Field::Race => ids_choices(game_data, "race", " ", BYTE),
        Field::SubRace => subrace_choices(game_data, cre.race()),
        Field::Alignment if matches!(cre.header, CreHeader::V22(_)) => aligns_choices(game_data),
        Field::Alignment => ids_choices(game_data, "alignmen", " ", BYTE),
        Field::Class => ids_choices(game_data, "class", " / ", BYTE),
        // A checkbox.
        Field::Fallen => Vec::new(),
        Field::OriginalClass => with_none(
            OriginalClass::ALL
                .iter()
                .map(|&class| (original_class_value(class), pretty(class.symbol(), " ")))
                .collect(),
        ),
        Field::Kit => with_none(kit_choices(game_data)),
        Field::Deity => {
            let deities = ids_choices(game_data, "deity", " ", 0xFFFF);
            with_none(if deities.is_empty() {
                ids_choices(game_data, "diety", " ", 0xFFFF)
            } else {
                deities
            })
        }
        Field::MageType => with_none(mage_type_choices(game_data)),
        Field::RacialEnemy => with_none(racial_enemy_choices(game_data)),
        Field::EnemyAlly => ids_choices(game_data, "ea", " ", BYTE),
        Field::State => ids_choices(game_data, "state", " ", u32::MAX),
    }
}

/// `choices` with "None" (0) first, when they don't name 0.
fn with_none(mut choices: Choices) -> Choices {
    if !choices.iter().any(|&(value, _)| value == 0) {
        choices.insert(0, (0, "None".to_owned()));
    }
    choices
}

/// The `file` IDS's entries up to `max`, each value once (under its first
/// name), named by [`pretty`] with `separator` between words.
fn ids_choices(game_data: &GameData, file: &str, separator: &str, max: u32) -> Choices {
    let Ok(ids) = game_data.import_ids_by_name(file) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    ids.entries
        .iter()
        // A negative value is one with the high bit set (`STATE.IDS`'s
        // 0x80000000).
        .map(|entry| (entry.value.cast_unsigned(), &entry.name))
        .filter(|&(value, _)| value <= max && seen.insert(value))
        .map(|(value, name)| (value, pretty(name, separator)))
        .collect()
}

/// IWD2's subraces of `race`, numbered within it: `SUBRACE.IDS` packs them
/// as `race << 16 | subrace`, its 0 the pure race.
fn subrace_choices(game_data: &GameData, race: u8) -> Choices {
    ids_choices(game_data, "subrace", " ", u32::MAX)
        .into_iter()
        .filter(|&(packed, _)| packed == 0 || packed >> 16 == u32::from(race))
        .map(|(packed, name)| (packed & 0xFFFF, name))
        .filter(|&(subrace, _)| subrace <= 0xFF)
        .collect()
}

/// IWD2's alignments: `ALIGNS.2DA`'s rows by their `VALUE`, in its order.
fn aligns_choices(game_data: &GameData) -> Choices {
    let Ok(aligns) = game_data.import_2da_by_name("aligns") else {
        return Vec::new();
    };
    let Some(column) = aligns
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case("VALUE"))
    else {
        return Vec::new();
    };
    let mut choices: Choices = aligns
        .rows
        .iter()
        .filter_map(|(name, cells)| {
            let value = u32::try_from(parse_2da_int(cells.get(column)?)?).ok()?;
            (value <= 0xFF).then(|| (value, pretty(name, " ")))
        })
        .collect();
    // The rows come unordered.
    choices.sort_by_key(|&(value, _)| value);
    choices.dedup_by_key(|(value, _)| *value);
    choices
}

/// The game's kits, named by `KITLIST.2DA`'s strrefs.
fn kit_choices(game_data: &GameData) -> Choices {
    kits(game_data)
        .into_iter()
        .map(|kit| {
            let name = if kit.kit == TRUE_CLASS {
                "Base Class".to_owned()
            } else {
                kit.name_strref
                    .map(|strref| resolve_strref(game_data, strref))
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| pretty(&kit.symbol, " "))
            };
            (kit.kit, name)
        })
        .collect()
}

/// PSTEE's mage types: `MAGESPEC.IDS`'s, else the schools' own bits.
fn mage_type_choices(game_data: &GameData) -> Choices {
    let choices = ids_choices(game_data, "magespec", " ", 0xFFFF);
    if !choices.is_empty() {
        return choices;
    }
    [
        (0x0040, "Abjurer"),
        (0x0080, "Conjurer"),
        (0x0100, "Diviner"),
        (0x0200, "Enchanter"),
        (0x0400, "Illusionist"),
        (0x0800, "Invoker"),
        (0x1000, "Necromancer"),
        (0x2000, "Transmuter"),
        (0x4000, "Generalist"),
    ]
    .into_iter()
    .map(|(value, name)| (value, name.to_owned()))
    .collect()
}

/// The races a ranger can hate: `RACE.IDS`'s, named as `HATERACE.2DA`
/// names them where it does.
fn racial_enemy_choices(game_data: &GameData) -> Choices {
    ids_choices(game_data, "race", " ", 0xFF)
        .into_iter()
        .filter(|&(race, _)| race != 0)
        .map(|(race, name)| {
            let hated = u8::try_from(race)
                .ok()
                .and_then(|race| racial_enemy_strref(game_data, race))
                .map(|strref| resolve_strref(game_data, strref))
                .filter(|hated| !hated.is_empty());
            (race, hated.unwrap_or(name))
        })
        .collect()
}

// ── Names ─────────────────────────────────────────────────────────────

/// The name `file`'s IDS gives `value`.
fn ids_symbol(game_data: &GameData, file: &str, value: i32) -> Option<String> {
    let ids = game_data.import_ids_by_name(file).ok()?;
    ids.entries
        .iter()
        .find(|entry| entry.value == value)
        .map(|entry| entry.name.clone())
}

/// An IDS or 2DA symbol as words: `LAWFUL_GOOD` → `Lawful Good`, with
/// `separator` between them.
fn pretty(symbol: &str, separator: &str) -> String {
    symbol
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(separator)
}

fn parse_2da_int(cell: &str) -> Option<i64> {
    let cell = cell.trim();
    match cell.strip_prefix("0x").or_else(|| cell.strip_prefix("0X")) {
        Some(hex) => i64::from_str_radix(hex, 16).ok(),
        None => cell.parse().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_ids_round_trip() {
        for field in Field::ALL {
            assert_eq!(Field::from_id(field.id()), Some(field));
        }
        assert_eq!(Field::from_id(-1), None);
        assert_eq!(Field::from_id(14), None);
    }

    #[test]
    fn original_classes_round_trip_through_their_values() {
        assert_eq!(original_class(0), None);
        for class in OriginalClass::ALL {
            assert_eq!(original_class(original_class_value(class)), Some(class));
        }
    }

    #[test]
    fn none_comes_first_unless_named() {
        let named = vec![(0, "No Race".to_owned()), (1, "Human".to_owned())];
        assert_eq!(with_none(named.clone()), named);
        assert_eq!(
            with_none(vec![(1, "Human".to_owned())]),
            [(0, "None".to_owned()), (1, "Human".to_owned())]
        );
    }

    #[test]
    fn symbols_read_as_words() {
        assert_eq!(pretty("LAWFUL_GOOD", " "), "Lawful Good");
        assert_eq!(pretty("FIGHTER_MAGE", " / "), "Fighter / Mage");
        assert_eq!(parse_2da_int("0x11"), Some(17));
        assert_eq!(parse_2da_int(" 34 "), Some(34));
    }
}
