//! The explorer's state: the game's resources, the tree, and the viewer of
//! the selected resource.

use std::rc::Rc;

use infinitier_core::game::{GameData, ResourceId};

use crate::fonts::FontRegistry;
use crate::tree::{ResourceTree, TreeModel};
use crate::viewers::{self, Viewer};

/// The TTF viewer's sample sentence, until the user edits it.
const DEFAULT_SAMPLE: &str = "The quick brown fox jumps over the lazy dog. 0123456789";

pub struct App {
    pub game_data: GameData,
    /// The left panel's rows, shared with the Slint list that shows them.
    pub tree: Rc<TreeModel>,
    pub selected: Option<ResourceId>,
    pub viewer: Viewer,
    pub dark_theme: bool,
    pub fonts: FontRegistry,
    /// The TTF viewer's sample sentence, kept from one font to the next.
    pub sample_text: String,
}

impl App {
    pub fn new(game_data: GameData) -> Self {
        Self {
            tree: Rc::new(TreeModel::new(ResourceTree::new(&game_data))),
            game_data,
            selected: None,
            viewer: Viewer::nothing_selected(),
            dark_theme: false,
            fonts: FontRegistry::default(),
            sample_text: DEFAULT_SAMPLE.to_owned(),
        }
    }

    /// Show resource `id`, unless it is already shown.
    pub fn open(&mut self, id: ResourceId) {
        if self.selected == Some(id) {
            return;
        }
        self.selected = Some(id);
        // Release the old viewer (and stop its playback threads) first.
        self.viewer = Viewer::nothing_selected();
        self.viewer = viewers::open(&self.game_data, id, &mut self.fonts);
    }

    /// The status line: the selected resource and where it comes from.
    pub fn status(&self) -> String {
        match self.selected.and_then(|id| self.game_data.get_by_id(id)) {
            Some(resource) => format!(
                "Resource: {} - Source: {:?}",
                resource.name, resource.data_origin
            ),
            None => "No file selected".to_owned(),
        }
    }
}
