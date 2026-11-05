//! Every raster image type (BMP, PVRZ, MOS, PNG…), and paperdolls (PLT),
//! fitted to the area.

use infinitier_core::game::GameResource;
use infinitier_core::imported_resource::image::ImportedImage;
use infinitier_core::imported_resource::plt::{ArtFormat, Equipment, ImportedPlt};
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

    /// A paperdoll, painted as [`ImportedPlt`] paints a lone one: in the
    /// colours of the game's base creature.
    pub fn from_plt(plt: &ImportedPlt) -> Self {
        let picture = plt.render(Equipment::Shown);
        let (width, height) = picture.dimensions();
        let format = plt.format();
        let format_description = match format {
            ArtFormat::Plt => "Paperdoll, in CHARBASE.CRE's colours",
            ArtFormat::Bam => "Paperdoll, a BAM under the PLT type",
            ArtFormat::Bmp => "Paperdoll, painted",
        };
        Self {
            image: rgba_image(picture.as_raw(), width, height),
            width,
            height,
            format_label: format.label(),
            format_description: format_description.to_owned(),
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

#[cfg(test)]
mod tests {
    use infinitier_core::fs::{CaseInsensitiveFS, DataSource};
    use infinitier_core::game::GameData;
    use infinitier_core::resource::Game;
    use infinitier_core::resource::plt::PLT_SIGNATURE;

    use super::*;

    #[test]
    fn a_plt_shows_at_its_size_labelled_as_a_plt() {
        let mut file = PLT_SIGNATURE.to_vec();
        file.extend([0; 8]);
        file.extend(3u32.to_le_bytes());
        file.extend(2u32.to_le_bytes());
        file.extend([0; 3 * 2 * 2]);
        let game_data = GameData::new(vec![], Game::Bg2, CaseInsensitiveFS::empty());
        let plt =
            ImportedPlt::load("test", &DataSource::new(file), &game_data).expect("a valid PLT");

        let viewer = ImageViewer::from_plt(&plt);
        assert_eq!((viewer.width, viewer.height), (3, 2));
        assert_eq!(viewer.format_label, "PLT");
        assert_eq!(viewer.image.size().width, 3);
    }
}
