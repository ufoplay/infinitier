//! The Kit Features dialog (opened from the Special Edits card): add or
//! remove what a base class or kit gives a character from one level to
//! another, as the game does when the character levels up (see
//! [`class_abilities`]): the effects of the spells it applies, and its
//! innate abilities.
//!
//! Add gives, for each level, what that level's column lists: a spell's
//! effects once per level that applies it, an innate one more use per
//! level that grants it. Remove takes away every effect of the spells the
//! levels apply and every innate ability they grant, however many times
//! they were added, and whatever added them (the game, when it levelled
//! up, or an editor). Either closes the dialog and leaves a notice of
//! what it did (see [`Model::notify`]).
//!
//! [`class_abilities`]: infinitier_core::class_abilities

use std::collections::BTreeSet;

use infinitier_core::class_abilities::{
    AbilityTable, ClassAbility, Grant, Owner, ability_tables, class_abilities, spell_effects,
};
use infinitier_core::game::GameData;
use infinitier_core::resource::cre::{Cre, CreHeader, EffectList};

use crate::state::{Model, NoticeKind};

/// The highest level a CLAB table has a column for.
pub const MAX_LEVEL: u32 = 50;

/// Which of a table's grants the dialog adds or removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Both,
    Effects,
    Innates,
}

impl Part {
    pub const ALL: [Self; 3] = [Self::Both, Self::Effects, Self::Innates];

    pub fn label(self) -> &'static str {
        match self {
            Self::Both => "Effects and Innates",
            Self::Effects => "Effects",
            Self::Innates => "Innates",
        }
    }

    fn includes(self, grant: Grant) -> bool {
        match (self, grant) {
            (Self::Both, _) | (Self::Effects, Grant::Effects) | (Self::Innates, Grant::Innate) => {
                true
            }
            (Self::Effects, Grant::Innate) | (Self::Innates, Grant::Effects) => false,
        }
    }
}

/// One line of the dialog's preview: what the table gives at a level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub level: u32,
    pub grant: Grant,
    /// The spell's name and resref, or its resref alone.
    pub spell: String,
}

/// The open Kit Features dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitFeatures {
    tables: Vec<AbilityTable>,
    table: usize,
    /// What the selected table gives, every level.
    abilities: Vec<ClassAbility>,
    part: Part,
    from: u32,
    to: u32,
    preview: Vec<Preview>,
}

impl KitFeatures {
    /// The tables' names, base classes first.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.tables.iter().map(|table| table.name.as_str())
    }

    pub fn table(&self) -> usize {
        self.table
    }

    pub fn part(&self) -> Part {
        self.part
    }

    pub fn levels(&self) -> (u32, u32) {
        (self.from, self.to)
    }

    pub fn preview(&self) -> &[Preview] {
        &self.preview
    }

    /// What's selected, in words: "Inquisitor, levels 1 to 25".
    fn what(&self) -> String {
        let name = self
            .tables
            .get(self.table)
            .map_or("", |table| table.name.as_str());
        format!("{name}, levels {} to {}", self.from, self.to)
    }

    /// That the selection holds nothing, in words.
    fn nothing(&self) -> String {
        let name = self
            .tables
            .get(self.table)
            .map_or("", |table| table.name.as_str());
        let part = match self.part {
            Part::Both => "effects or innates",
            Part::Effects => "effects",
            Part::Innates => "innates",
        };
        format!(
            "{name} gives no {part} in levels {} to {}.",
            self.from, self.to
        )
    }

    /// What the selected table gives from `from` to `to`, of `part`.
    fn selected(&self) -> impl Iterator<Item = &ClassAbility> {
        self.abilities.iter().filter(|ability| {
            (self.from..=self.to).contains(&ability.level) && self.part.includes(ability.grant)
        })
    }

    /// Load the table at `index`, and show what it gives.
    fn select_table(&mut self, game_data: &GameData, index: usize) {
        let Some(table) = self.tables.get(index) else {
            return;
        };
        self.table = index;
        self.abilities = class_abilities(game_data, &table.table);
        self.update_preview(game_data);
    }

    fn update_preview(&mut self, game_data: &GameData) {
        let tlk = game_data.dialog_tlk().ok();
        let name = |spell: &str| {
            let name = game_data
                .import_spl_by_name(spell)
                .ok()
                .and_then(|spl| tlk.as_ref()?.get(spl.spl().header.name_strref()))
                .filter(|name| !name.trim().is_empty());
            match name {
                Some(name) => format!("{name} [{spell}]"),
                None => spell.to_owned(),
            }
        };
        self.preview = self
            .selected()
            .map(|ability| Preview {
                level: ability.level,
                grant: ability.grant,
                spell: name(&ability.spell),
            })
            .collect();
    }
}

