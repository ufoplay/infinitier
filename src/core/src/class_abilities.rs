//! What each base class and kit gives a character level by level: its CLAB
//! table, a base class's named in GemRB's `clskills.2da` (the engines
//! hardcode them), a kit's in `KITLIST.2DA`'s `ABILITIES` column. A table
//! has a column per level, and each cell of a level's column is a spell:
//! `AP_` applied to the character, whose effects stay on it; `GA_` granted
//! as an innate ability, one more use each time. (IWD2's `FA_`, `FS_` and
//! `RA_` cells, and its tables, are its own.)
//!
//! [`spell_effects`] turns an applied spell into the effects the game keeps
//! on the character, as it writes them: copies of the spell's effects, with
//! the spell as their source.

use infinitier_cre_resource::Effect;
use infinitier_spl_resource::{Spl, SplEffect};
use infinitier_two_da_resource::TwoDA;

use crate::game::GameData;
use crate::imported_resource::cre::kits;

/// A table of class abilities: a base class's or a kit's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityTable {
    /// The class's or kit's name.
    pub name: String,
    /// The CLAB table, e.g. `CLABPA03`.
    pub table: String,
    pub owner: Owner,
}

/// Whose an [`AbilityTable`] is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// A base class, by its `CLASS.IDS` symbol (`PALADIN`).
    BaseClass(String),
    /// A kit, by its CRE kit dword.
    Kit(u32),
}

/// What a CLAB cell gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grant {
    /// `AP_`: the spell is applied to the character; its effects stay.
    Effects,
    /// `GA_`: the spell is an innate ability, one more use.
    Innate,
}

/// One CLAB cell: what the character gets at `level`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAbility {
    /// The level (1-based) whose column it's in.
    pub level: u32,
    pub grant: Grant,
    /// The spell's resref, in capitals.
    pub spell: String,
}

/// The source type of an effect a spell applied (the effect's "parent
/// resource type").
const FROM_SPELL: u32 = 1;

/// Every base class's ability table (by name), then every kit's (in
/// `KITLIST.2DA` order). Empty for a game without them.
pub fn ability_tables(game_data: &GameData) -> Vec<AbilityTable> {
    let mut classes: Vec<AbilityTable> = game_data
        .import_2da_by_name("clskills")
        .ok()
        .map(|clskills| base_class_tables(&clskills))
        .unwrap_or_default();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let tlk = game_data.dialog_tlk().ok();
    let kit_tables = kits(game_data).into_iter().filter_map(|kit| {
        let table = kit.abilities?;
        let name = kit
            .name_strref
            .and_then(|strref| tlk.as_ref()?.get(strref))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| title_case(&kit.symbol));
        Some(AbilityTable {
            name,
            table,
            owner: Owner::Kit(kit.kit),
        })
    });
    classes.into_iter().chain(kit_tables).collect()
}

/// The single classes' tables in `clskills.2da`: a multi-class row lists
/// its classes' (`FI01,TH01`), which are theirs.
fn base_class_tables(clskills: &TwoDA) -> Vec<AbilityTable> {
    let Some(col) = clskills
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case("ABILITIES"))
    else {
        return Vec::new();
    };
    clskills
        .rows
        .iter()
        .filter(|(class, _)| !class.contains('_'))
        .filter_map(|(class, cells)| {
            let table = cells.get(col)?;
            let table = table.to_ascii_uppercase();
            table.starts_with("CLAB").then(|| AbilityTable {
                name: title_case(class),
                table,
                owner: Owner::BaseClass(class.to_ascii_uppercase()),
            })
        })
        .collect()
}

/// What `table` gives, level by level (and in row order within a level).
/// Empty when the game lacks it.
pub fn class_abilities(game_data: &GameData, table: &str) -> Vec<ClassAbility> {
    game_data
        .import_2da_by_name(table)
        .map(|clab| abilities_of(&clab))
        .unwrap_or_default()
}

