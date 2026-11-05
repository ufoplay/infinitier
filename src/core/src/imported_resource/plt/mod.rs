//! [`ImportedPlt`]: a paperdoll, the figure the inventory and
//! character-generation screens draw, ready to render in a creature's
//! colours, with or without its equipment.
//!
//! This is the form stored in [`ImportedResource::Plt`](super::ImportedResource::Plt),
//! where it holds the one PLT resource. [`ImportedPlt::for_creature`] builds a
//! creature's whole doll, from whichever art its game has.

mod art;
mod creature;
mod gradients;

use std::io;

use image::RgbaImage;
use infinitier_common::ResourceType;
use infinitier_cre_resource::Colors;
use infinitier_datasource::DataSource;

use crate::game::GameData;
use crate::imported_resource::ImportedResource;
use crate::imported_resource::cre::ImportedCre;
use art::{Art, Palettes};

pub use gradients::Gradients;

/// Whether a doll shows what the creature wears and holds: its armour, its
/// weapon, its shield or off-hand weapon, its helmet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Equipment {
    #[default]
    Shown,
    /// The doll bare, as character generation shows it.
    Hidden,
}

/// What a paperdoll's art is stored as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtFormat {
    /// A PLT: each pixel a colour and a shade of it.
    Plt,
    /// A BAM, whose palette takes the colours: a paperdoll's two halves (also
    /// what the Enhanced Editions keep under the PLT type), or a sprite.
    Bam,
    /// A painted BMP, which the colours don't change.
    Bmp,
}

impl ArtFormat {
    /// The format's short name: `"PLT"`, `"BAM"`, `"BMP"`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Plt => "PLT",
            Self::Bam => "BAM",
            Self::Bmp => "BMP",
        }
    }
}

/// A paperdoll: its art, the colours to paint it in and the gradients those
/// colours select.
#[derive(Debug, Clone)]
pub struct ImportedPlt {
    /// The doll bare.
    bare: Art,
    /// In the armour it wears, when that's other art: drawn instead of the
    /// bare doll when the equipment is shown.
    armored: Option<Art>,
    /// Drawn over the doll when the equipment is shown, in this order:
    /// weapon, off hand, helmet.
    equipment: Vec<Art>,
    colors: Colors,
    palettes: Palettes,
}

impl ImportedPlt {
    /// A PLT resource on its own: a PLT, or the paperdoll BAM the Enhanced
    /// Editions keep under that type. Painted in the colours of the game's
    /// base creature, `CHARBASE.CRE` (the first gradient of each colour in a
    /// game without one), until given others with
    /// [`with_colors`](Self::with_colors).
    ///
    /// # Errors
    ///
    /// When `source` holds neither, or a BAM V2, which has no palette to
    /// paint.
    pub fn load(name: &str, source: &DataSource, game_data: &GameData) -> io::Result<Self> {
        Ok(Self {
            bare: Art::from_plt_resource(name, source)?,
            armored: None,
            equipment: Vec::new(),
            colors: base_creature_colors(game_data),
            palettes: Palettes::load(game_data),
        })
    }

    /// `cre`'s paperdoll: in its colours, its armour and its equipment. `None`
    /// when its animation has no doll: a creature that isn't humanoid, a
    /// polymorph.
    pub fn for_creature(game_data: &GameData, cre: &ImportedCre) -> Option<Self> {
        let creature::Parts {
            bare,
            armored,
            equipment,
        } = creature::art(game_data, cre)?;
        Some(Self {
            bare,
            armored,
            equipment,
            colors: cre.colors(),
            palettes: Palettes::load(game_data),
        })
    }

    /// The same doll in other colours.
    #[must_use]
    pub fn with_colors(self, colors: Colors) -> Self {
        Self { colors, ..self }
    }

    /// The colours it's painted in.
    pub fn colors(&self) -> Colors {
        self.colors
    }

    /// Whether there is equipment to show: armour, or something held or worn
    /// over it. [`Equipment`] then changes the picture.
    pub fn has_equipment(&self) -> bool {
        self.armored.is_some() || !self.equipment.is_empty()
    }

    /// What the bare doll is stored as.
    pub fn format(&self) -> ArtFormat {
        self.bare.format()
    }

    /// The doll as RGBA pixels, painted: in its armour and with what it holds
    /// and wears over it, or bare.
    pub fn render(&self, equipment: Equipment) -> RgbaImage {
        let (body, worn): (&Art, &[Art]) = match equipment {
            Equipment::Shown => (self.armored.as_ref().unwrap_or(&self.bare), &self.equipment),
            Equipment::Hidden => (&self.bare, &[]),
        };
        let (width, height) = body.size();
        let mut canvas = RgbaImage::new(width, height);
        for art in std::iter::once(body).chain(worn) {
            art.draw(&mut canvas, self.colors, &self.palettes);
        }
        canvas
    }
}

