//! App-wide state for the Slint keeper.
//!
//! A trimmed analogue of the egui keeper's `state.rs`: for now the Slint
//! keeper opens a single save from the CLI (no multi-tab strip yet), so the
//! per-save fields live directly on [`Model`] rather than in a `Vec<SaveTab>`.
//! The [`CharacterTab`] enum is duplicated from the egui keeper's
//! `ui/tabs/mod.rs`.

use infinitier_core::engine_caps::EngineCaps;
use infinitier_core::game::GameData;
use infinitier_core::imported_resource::cre::ImportedCre;
use infinitier_core::imported_resource::gam::{ImportedGam, ImportedGamNpc, NpcCre};
use infinitier_core::resource::cre::{Cre, Iwd2Spellbook, SpellType};
use infinitier_core::resource::{Engine, Game};
use infinitier_core::save_games::SaveGame;

/// Identifies one spell in the Spells tab, regardless of engine — kept on
/// [`Model`] parallel to the displayed rows so a row's delete / set-memorised
/// edit can find its spell by row index (the id passed across the boundary).
#[derive(Clone)]
pub enum SpellRefKey {
    Adnd {
        spell_type: SpellType,
        resref: String,
    },
    Iwd2 {
        book: Iwd2Spellbook,
        level: u16,
        index: u32,
    },
}

/// Sentinel for "inner spell tab not yet chosen" → auto-pick the first
/// non-empty category on the next build.
pub const SPELL_TAB_AUTO: usize = usize::MAX;

/// One entry in the Item/Spell Browser index: a game resource the browser can
/// add to the selected creature.
pub struct BrowserEntry {
    pub resref: String,
    pub name: String,
    /// Item type ("Sword") or, for spells, the spell-type label.
    pub category: String,
}

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

/// App-wide mutable state, shared across the Slint callbacks.
pub struct Model {
    pub game_data: GameData,
    pub engine_caps: EngineCaps,
    pub gam: Box<ImportedGam>,
    pub save_game: SaveGame,
    /// Selected party-member row, or `None` when the party is empty.
    pub selected: Option<usize>,
    /// The active per-character sub-tab.
    pub selected_tab: CharacterTab,
    /// Inventory tab's selected slot (the Item Browser's assignment target).
    /// Transient UI state; cleared when the party selection changes.
    pub inventory_selected: Option<usize>,
    /// Spells tab: selected inner tab index ([`SPELL_TAB_AUTO`] = auto-pick).
    pub spell_tab: usize,
    /// Spells tab: the SpellRef for each displayed row, in display order —
    /// rebuilt each Spells refresh; indexed by the row id on delete/memorise.
    pub spell_refs: Vec<SpellRefKey>,
    /// Item Browser overlay: open flag, lazy-built item index, search query,
    /// and selected item resref.
    pub item_browser_open: bool,
    pub item_index: Vec<BrowserEntry>,
    pub item_query: String,
    pub item_selected: Option<String>,
    /// Spell Browser overlay: same shape as the item browser.
    pub spell_browser_open: bool,
    pub spell_index: Vec<BrowserEntry>,
    pub spell_query: String,
    pub spell_selected: Option<String>,
    /// Other open saves, "parked" while inactive. `parked[active_tab]` is
    /// always `None` — that save is the one hydrated into the flat active
    /// fields above; every other slot holds `Some`. This keeps all the tab
    /// modules operating on the flat active fields, unchanged.
    pub parked: Vec<Option<SaveState>>,
    /// Index of the active save in the tab strip (`parked[active_tab]` is None).
    pub active_tab: usize,
    /// Load dialog overlay: open flag, search query, selected save index.
    pub load_open: bool,
    pub load_query: String,
    pub load_selected: Option<usize>,
}

/// The per-save state parked when a save isn't the active tab (mirrors the
/// flat active fields on [`Model`]).
pub struct SaveState {
    pub save_game: SaveGame,
    pub gam: Box<ImportedGam>,
    pub selected: Option<usize>,
    pub selected_tab: CharacterTab,
    pub inventory_selected: Option<usize>,
    pub spell_tab: usize,
    pub spell_refs: Vec<SpellRefKey>,
}

impl Model {
    /// Tabs visible for the loaded game, in display order.
    pub fn visible_tabs(&self) -> Vec<CharacterTab> {
        let game = self.game_data.game();
        CharacterTab::ALL
            .iter()
            .copied()
            .filter(|t| t.is_visible_for_game(game))
            .collect()
    }

