//! Every raster image type (BMP, PVRZ, MOS, PNG…), fitted to the area.

use infinitier_core::{game::GameResource, imported_resource::image::ImportedImage};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

use super::info::{origin_label, size_label};

/// An RGBA8 (not premultiplied) buffer as a Slint image.
pub fn rgba_image(pixels: &[u8], width: u32, height: u32) -> Image {
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        pixels, width, height,
    ))
}

pub struct ImageViewer {
    pub image: Image,
    pub width: u32,
    pub height: u32,
    /// Short uppercase label of the source format (e.g. `"BMP"`).
    format_label: &'static str,
    /// Bit depth / compression / DXT variant.
    format_description: String,
}

impl ImageViewer {
    pub fn new(img: ImportedImage) -> Self {
        let (width, height) = (img.width(), img.height());
        Self {
            image: rgba_image(img.image.as_raw(), width, height),
            width,
            height,
            format_label: img.format_label(),
            format_description: img.format_description(),
        }
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        vec![
            format!("{} × {} px", self.width, self.height),
            size_label(resource),
            self.format_label.to_owned(),
            self.format_description.clone(),
            origin_label(resource),
        ]
    }
}
