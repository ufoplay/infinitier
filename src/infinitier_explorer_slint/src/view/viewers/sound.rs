use slint::ComponentHandle;

use crate::viewers::sound::SoundViewer;
use crate::{AppWindow, SoundAdapter};

pub fn refresh(ui: &AppWindow, viewer: &SoundViewer) {
    let adapter = ui.global::<SoundAdapter>();
    adapter.set_device_missing(viewer.device_missing());
    adapter.set_progress(viewer.progress());
    adapter.set_time(viewer.time_text().into());
    adapter.set_playing(viewer.is_playing());
    adapter.set_can_play(viewer.can_play());
}
