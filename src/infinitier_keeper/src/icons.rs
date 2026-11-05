//! Decoding a game's images (BAM icons, BMP portraits) into Slint images.

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::ImportedResource;
use infinitier_core::resource::ResourceType;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

/// Wrap `width × height` RGBA bytes in a Slint `Image`.
pub fn rgba_image(width: u32, height: u32, rgba: &[u8]) -> Image {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    buffer.make_mut_bytes().copy_from_slice(rgba);
    Image::from_rgba8(buffer)
}

/// Decode the first frame of the BAM `resref` (an inventory or spellbook
/// icon) into a Slint `Image`. `None` when it's unset, missing or empty.
pub fn bam_icon(game_data: &GameData, resref: &str) -> Option<Image> {
    if resref.is_empty() {
        return None;
    }
    let imported = game_data
        .import_by_name_and_type(resref, ResourceType::Bam)
        .ok()?;
    let ImportedResource::Bam(bam) = imported.as_ref() else {
        return None;
    };
    let frame = bam.render_frame_centered(0, 0)?;
    let (w, h) = (frame.width(), frame.height());
    if w == 0 || h == 0 {
        return None;
    }
    Some(rgba_image(w, h, &frame.into_raw()))
}

/// Decode the BMP `resref` (a portrait) into a Slint `Image`. `None` when
/// it's missing or empty.
pub fn bmp_image(game_data: &GameData, resref: &str) -> Option<Image> {
    let imported = game_data
        .import_by_name_and_type(resref, ResourceType::Bmp)
        .ok()?;
    let ImportedResource::Image(img) = imported.as_ref() else {
        return None;
    };
    let (w, h) = (img.image.width(), img.image.height());
    if w == 0 || h == 0 {
        return None;
    }
    Some(rgba_image(w, h, img.image.as_raw()))
}