/// Whether the dialog is for `cre`: an AD&D creature (IWD2's tables are
/// its own).
pub fn supports(cre: &Cre) -> bool {
    !matches!(cre.header, CreHeader::V22(_))
}

/// Open the dialog on the character's own kit (or class), from level 1 to
/// its level.
pub fn open(model: &mut Model) {
    let Some(cre) = model.selected_cre() else {
        return;
    };
    let game_data = &model.game_data;
    let tables = ability_tables(game_data);
    let table = own_table(game_data, cre, &tables).unwrap_or(0);
    let to = cre
        .level_first_class()
        .map_or(1, u32::from)
        .clamp(1, MAX_LEVEL);
    let mut features = KitFeatures {
        tables,
        table,
        abilities: Vec::new(),
        part: Part::Both,
        from: 1,
        to,
        preview: Vec::new(),
    };
    features.select_table(game_data, table);
    model.kit_features = Some(features);
}

/// The index of `cre`'s kit's table in `tables`, else its (first) class's.
fn own_table(game_data: &GameData, cre: &Cre, tables: &[AbilityTable]) -> Option<usize> {
    let by_kit = cre.kit().and_then(|kit| {
        tables
            .iter()
            .position(|table| table.owner == Owner::Kit(kit))
    });
    by_kit.or_else(|| {
        let ids = game_data.import_ids_by_name("CLASS").ok()?;
        let symbol = ids
            .entries
            .iter()
            .find(|entry| entry.value == i32::from(cre.class()))
            .map(|entry| entry.name.to_ascii_uppercase())?;
        // A multi-class's first class.
        let first = symbol.split('_').next().unwrap_or_default().to_owned();
        tables
            .iter()
            .position(|table| table.owner == Owner::BaseClass(first.clone()))
    })
}

/// Run `edit` on the open dialog, with the game's data, then show what its
/// table gives again.
fn edit(model: &mut Model, change: impl FnOnce(&mut KitFeatures, &GameData)) {
    let Model {
        game_data,
        kit_features,
        ..
    } = model;
    if let Some(features) = kit_features.as_mut() {
        change(features, game_data);
        features.update_preview(game_data);
    }
}

pub fn select_table(model: &mut Model, index: i32) {
    if let Ok(index) = usize::try_from(index) {
        edit(model, |features, game_data| {
            features.select_table(game_data, index);
        });
    }
}

pub fn select_part(model: &mut Model, index: i32) {
    let part = usize::try_from(index)
        .ok()
        .and_then(|index| Part::ALL.get(index));
    if let Some(&part) = part {
        edit(model, |features, _| features.part = part);
    }
}

/// A level as the user set it: a whole number in range.
fn level(value: f32) -> u32 {
    // In range, so a whole number from 1 to MAX_LEVEL.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let level = value.round().clamp(1.0, MAX_LEVEL as f32) as u32;
    level
}

/// Set the first level; the last follows it up when it would come before.
pub fn set_from(model: &mut Model, value: f32) {
    edit(model, |features, _| {
        features.from = level(value);
        features.to = features.to.max(features.from);
    });
}

/// Set the last level; the first follows it down when it would come after.
pub fn set_to(model: &mut Model, value: f32) {
    edit(model, |features, _| {
        features.to = level(value);
        features.from = features.from.min(features.to);
    });
}