fn abilities_of(clab: &TwoDA) -> Vec<ClassAbility> {
    // Rows are ABILITY1, ABILITY2…: by their number, not as text.
    let mut rows: Vec<(&String, &Vec<String>)> = clab.rows.iter().collect();
    rows.sort_by_key(|(label, _)| {
        let digits: String = label.chars().filter(char::is_ascii_digit).collect();
        (digits.parse::<u32>().unwrap_or(u32::MAX), (*label).clone())
    });
    let mut abilities: Vec<ClassAbility> = Vec::new();
    for (col, header) in clab.headers.iter().enumerate() {
        let Ok(level) = header.trim().parse::<u32>() else {
            continue;
        };
        for (_, cells) in &rows {
            let Some(cell) = cells.get(col) else {
                continue;
            };
            let cell = cell.to_ascii_uppercase();
            let grant = if cell.starts_with("AP_") {
                Grant::Effects
            } else if cell.starts_with("GA_") {
                Grant::Innate
            } else {
                continue;
            };
            abilities.push(ClassAbility {
                level,
                grant,
                spell: cell[3..].to_owned(),
            });
        }
    }
    abilities.sort_by_key(|ability| ability.level);
    abilities
}

/// The effects applying `spell` at `level` leaves on a character, as the
/// game writes them: its ability for that level (the highest one the level
/// reaches, else the first), each effect copied with the spell as its
/// source and `level` as its caster level. Empty when the game lacks it.
pub fn spell_effects(game_data: &GameData, spell: &str, level: u32) -> Vec<Effect> {
    let Ok(spl) = game_data.import_spl_by_name(spell) else {
        return Vec::new();
    };
    ability_effects(spl.spl(), level)
        .iter()
        .map(|feature| applied(feature, spell, level))
        .collect()
}

/// The effects of `spl`'s ability for `level`.
fn ability_effects(spl: &Spl, level: u32) -> &[SplEffect] {
    let windows = spl.abilities.iter().map(|ability| {
        (
            ability.level_required,
            ability.first_effect_index,
            ability.num_effects,
        )
    });
    ability_window(windows, level)
        .and_then(|range| spl.effects.get(range))
        .unwrap_or(&[])
}

/// Of abilities given as (level required, first effect, effect count), the
/// effects of the one for `level`: the highest it reaches, else the first.
fn ability_window(
    abilities: impl Iterator<Item = (u16, u16, u16)> + Clone,
    level: u32,
) -> Option<std::ops::Range<usize>> {
    let (_, first, count) = abilities
        .clone()
        .filter(|&(required, _, _)| u32::from(required) <= level)
        .max_by_key(|&(required, _, _)| required)
        .or_else(|| abilities.clone().next())?;
    let first = usize::from(first);
    Some(first..first + usize::from(count))
}

/// `feature` of `spell`, applied at `level`, as the creature keeps it.
fn applied(feature: &SplEffect, spell: &str, level: u32) -> Effect {
    let mut effect = Effect::permanent(
        feature.opcode.into(),
        feature.parameter1,
        feature.parameter2,
    );
    effect.target = feature.target_type.into();
    effect.power = feature.power.into();
    effect.timing_mode = feature.timing_mode;
    effect.resistance = feature.resistance;
    effect.duration = feature.duration;
    effect.probability1 = feature.probability1.into();
    effect.probability2 = feature.probability2.into();
    effect.resource = feature.resource.to_ascii_uppercase();
    effect.dice_thrown = feature.dice_thrown;
    effect.dice_sides = feature.dice_sides;
    effect.save_type = feature.save_type;
    effect.save_bonus = u32::from_le_bytes(feature.save_bonus.to_le_bytes());
    effect.special = feature.trailing_extra;
    effect.parent_resource_type = FROM_SPELL;
    effect.parent_resource = spell.to_ascii_uppercase();
    effect.caster_level = level;
    // As the game leaves an effect it has applied.
    effect.first_apply = 1;
    effect
}

