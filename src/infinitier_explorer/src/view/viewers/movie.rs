use slint::ComponentHandle;

use crate::viewers::movie::MovieViewer;
use crate::{AppWindow, MovieAdapter};

pub fn refresh(ui: &AppWindow, viewer: &MovieViewer) {
    let adapter = ui.global::<MovieAdapter>();
    let (width, height) = viewer.size();
    adapter.set_frame(viewer.frame().cloned().unwrap_or_default());
    adapter.set_has_frame(viewer.frame().is_some());
    adapter.set_width(f32::from(width));
    adapter.set_height(f32::from(height));
    adapter.set_size_text(format!("{width}×{height}").into());
    adapter.set_error(viewer.decode_error().unwrap_or_default().into());
    adapter.set_progress(viewer.progress());
    adapter.set_time(viewer.time_text().into());
    adapter.set_playing(viewer.playing());
    adapter.set_can_play(viewer.can_play());
}