/// The colours of the game's base creature, `CHARBASE.CRE`; the first
/// gradient of each colour when the game has none.
fn base_creature_colors(game_data: &GameData) -> Colors {
    match game_data
        .import_by_name_and_type("charbase", ResourceType::Cre)
        .ok()
        .as_deref()
    {
        Some(ImportedResource::Cre(cre)) => cre.colors(),
        _ => Colors::default(),
    }
}

#[cfg(test)]
mod tests {
    use infinitier_bam_resource::BamV1Frame;
    use infinitier_bam_resource::common::Rgb;
    use infinitier_plt_resource::PLT_SIGNATURE;

    use super::*;

    fn plt_source(pixels: &[u8]) -> DataSource {
        let mut file = PLT_SIGNATURE.to_vec();
        file.extend([0; 8]);
        file.extend(2u32.to_le_bytes());
        file.extend(1u32.to_le_bytes());
        file.extend(pixels);
        DataSource::new(file)
    }

    /// A one-pixel sword at the top left of the doll's canvas.
    fn sword() -> Art {
        let mut palette = vec![
            Rgb {
                r: 7,
                g: 7,
                b: 7,
                alpha: u8::MAX,
            };
            3
        ];
        palette[0].alpha = 0;
        Art::Halves {
            frames: vec![BamV1Frame {
                width: 1,
                height: 1,
                center_x: 0,
                center_y: 0,
                pixel_palette_indexes: vec![2],
            }],
            palette,
        }
    }

    /// A two-pixel PLT: `group`, shade 0, then transparent.
    fn plt(group: u8) -> Art {
        Art::from_plt_resource("test", &plt_source(&[0, group, 255, 0])).expect("a valid PLT")
    }

    /// A two-pixel doll, skin (bare) or hair (armoured), with a sword.
    fn doll() -> ImportedPlt {
        ImportedPlt {
            bare: plt(0),
            armored: Some(plt(1)),
            equipment: vec![sword()],
            colors: Colors {
                hair: 1,
                ..Colors::default()
            },
            palettes: Palettes::new(
                Gradients::from_ramps(vec![vec![[1, 2, 3]], vec![[4, 5, 6]]]),
                Gradients::from_ramps(vec![]),
            ),
        }
    }

    #[test]
    fn shown_the_doll_is_armoured_and_holds_its_equipment() {
        let picture = doll().render(Equipment::Shown);
        assert_eq!(picture.dimensions(), (2, 1));
        // The sword covers the armour.
        assert_eq!(picture.get_pixel(0, 0).0, [7, 7, 7, 255]);
        assert_eq!(picture.get_pixel(1, 0).0, [0; 4]);
        let unarmed = ImportedPlt {
            equipment: Vec::new(),
            ..doll()
        };
        assert_eq!(
            unarmed.render(Equipment::Shown).get_pixel(0, 0).0,
            [4, 5, 6, 255]
        );
    }

    #[test]
    fn hidden_the_doll_is_bare() {
        let picture = doll().render(Equipment::Hidden);
        assert_eq!(picture.get_pixel(0, 0).0, [1, 2, 3, 255]);
        assert_eq!(picture.get_pixel(1, 0).0, [0; 4]);
    }

    #[test]
    fn armour_alone_is_equipment_to_show() {
        let armoured = ImportedPlt {
            equipment: Vec::new(),
            ..doll()
        };
        assert!(armoured.has_equipment());
        let bare = ImportedPlt {
            armored: None,
            ..armoured
        };
        assert!(!bare.has_equipment());
        assert_eq!(
            bare.render(Equipment::Shown),
            bare.render(Equipment::Hidden)
        );
    }

    #[test]
    fn a_doll_takes_other_colours() {
        let colors = Colors {
            skin: 1,
            ..Colors::default()
        };
        let doll = doll().with_colors(colors);
        assert_eq!(doll.colors(), colors);
        assert_eq!(
            doll.render(Equipment::Hidden).get_pixel(0, 0).0,
            [4, 5, 6, 255]
        );
    }

    #[test]
    fn a_lone_plt_has_no_equipment() {
        let game_data = GameData::new(
            vec![],
            infinitier_common::Game::Bg2,
            infinitier_fs::CaseInsensitiveFS::empty(),
        );
        let plt = ImportedPlt::load("test", &plt_source(&[0, 0, 255, 0]), &game_data)
            .expect("a valid PLT");
        assert!(!plt.has_equipment());
        assert_eq!(plt.format(), ArtFormat::Plt);
        assert_eq!(plt.format().label(), "PLT");
        assert_eq!(plt.colors(), Colors::default());
        assert_eq!(plt.render(Equipment::Shown), plt.render(Equipment::Hidden));
    }
}