    /// The selected party slot's GAM record, if any.
    pub fn selected_npc(&self) -> Option<&ImportedGamNpc> {
        self.selected.and_then(|i| self.gam.party_npcs.get(i))
    }

    /// The selected party member's embedded [`ImportedCre`] wrapper, if any
    /// (needed for resolvers that live on `ImportedCre`, e.g. inventory).
    pub fn selected_imported_cre(&self) -> Option<&ImportedCre> {
        match self.selected_npc()?.cre.as_ref()? {
            NpcCre::Cre(c) => Some(c),
            NpcCre::Ref(_) => None,
        }
    }

    /// Shared borrow of the selected party member's embedded CRE, if any.
    pub fn selected_cre(&self) -> Option<&Cre> {
        let idx = self.selected?;
        match self.gam.party_npcs.get(idx)?.cre.as_ref()? {
            NpcCre::Cre(c) => Some(c.cre()),
            NpcCre::Ref(_) => None,
        }
    }

    /// Mutable borrow of the selected party member's embedded CRE, if any.
    pub fn selected_cre_mut(&mut self) -> Option<&mut Cre> {
        let idx = self.selected?;
        match self.gam.party_npcs.get_mut(idx)?.cre.as_mut()? {
            NpcCre::Cre(c) => Some(c.cre_mut()),
            NpcCre::Ref(_) => None,
        }
    }

    pub fn character_name(&self) -> String {
        self.selected
            .and_then(|idx| self.gam.party_npcs.get(idx))
            .map(|n| n.display_name.clone())
            .unwrap_or_default()
    }

    // ── Multi-save tab management ─────────────────────────────────────

    /// The folder name of each open save, in tab-strip order.
    pub fn save_tab_names(&self) -> Vec<String> {
        self.parked
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if i == self.active_tab {
                    self.save_game.folder_name().to_string()
                } else {
                    p.as_ref()
                        .map(|s| s.save_game.folder_name().to_string())
                        .unwrap_or_default()
                }
            })
            .collect()
    }

    /// Switch the active save to tab `i` by swapping the flat active fields
    /// with `parked[i]` (parking the current save at its own slot).
    pub fn switch_save(&mut self, i: usize) {
        if i == self.active_tab || i >= self.parked.len() {
            return;
        }
        let Some(incoming) = self.parked[i].take() else {
            return;
        };
        let outgoing = SaveState {
            save_game: std::mem::replace(&mut self.save_game, incoming.save_game),
            gam: std::mem::replace(&mut self.gam, incoming.gam),
            selected: std::mem::replace(&mut self.selected, incoming.selected),
            selected_tab: std::mem::replace(&mut self.selected_tab, incoming.selected_tab),
            inventory_selected: std::mem::replace(
                &mut self.inventory_selected,
                incoming.inventory_selected,
            ),
            spell_tab: std::mem::replace(&mut self.spell_tab, incoming.spell_tab),
            spell_refs: std::mem::replace(&mut self.spell_refs, incoming.spell_refs),
        };
        self.parked[self.active_tab] = Some(outgoing);
        self.active_tab = i;
    }

    /// Open a freshly-loaded save into a new tab and make it active.
    pub fn open_save(&mut self, save_game: SaveGame, gam: Box<ImportedGam>) {
        let selected = if gam.party_npcs.is_empty() {
            None
        } else {
            Some(0)
        };
        let new_state = SaveState {
            save_game,
            gam,
            selected,
            selected_tab: CharacterTab::Abilities,
            inventory_selected: None,
            spell_tab: SPELL_TAB_AUTO,
            spell_refs: Vec::new(),
        };
        let idx = self.parked.len();
        self.parked.push(Some(new_state));
        self.switch_save(idx);
    }

    /// Close the save at tab `i`. Keeps at least one save open (no-op on the
    /// last). Returns whether a tab was closed.
    pub fn close_save(&mut self, i: usize) -> bool {
        if i >= self.parked.len() || self.parked.len() == 1 {
            return false;
        }
        if i == self.active_tab {
            let neighbor = if i == 0 { 1 } else { i - 1 };
            self.switch_save(neighbor);
        }
        self.parked.remove(i);
        if self.active_tab > i {
            self.active_tab -= 1;
        }
        true
    }
}
