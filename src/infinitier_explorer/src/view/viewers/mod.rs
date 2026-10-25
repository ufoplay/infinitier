//! The selected resource's viewer: which one, its info bar, and its own
//! adapter.

mod bam;
mod bcs;
mod fnt;
mod image;
mod movie;
mod sound;
mod tis;
mod ttf;

use slint::ComponentHandle;

use super::strings_model;
use crate::app::App;
use crate::viewers::Viewer;
use crate::{AppWindow, ViewerAdapter, ViewerKind};

pub fn refresh(ui: &AppWindow, app: &App) {
    let adapter = ui.global::<ViewerAdapter>();
    let info = app
        .selected
        .and_then(|id| app.game_data.get_by_id(id))
        .map(|resource| app.viewer.info(resource))
        .unwrap_or_default();
    adapter.set_info(strings_model(info));
    let kind = match &app.viewer {
        Viewer::Message { text, error } => {
            adapter.set_message(text.as_str().into());
            adapter.set_error(*error);
            ViewerKind::Message
        }
        Viewer::Image(viewer) => {
            image::refresh(ui, viewer);
            ViewerKind::Image
        }
        Viewer::Bam(viewer) => {
            bam::refresh(ui, viewer);
            ViewerKind::Bam
        }
        Viewer::Tis(viewer) => {
            tis::refresh(ui, viewer);
            ViewerKind::Tis
        }
        Viewer::Fnt(viewer) => {
            fnt::refresh(ui, viewer);
            ViewerKind::Fnt
        }
        Viewer::Ttf(viewer) => {
            ttf::refresh(ui, viewer, &app.sample_text);
            ViewerKind::Ttf
        }
        Viewer::Bcs(viewer) => {
            bcs::refresh(ui, viewer);
            ViewerKind::Bcs
        }
        Viewer::Sound(viewer) => {
            sound::refresh(ui, viewer);
            ViewerKind::Sound
        }
        Viewer::Movie(viewer) => {
            movie::refresh(ui, viewer);
            ViewerKind::Movie
        }
    };
    adapter.set_kind(kind);
}
