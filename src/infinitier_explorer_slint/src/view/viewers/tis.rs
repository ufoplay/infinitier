use slint::ComponentHandle;

use crate::viewers::tis::TisViewer;
use crate::{AppWindow, TisAdapter};

pub fn refresh(ui: &AppWindow, viewer: &TisViewer) {
    let adapter = ui.global::<TisAdapter>();
    let (width, height) = viewer.image_size();
    let tile = viewer.tile_dimension().max(1);
    adapter.set_image(viewer.image().clone());
    adapter.set_width(width as f32);
    adapter.set_height(height as f32);
    adapter.set_tile_size(tile as f32);
    adapter.set_grid_columns(to_i32(width / tile));
    adapter.set_grid_rows(to_i32(height / tile));
    adapter.set_columns(to_i32(viewer.columns()));
    adapter.set_columns_min(to_i32(*viewer.range().start()));
    adapter.set_columns_max(to_i32(*viewer.range().end()));
    adapter.set_show_grid(viewer.show_grid());
    adapter.set_truncated(viewer.truncated());
}

fn to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
