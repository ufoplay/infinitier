//! Decoding a game's BAM icons into Slint images.

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::ImportedResource;
use infinitier_core::resource::ResourceType;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

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
    let raw = frame.into_raw();
    let mut pb = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    pb.make_mut_bytes().copy_from_slice(&raw);
    Some(Image::from_rgba8(pb))
}
