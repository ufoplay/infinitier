//! Fonts found or registered at run time, through the fontique collection
//! Slint renders text with.

use std::collections::HashMap;
use std::sync::Arc;

use infinitier_core::game::ResourceId;
use slint::fontique_011::{fontique, shared_collection};

/// The family name of the system's monospace font, for the code views
/// (Slint takes a font family by name, and "monospace" isn't one). `None`
/// if the system has none.
pub fn monospace_family() -> Option<String> {
    let mut collection = shared_collection();
    let family = collection
        .generic_families(fontique::GenericFamily::Monospace)
        .next()?;
    collection.family_name(family).map(str::to_owned)
}

/// The TTF resources' fonts, registered once each, on their first view: the
/// family name each one was registered under (`None` if it couldn't be).
#[derive(Default)]
pub struct FontRegistry {
    families: HashMap<ResourceId, Option<String>>,
}

impl FontRegistry {
    pub fn family(&mut self, id: ResourceId, font: &Arc<Vec<u8>>) -> Option<String> {
        self.families
            .entry(id)
            .or_insert_with(|| register(font))
            .clone()
    }
}

/// Register `font` for the whole process: Slint finds it by the family name
/// returned.
fn register(font: &Arc<Vec<u8>>) -> Option<String> {
    let mut collection = shared_collection();
    let fonts = collection.register_fonts(fontique::Blob::new(font.clone()), None);
    let (family, _) = fonts.first()?;
    collection.family_name(*family).map(str::to_owned)
}
