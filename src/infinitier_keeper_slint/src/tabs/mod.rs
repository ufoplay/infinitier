//! The character tabs: one module per tab building its content for the
//! Slint pages in `ui/tabs/` and committing its edits, like the egui
//! keeper's `ui/tabs/`. The [`CharacterTab`] enum is duplicated from the
//! egui keeper's `ui/tabs/mod.rs`.

pub mod abilities;
pub mod appearance;
pub mod characteristics;
pub mod effects;
pub mod feats;
pub mod inventory;
pub mod journal;
pub mod levels;
pub mod memorization;
pub mod miscellaneous;
pub mod proficiencies;
pub mod resistances;
pub mod spells;
pub mod variables;

use infinitier_core::resource::{Engine, Game};

/// Identifier for the active per-character tab. Order matches the
/// EEKeeper tab strip. Duplicated from the egui keeper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterTab {
    Abilities,
    Levels,
    Characteristics,
    Appearance,
    Inventory,
    Spells,
    Feats,
    Memorization,
    Proficiencies,
    Resistances,
    Effects,
    LocalVariables,
    GlobalVariables,
    JournalEntries,
    Miscellaneous,
}

impl CharacterTab {
    /// Tabs in EEKeeper display order.
    pub const ALL: &'static [CharacterTab] = &[
        CharacterTab::Abilities,
        CharacterTab::Levels,
        CharacterTab::Characteristics,
        CharacterTab::Appearance,
        CharacterTab::Inventory,
        CharacterTab::Feats,
        CharacterTab::Memorization,
        CharacterTab::Spells,
        CharacterTab::Proficiencies,
        CharacterTab::Resistances,
        CharacterTab::Effects,
        CharacterTab::LocalVariables,
        CharacterTab::GlobalVariables,
        CharacterTab::JournalEntries,
        CharacterTab::Miscellaneous,
    ];

    /// True when this tab should appear for the given game. `Spells` is
    /// universal; `Levels`/`Feats` are IWD2-only, and
    /// `Memorization`/`Proficiencies` are AD&D-only.
    pub fn is_visible_for_game(&self, game: Game) -> bool {
        let is_iwd2 = game.engine() == Engine::Iwd2;
        match self {
            CharacterTab::Levels | CharacterTab::Feats => is_iwd2,
            CharacterTab::Memorization | CharacterTab::Proficiencies => !is_iwd2,
            CharacterTab::Abilities
            | CharacterTab::Characteristics
            | CharacterTab::Appearance
            | CharacterTab::Inventory
            | CharacterTab::Spells
            | CharacterTab::Resistances
            | CharacterTab::Effects
            | CharacterTab::LocalVariables
            | CharacterTab::GlobalVariables
            | CharacterTab::JournalEntries
            | CharacterTab::Miscellaneous => true,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CharacterTab::Abilities => "Abilities",
            CharacterTab::Levels => "Levels & Kits",
            CharacterTab::Characteristics => "Characteristics",
            CharacterTab::Appearance => "Appearance",
            CharacterTab::Inventory => "Inventory",
            CharacterTab::Spells => "Spells",
            CharacterTab::Feats => "Feats",
            CharacterTab::Memorization => "Memorization",
            CharacterTab::Proficiencies => "Proficiencies",
            CharacterTab::Resistances => "Resistances",
            CharacterTab::Effects => "Effects",
            CharacterTab::LocalVariables => "Local Variables",
            CharacterTab::GlobalVariables => "Global Variables",
            CharacterTab::JournalEntries => "Journal Entries",
            CharacterTab::Miscellaneous => "Miscellaneous",
        }
    }
}
