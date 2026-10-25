use slint::ComponentHandle;

use crate::viewers::bcs::BcsViewer;
use crate::{AppWindow, BcsAdapter};

pub fn refresh(ui: &AppWindow, viewer: &BcsViewer) {
    let adapter = ui.global::<BcsAdapter>();
    // The same model each time: the list keeps its rows and scroll position.
    adapter.set_lines(viewer.lines());
    adapter.set_columns(i32::try_from(viewer.columns()).unwrap_or(i32::MAX));
    adapter.set_source(viewer.source());
}