/// `PALADIN` → `Paladin`, `TOTEMIC_DRUID` → `Totemic Druid`.
fn title_case(symbol: &str) -> String {
    symbol
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            let mut chars = lower.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use infinitier_cre_resource::{Resistance, SaveType, TimingMode};

    use super::*;

    fn two_da(headers: &[&str], rows: &[(&str, &[&str])]) -> TwoDA {
        TwoDA {
            headers: headers.iter().map(|h| (*h).to_owned()).collect(),
            default: "****".to_owned(),
            rows: rows
                .iter()
                .map(|(label, cells)| {
                    let cells = cells.iter().map(|c| (*c).to_owned()).collect();
                    ((*label).to_owned(), cells)
                })
                .collect::<HashMap<_, _>>(),
        }
    }

    #[test]
    fn a_clab_lists_its_spells_by_level_then_row() {
        let clab = two_da(
            &["1", "2", "3"],
            &[
                ("ABILITY10", &["GA_SPCL213", "****", "****"]),
                ("ABILITY1", &["ap_spcl233b", "****", "AP_SPCL233B"]),
                ("ABILITY2", &["GA_SPCL212", "GA_SPCL212", "****"]),
            ],
        );
        let got: Vec<(u32, Grant, String)> = abilities_of(&clab)
            .into_iter()
            .map(|a| (a.level, a.grant, a.spell))
            .collect();
        let expected: Vec<(u32, Grant, String)> = [
            (1, Grant::Effects, "SPCL233B"),
            (1, Grant::Innate, "SPCL212"),
            (1, Grant::Innate, "SPCL213"),
            (2, Grant::Innate, "SPCL212"),
            (3, Grant::Effects, "SPCL233B"),
        ]
        .into_iter()
        .map(|(level, grant, spell)| (level, grant, spell.to_owned()))
        .collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn the_base_classes_are_the_single_classes_tables() {
        let clskills = two_da(
            &["DRUIDSPELL", "ABILITIES"],
            &[
                ("PALADIN", &["*", "CLABPA01"]),
                ("FIGHTER_THIEF", &["*", "FI01,TH01"]),
                ("FIGHTER_MAGE", &["*", "CLABFI01"]),
                ("SORCERER", &["*", "*"]),
            ],
        );
        assert_eq!(
            base_class_tables(&clskills),
            [AbilityTable {
                name: "Paladin".to_owned(),
                table: "CLABPA01".to_owned(),
                owner: Owner::BaseClass("PALADIN".to_owned()),
            }]
        );
    }

    #[test]
    fn an_applied_effect_names_its_spell_and_level() {
        let feature = SplEffect {
            opcode: 101,
            target_type: 2,
            power: 0,
            parameter1: 0,
            parameter2: 5,
            timing_mode: TimingMode::PermanentAfterDeath,
            resistance: Resistance::BypassResistance,
            duration: 0,
            probability1: 100,
            probability2: 0,
            resource: "spnwchrm".to_owned(),
            dice_thrown: 0,
            dice_sides: 0,
            save_type: SaveType::empty(),
            save_bonus: -2,
            trailing_extra: 0,
        };
        let effect = applied(&feature, "spcl233b", 15);
        assert_eq!((effect.opcode, effect.param2, effect.target), (101, 5, 2));
        assert_eq!(effect.resource, "SPNWCHRM");
        assert_eq!(effect.parent_resource, "SPCL233B");
        assert_eq!(effect.parent_resource_type, FROM_SPELL);
        assert_eq!(effect.caster_level, 15);
        assert_eq!(effect.save_bonus, u32::MAX - 1);
        assert_eq!(effect.parent_resource_slot, u32::MAX);
    }

    #[test]
    fn a_spell_applies_the_ability_its_level_reaches() {
        let abilities = [(1, 0, 2), (10, 2, 3)];
        let window = |level| ability_window(abilities.iter().copied(), level);
        assert_eq!(window(9), Some(0..2));
        assert_eq!(window(10), Some(2..5));
        // Below every ability: the first.
        assert_eq!(window(0), Some(0..2));
        assert_eq!(ability_window(std::iter::empty(), 5), None);
    }

    #[test]
    fn a_symbol_reads_as_a_name() {
        assert_eq!(title_case("TOTEMIC_DRUID"), "Totemic Druid");
        assert_eq!(title_case("PALADIN"), "Paladin");
    }
}
