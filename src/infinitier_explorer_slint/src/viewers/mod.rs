//! The viewers' state and logic, one module per kind of resource (like the
//! egui explorer's `components/resource_viewer/`). [`open`] imports the
//! selected resource and picks its viewer; the `view` module pushes a
//! viewer's state into its Slint adapter.

mod audio;
mod baf_highlight;
pub mod bam;
pub mod bcs;
pub mod fnt;
pub mod image;
mod info;
pub mod movie;
pub mod sound;
pub mod tis;
pub mod ttf;

use std::time::Duration;

use infinitier_core::{
    game::{GameData, GameResource, ResourceId},
    imported_resource::ImportedResource,
    resource::ResourceType,
};
use log::error;

use crate::fonts::FontRegistry;
use bam::BamViewer;
use bcs::BcsViewer;
use fnt::FntViewer;
use image::ImageViewer;
use movie::MovieViewer;
use sound::SoundViewer;
use tis::TisViewer;
use ttf::TtfViewer;

/// What the selected resource is shown with.
pub enum Viewer {
    /// A line of text: nothing selected, a format not viewed yet, an
    /// unknown type, or an error.
    Message {
        text: String,
        error: bool,
    },
    Image(ImageViewer),
    Bam(BamViewer),
    Tis(TisViewer),
    Fnt(FntViewer),
    Ttf(TtfViewer),
    Bcs(BcsViewer),
    Sound(SoundViewer),
    Movie(MovieViewer),
}

impl Viewer {
    pub fn nothing_selected() -> Self {
        Self::message("Select a resource from the panel on the left.")
    }

    fn message(text: impl Into<String>) -> Self {
        Self::Message {
            text: text.into(),
            error: false,
        }
    }

    fn error(text: String) -> Self {
        error!("{text}");
        Self::Message { text, error: true }
    }

    /// The facts of the info bar under the viewer; none for a message.
    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        match self {
            Viewer::Message { .. } => Vec::new(),
            Viewer::Image(viewer) => viewer.info(resource),
            Viewer::Bam(viewer) => viewer.info(resource),
            Viewer::Tis(viewer) => viewer.info(resource),
            Viewer::Fnt(viewer) => viewer.info(resource),
            Viewer::Ttf(viewer) => viewer.info(resource),
            Viewer::Bcs(viewer) => viewer.info(resource),
            Viewer::Sound(viewer) => viewer.info(),
            Viewer::Movie(viewer) => viewer.info(),
        }
    }

    /// How often the viewer wants [`tick`](Self::tick) to run: `Some` while
    /// it plays something.
    pub fn tick_interval(&self) -> Option<Duration> {
        match self {
            Viewer::Bam(viewer) => viewer.tick_interval(),
            Viewer::Sound(viewer) => viewer.tick_interval(),
            Viewer::Movie(viewer) => viewer.tick_interval(),
            Viewer::Message { .. }
            | Viewer::Image(_)
            | Viewer::Tis(_)
            | Viewer::Fnt(_)
            | Viewer::Ttf(_)
            | Viewer::Bcs(_) => None,
        }
    }

    /// Advance playback.
    pub fn tick(&mut self) {
        match self {
            Viewer::Bam(viewer) => viewer.tick(),
            Viewer::Sound(viewer) => viewer.tick(),
            Viewer::Movie(viewer) => viewer.tick(),
            Viewer::Message { .. }
            | Viewer::Image(_)
            | Viewer::Tis(_)
            | Viewer::Fnt(_)
            | Viewer::Ttf(_)
            | Viewer::Bcs(_) => {}
        }
    }
}

