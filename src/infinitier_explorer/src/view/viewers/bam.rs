use slint::ComponentHandle;

use crate::view::strings_model;
use crate::viewers::bam::BamViewer;
use crate::{AppWindow, BamAdapter};

pub fn refresh(ui: &AppWindow, viewer: &BamViewer) {
    let adapter = ui.global::<BamAdapter>();
    match viewer.frame() {
        Some(frame) => {
            adapter.set_image(frame.image.clone());
            adapter.set_width(frame.width as f32);
            adapter.set_height(frame.height as f32);
        }
        None => adapter.set_image(slint::Image::default()),
    }
    adapter.set_has_frame(viewer.current_global_frame_index().is_some());
    adapter.set_playing(viewer.is_playing());
    adapter.set_can_play(viewer.can_play());
    adapter.set_cycles(strings_model(viewer.cycle_labels()));
    adapter.set_cycle(index_or_none(viewer.selected_cycle()));

    let frames = viewer.frames_in_selected_cycle();
    let frame = viewer.selected_frame();
    let frame_max = frames.saturating_sub(1);
    adapter.set_frame(to_i32(frame));
    adapter.set_frame_max(to_i32(frame_max));
    adapter.set_frame_text(
        if frames == 0 {
            "—".to_owned()
        } else {
            format!("{frame} / {frame_max}")
        }
        .into(),
    );
    adapter.set_frame_index_text(
        viewer
            .current_global_frame_index()
            .map(|index| format!("(frame #{index})"))
            .unwrap_or_default()
            .into(),
    );
}

fn to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// A Slint index for `index`, `-1` for none.
fn index_or_none(index: Option<usize>) -> i32 {
    index.map(to_i32).unwrap_or(-1)
}
