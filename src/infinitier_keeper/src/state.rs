//! App-wide mutable state, separated from the egui scaffolding so
//! the UI panels can borrow it without coupling to the eframe shell.
//!
//! Shape:
//! - [`AppState`] owns the *game-wide* context (the resolved
//!   [`GameData`] index and the engine-derived caps + 2DA bonus
//!   tables) plus a `Vec<SaveTab>` of opened save games and the
//!   index of the active tab.
//! - [`SaveTab`] holds the *per-save* state — the on-disk folder
//!   path, the loaded GAM (with edits-in-flight), the currently
//!   selected party slot, and which character sub-tab is showing.
//!
//! All UI panels still address a single save at a time; instead of
//! reading `state.save` they now read `state.active().save`. The
//! borrow rules behave the same — the split exists so future code
//! can open additional saves into new tabs without restarting the
//! keeper.

use std::io;
use std::path::PathBuf;

use infinitier_core::engine_caps::EngineCaps;
use infinitier_core::game::{GameData, GameDataBuilder};
use infinitier_core::imported_resource::gam::ImportedGam;
use infinitier_core::resource::Game;
use infinitier_core::save_games::SaveGame;
use thiserror::Error;

use crate::ui::CharacterTab;

/// Top-level keeper state.
pub struct AppState {
    /// Pre-indexed game data — the FS, `chitin.key` index, and every
    /// resource the engine reaches. Shared across every open save in
    /// this keeper instance because they all came from the same
    /// game install.
    pub game_data: GameData,
    /// Cap ranges + bonus tables for the active engine. Built with
    /// the game data via [`EngineCaps::new`], which reads `STRMOD.2DA`
    /// / `STRMODEX.2DA` / `DEXMOD.2DA` / `HPCONBON.2DA` so modded
    /// installs get their actual numbers instead of the vanilla
    /// distribution.
    pub engine_caps: EngineCaps,
    /// One entry per opened save, added by the Load button.
    pub tabs: Vec<SaveTab>,
    /// Index into [`Self::tabs`] of the tab whose content the rest
    /// of the UI is showing. Always in range — the constructors and
    /// any tab-mutation path keep this invariant.
    pub active_tab: usize,
}

/// Per-save state for one open save game.
pub struct SaveTab {
    /// Reference to the save game on disk.
    pub save_game: SaveGame,
    /// Loaded save state, with every embedded CRE parsed and every
    /// NPC name resolved through `dialog.tlk` when one was
    /// reachable.
    pub save: Box<ImportedGam>,
    /// Selected party-member row, or `None` until the user clicks
    /// one.
    pub selected_party_index: Option<usize>,
    /// Currently-active sub-tab in the per-character editor on the
    /// right.
    pub selected_tab: CharacterTab,
}

/// Why [`AppState::load`] couldn't open a game install.
#[derive(Debug, Error)]
pub enum GameLoadError {
    #[error("couldn't load the game data: {0}")]
    GameData(#[source] io::Error),
    #[error("couldn't read the engine tables: {0}")]
    EngineCaps(#[source] io::Error),
}

/// Window title while no game is loaded; [`AppState::window_title`]
/// appends the game.
pub fn app_title() -> String {
    format!("Infinitier Keeper {}", crate::build_version())
}

impl AppState {
    /// Index the game install spread over `roots` (the install first,
    /// then overlays such as the save folder) and derive its engine
    /// caps, with no save open yet. `game_type: None` autodetects the
    /// game from the install.
    pub fn load(roots: &[PathBuf], game_type: Option<Game>) -> Result<Self, GameLoadError> {
        let game_data = GameDataBuilder::new(roots, game_type)
            .and_then(|b| b.build())
            .map_err(GameLoadError::GameData)?;
        let engine_caps = EngineCaps::new(&game_data).map_err(GameLoadError::EngineCaps)?;
        Ok(Self::new(game_data, engine_caps))
    }

    /// Window title for the keeper. Carries the game type only —
    /// per-save data (save name) lives in the tab strip below the
    /// header.
    pub fn window_title(&self) -> String {
        format!("{} - {:?}", app_title(), self.game_data.game())
    }

    fn new(game_data: GameData, engine_caps: EngineCaps) -> Self {
        Self {
            game_data,
            engine_caps,
            tabs: vec![],
            active_tab: 0,
        }
    }

    /// Shared borrow of the active save's per-tab state.
    pub fn active(&self) -> &SaveTab {
        &self.tabs[self.active_tab]
    }

    /// Mutable borrow of the active save's per-tab state. Holding
    /// this mutably locks the entire [`AppState`], so call sites
    /// that also need `&self.engine_caps` should split-borrow over
    /// `AppState`'s fields (`let AppState { engine_caps, tabs,
    /// active_tab, .. } = &mut *state;`) instead.
    pub fn active_mut(&mut self) -> &mut SaveTab {
        &mut self.tabs[self.active_tab]
    }
}

impl SaveTab {
    pub fn new(save_game: SaveGame, save: Box<ImportedGam>) -> Self {
        let selected_party_index = if save.party_npcs.is_empty() {
            None
        } else {
            Some(0)
        };
        Self {
            save_game,
            save,
            selected_party_index,
            selected_tab: CharacterTab::Abilities,
        }
    }
}