/// Import resource `id` and pick the viewer it is shown with.
pub fn open(game_data: &GameData, id: ResourceId, fonts: &mut FontRegistry) -> Viewer {
    let Some(resource) = game_data.get_by_id(id) else {
        return Viewer::error(format!("Resource not found: {id:?}"));
    };
    let imported = match resource.import(game_data) {
        Ok(imported) => imported.into_owned(),
        Err(err) => return Viewer::error(format!("Error importing resource: {id:?}, {err:?}")),
    };
    match imported {
        ImportedResource::Bam(bam) => Viewer::Bam(BamViewer::new(bam)),
        ImportedResource::Image(img) => Viewer::Image(ImageViewer::new(img)),
        ImportedResource::Tis(tis) => Viewer::Tis(TisViewer::new(tis)),
        ImportedResource::Fnt(fnt) => Viewer::Fnt(FntViewer::new(fnt)),
        ImportedResource::Ttf(ttf) => {
            let family = fonts.family(id, &ttf.raw);
            Viewer::Ttf(TtfViewer::new(ttf, family))
        }
        ImportedResource::Bcs(bcs) => Viewer::Bcs(BcsViewer::new(bcs)),
        ImportedResource::Sound(decoder) => Viewer::Sound(SoundViewer::new(decoder)),
        ImportedResource::Mve(source) | ImportedResource::Wbm(source) => {
            Viewer::Movie(MovieViewer::new(source))
        }
        ImportedResource::Ids(_) => Viewer::message("IDS Viewer"),
        ImportedResource::Ini(_) => Viewer::message("INI Viewer"),
        ImportedResource::TwoDA(_) => Viewer::message("2DA Viewer"),
        ImportedResource::Wed(_) => Viewer::message("WED Viewer"),
        ImportedResource::Are => Viewer::message("ARE Viewer"),
        ImportedResource::Bah => Viewer::message("BAH Viewer"),
        ImportedResource::Bio => Viewer::message("BIO Viewer"),
        ImportedResource::Chr => Viewer::message("CHR Viewer"),
        ImportedResource::Chu => Viewer::message("CHU Viewer"),
        ImportedResource::Cre(_) => Viewer::message("CRE Viewer"),
        ImportedResource::Dlg => Viewer::message("DLG Viewer"),
        ImportedResource::Eff => Viewer::message("EFF Viewer"),
        ImportedResource::Gam(_) => Viewer::message("GAM Viewer"),
        ImportedResource::Glsl => Viewer::message("GLSL Viewer"),
        ImportedResource::Gui => Viewer::message("GUI Viewer"),
        ImportedResource::Itm(_) => Viewer::message("ITM Viewer"),
        ImportedResource::Lua => Viewer::message("LUA Viewer"),
        ImportedResource::Maze => Viewer::message("MAZE Viewer"),
        ImportedResource::Menu => Viewer::message("MENU Viewer"),
        ImportedResource::Mus => Viewer::message("MUS Viewer"),
        ImportedResource::Plt => Viewer::message("PLT Viewer"),
        ImportedResource::Pro => Viewer::message("PRO Viewer"),
        ImportedResource::Spl(_) => Viewer::message("SPL Viewer"),
        ImportedResource::Sql => Viewer::message("SQL Viewer"),
        ImportedResource::Src => Viewer::message("SRC Viewer"),
        ImportedResource::Sto => Viewer::message("STO Viewer"),
        ImportedResource::Tga => Viewer::message("TGA Viewer"),
        ImportedResource::Toh => Viewer::message("TOH Viewer"),
        ImportedResource::Tot => Viewer::message("TOT Viewer"),
        ImportedResource::Vef => Viewer::message("VEF Viewer"),
        ImportedResource::Vvc => Viewer::message("VVC Viewer"),
        ImportedResource::Wfx => Viewer::message("WFX Viewer"),
        ImportedResource::Wmp => Viewer::message("WMP Viewer"),
        // Sav / Tlk have no `ResourceType` entry yet, so the import
        // dispatcher never produces them here: treat them as unknown.
        ImportedResource::Sav(_) | ImportedResource::Tlk(_) | ImportedResource::Unknown(_) => {
            let type_id = if let ResourceType::Unknown(id) = resource.r#type {
                id
            } else {
                0
            };
            Viewer::message(format!("Unknown Viewer (type: {type_id:#06x})"))
        }
    }
}