/// Add what the selected levels give.
pub fn add(model: &mut Model) {
    let Some(features) = model.kit_features.as_ref() else {
        return;
    };
    let game_data = &model.game_data;
    let mut effects = Vec::new();
    let mut innates = Vec::new();
    for ability in features.selected() {
        match ability.grant {
            Grant::Effects => {
                effects.extend(spell_effects(game_data, &ability.spell, ability.level))
            }
            Grant::Innate => innates.push(ability.spell.clone()),
        }
    }
    let (what, nothing) = (features.what(), features.nothing());
    let who = model.character_name();
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    let effect_count = effects.len();
    let effects_added = effects.is_empty() || cre.add_effects(effects);
    let innate_uses = innates
        .iter()
        .filter(|spell| cre.add_innate_use(spell))
        .count();
    if !effects_added {
        finish(
            model,
            NoticeKind::Warning,
            "Kit features added",
            format!(
                "Added {innate_uses} innate uses to {who} ({what}). Its effects are in the \
                 classic format, which has no room for their source: its {effect_count} \
                 effects weren't added."
            ),
        );
    } else if effect_count == 0 && innate_uses == 0 {
        finish(model, NoticeKind::Warning, "Nothing added", nothing);
    } else {
        finish(
            model,
            NoticeKind::Done,
            "Kit features added",
            format!(
                "Added {effect_count} effects and {innate_uses} innate uses to {who} ({what})."
            ),
        );
    }
}

/// Remove what the selected levels give: every effect of their spells and
/// every innate ability they grant.
pub fn remove(model: &mut Model) {
    let Some(features) = model.kit_features.as_ref() else {
        return;
    };
    let spells = |grant: Grant| -> BTreeSet<String> {
        features
            .selected()
            .filter(|ability| ability.grant == grant)
            .map(|ability| ability.spell.clone())
            .collect()
    };
    let (sources, innates) = (spells(Grant::Effects), spells(Grant::Innate));
    let what = features.what();
    let who = model.character_name();
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    let classic = matches!(cre.effect_list(), EffectList::V1(_)) && !sources.is_empty();
    let effects: usize = sources
        .iter()
        .map(|spell| cre.remove_effects_from(spell))
        .sum();
    let abilities = innates
        .iter()
        .filter(|spell| cre.remove_innate(spell))
        .count();
    if classic {
        finish(
            model,
            NoticeKind::Warning,
            "Kit features removed",
            format!(
                "Removed {abilities} innate abilities from {who} ({what}). Its effects are in \
                 the classic format, which doesn't record their source: none could be removed."
            ),
        );
    } else if effects == 0 && abilities == 0 {
        finish(
            model,
            NoticeKind::Warning,
            "Nothing removed",
            format!("{who} has none of what {what} gives."),
        );
    } else {
        finish(
            model,
            NoticeKind::Done,
            "Kit features removed",
            format!(
                "Removed {effects} effects and {abilities} innate abilities from {who} ({what})."
            ),
        );
    }
}

/// Close the dialog, leaving a notice of what it did.
fn finish(model: &mut Model, kind: NoticeKind, title: &str, text: String) {
    model.kit_features = None;
    model.notify(kind, title, text);
}

pub fn close(model: &mut Model) {
    model.kit_features = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ability(level: u32, grant: Grant, spell: &str) -> ClassAbility {
        ClassAbility {
            level,
            grant,
            spell: spell.to_owned(),
        }
    }

    fn features(part: Part, from: u32, to: u32) -> KitFeatures {
        KitFeatures {
            tables: Vec::new(),
            table: 0,
            abilities: vec![
                ability(1, Grant::Effects, "SPCL233B"),
                ability(1, Grant::Innate, "SPCL212"),
                ability(5, Grant::Innate, "SPCL231"),
                ability(15, Grant::Effects, "SPCL233B"),
            ],
            part,
            from,
            to,
            preview: Vec::new(),
        }
    }

    fn spells(features: &KitFeatures) -> Vec<(u32, &str)> {
        features
            .selected()
            .map(|ability| (ability.level, ability.spell.as_str()))
            .collect()
    }

    #[test]
    fn the_selection_is_the_levels_and_the_part() {
        assert_eq!(
            spells(&features(Part::Both, 1, 5)),
            [(1, "SPCL233B"), (1, "SPCL212"), (5, "SPCL231")]
        );
        assert_eq!(
            spells(&features(Part::Effects, 1, 20)),
            [(1, "SPCL233B"), (15, "SPCL233B")]
        );
        assert_eq!(spells(&features(Part::Innates, 2, 20)), [(5, "SPCL231")]);
    }

    #[test]
    fn a_level_is_whole_and_in_range() {
        assert_eq!(level(0.0), 1);
        assert_eq!(level(7.4), 7);
        assert_eq!(level(99.0), MAX_LEVEL);
    }
}
