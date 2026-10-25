use slint::ComponentHandle;

use crate::viewers::image::ImageViewer;
use crate::{AppWindow, ImageAdapter};

pub fn refresh(ui: &AppWindow, viewer: &ImageViewer) {
    let adapter = ui.global::<ImageAdapter>();
    adapter.set_image(viewer.image.clone());
    adapter.set_width(viewer.width as f32);
    adapter.set_height(viewer.height as f32);
}
