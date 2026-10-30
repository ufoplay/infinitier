//! The explorer's state: the loaded game's resources, the tree, the viewer
//! of the selected resource, and the Settings dialog.

use std::rc::Rc;

use infinitier_core::game::{GameData, GameResource, ResourceId};
use infinitier_core::resource::Game;
use infinitier_gui_common::{GameConfig, SettingsDialog};
use slint::Timer;

use crate::fonts::FontRegistry;
use crate::loading::PendingGame;
use crate::tree::{ResourceTree, TreeModel};
use crate::viewers::{self, Viewer};

/// The TTF viewer's sample sentence, until the user edits it.
const DEFAULT_SAMPLE: &str = "The quick brown fox jumps over the lazy dog. 0123456789";

/// A loaded game: its resources, and the config they were read from.
pub struct LoadedGame {
    pub config: GameConfig,
    pub data: GameData,
}

pub struct App {
    /// `None` until a game is loaded.
    pub game: Option<LoadedGame>,
    /// The left panel's rows, shared with the Slint list that shows them.
    pub tree: Rc<TreeModel>,
    pub selected: Option<ResourceId>,
    pub viewer: Viewer,
    pub fonts: FontRegistry,
    /// The TTF viewer's sample sentence, kept from one font to the next.
    pub sample_text: String,
    /// The configured games, the theme, and the dialog that edits them.
    pub settings: SettingsDialog,
    /// The game being loaded, if any; its overlay blocks the UI meanwhile.
    pub pending: Option<PendingGame>,
    /// Polls [`Self::pending`] for completion while a load runs.
    pub loading_timer: Timer,
}

impl App {
    pub fn new(settings: SettingsDialog) -> Self {
        Self {
            game: None,
            tree: Rc::new(TreeModel::new(ResourceTree::default())),
            selected: None,
            viewer: Viewer::no_game(),
            fonts: FontRegistry::default(),
            sample_text: DEFAULT_SAMPLE.to_owned(),
            settings,
            pending: None,
            loading_timer: Timer::default(),
        }
    }

    /// Show `game` in place of the loaded game: its own tree, with nothing
    /// selected.
    pub fn set_game(&mut self, game: LoadedGame) {
        // Release the old viewer (and stop its playback threads) first.
        self.viewer = Viewer::nothing_selected();
        self.selected = None;
        // The fonts are registered by resource id, which is per game.
        self.fonts = FontRegistry::default();
        self.tree.replace(ResourceTree::new(&game.data));
        self.game = Some(game);
    }

    /// Show resource `id`, unless it is already shown.
    pub fn open(&mut self, id: ResourceId) {
        let Some(game) = &self.game else {
            return;
        };
        if self.selected == Some(id) {
            return;
        }
        self.selected = Some(id);
        // Release the old viewer (and stop its playback threads) first.
        self.viewer = Viewer::nothing_selected();
        self.viewer = viewers::open(&game.data, id, &mut self.fonts);
    }

    /// The loaded game's type.
    pub fn game_type(&self) -> Option<Game> {
        self.game.as_ref().map(|game| game.data.game())
    }

    /// The resource shown by the viewer.
    pub fn selected_resource(&self) -> Option<&GameResource> {
        self.game.as_ref()?.data.get_by_id(self.selected?)
    }

    /// The status line: the selected resource and where it comes from.
    pub fn status(&self) -> String {
        match self.selected_resource() {
            Some(resource) => format!(
                "Resource: {} - Source: {:?}",
                resource.name, resource.data_origin
            ),
            None => "No file selected".to_owned(),
        }
    }
}
