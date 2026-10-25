//! The viewers' controls: players, pickers and the TTF sample.

use slint::ComponentHandle;

use super::Ctx;
use crate::viewers::Viewer;
use crate::{AppWindow, BamAdapter, MovieAdapter, SoundAdapter, TisAdapter, TtfAdapter};

pub(super) fn wire(ui: &AppWindow, ctx: &Ctx) {
    wire_bam(ui, ctx);
    wire_tis(ui, ctx);
    wire_ttf(ui, ctx);
    wire_players(ui, ctx);
}

/// A callback that edits the BAM viewer, if it is the one shown.
macro_rules! on_viewer {
    ($ctx:expr, $variant:ident, |$viewer:ident $(, $arg:ident)*| $body:expr) => {{
        let ctx = $ctx.clone();
        move |$($arg),*| {
            ctx.update(|app| {
                if let Viewer::$variant($viewer) = &mut app.viewer {
                    $body;
                }
            })
        }
    }};
}

fn wire_bam(ui: &AppWindow, ctx: &Ctx) {
    let adapter = ui.global::<BamAdapter>();
    adapter.on_play_toggled(on_viewer!(ctx, Bam, |viewer| viewer.toggle_play()));
    adapter.on_cycle_selected(on_viewer!(ctx, Bam, |viewer, cycle| {
        if let Ok(cycle) = usize::try_from(cycle) {
            viewer.select_cycle(cycle);
        }
    }));
    adapter.on_frame_changed(on_viewer!(ctx, Bam, |viewer, frame| {
        if let Ok(frame) = usize::try_from(frame) {
            viewer.select_frame(frame);
        }
    }));
}

fn wire_tis(ui: &AppWindow, ctx: &Ctx) {
    let adapter = ui.global::<TisAdapter>();
    adapter.on_columns_changed(on_viewer!(ctx, Tis, |viewer, columns| {
        if let Ok(columns) = u32::try_from(columns) {
            viewer.set_columns(columns);
        }
    }));
    adapter.on_grid_toggled(on_viewer!(ctx, Tis, |viewer| viewer.toggle_grid()));
}

fn wire_ttf(ui: &AppWindow, ctx: &Ctx) {
    let ctx = ctx.clone();
    ui.global::<TtfAdapter>()
        .on_sample_edited(move |text| ctx.update(|app| app.sample_text = text.into()));
}

fn wire_players(ui: &AppWindow, ctx: &Ctx) {
    let sound = ui.global::<SoundAdapter>();
    sound.on_play_toggled(on_viewer!(ctx, Sound, |viewer| viewer.toggle_play_pause()));
    sound.on_stop(on_viewer!(ctx, Sound, |viewer| viewer.stop_playback()));
    let movie = ui.global::<MovieAdapter>();
    movie.on_play_toggled(on_viewer!(ctx, Movie, |viewer| viewer.toggle_play_pause()));
    movie.on_stop(on_viewer!(ctx, Movie, |viewer| viewer.stop_playback()));
}
