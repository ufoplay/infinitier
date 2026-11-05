//! The kinds of art a paperdoll is made of, and how each takes a creature's
//! colours.

use std::io;

use image::{Rgba, RgbaImage};
use infinitier_bam_resource::common::Rgb;
use infinitier_bam_resource::{Bam, BamImporter, BamV1, BamV1Frame};
use infinitier_cre_resource::Colors;
use infinitier_datasource::{DataSource, Importer};
use infinitier_plt_resource::{Plt, PltImporter, PltPixel, is_plt};

use super::ArtFormat;
use super::gradients::Gradients;
use crate::game::GameData;

/// A PLT's size, which is also the canvas a paperdoll BAM's halves share.
pub(super) const DOLL_WIDTH: u32 = 128;
pub(super) const DOLL_HEIGHT: u32 = 160;
/// A paperdoll BAM's second frame is anchored this far below its first.
const DOLL_HALF_HEIGHT: i32 = 80;
/// A paperdoll BAM's halves: its first frames. Some hold more, which aren't
/// part of the doll.
const DOLL_HALVES: usize = 2;

/// The opacity of a shadow: the game lets the background through.
const SHADOW_ALPHA: u8 = 128;
/// The palette entry of a BAM's shadow.
const BAM_SHADOW_ENTRY: usize = 1;
/// The colour a BMP paints what's outside the figure with.
const BMP_TRANSPARENT: [u8; 3] = [0, 255, 0];

/// Where a sprite palette's seven gradients start, in [`sprite_groups`]
/// order, and how many shades each has.
const GRADIENTS_START: usize = 0x04;
const GRADIENT_SHADES: usize = 12;

/// The rest of a sprite palette: shades 1 to 8 of a colour (its index in
/// [`sprite_groups`]) repeated at these offsets.
const REPEATS: [(usize, usize); 20] = {
    const METAL: usize = 0;
    const MINOR: usize = 1;
    const MAJOR: usize = 2;
    const SKIN: usize = 3;
    const LEATHER: usize = 4;
    [
        (0x58, MINOR),
        (0x60, MAJOR),
        (0x68, MINOR),
        (0x70, METAL),
        (0x78, LEATHER),
        (0x80, LEATHER),
        (0x88, MINOR),
        (0x90, LEATHER),
        (0x98, LEATHER),
        (0xA0, LEATHER),
        (0xB0, SKIN),
        (0xB8, LEATHER),
        (0xC0, LEATHER),
        (0xC8, LEATHER),
        (0xD0, LEATHER),
        (0xD8, LEATHER),
        (0xE0, LEATHER),
        (0xE8, LEATHER),
        (0xF0, LEATHER),
        (0xF8, LEATHER),
    ]
};
const REPEAT_FIRST_SHADE: usize = 1;
const REPEAT_SHADES: usize = 8;

/// The gradients a doll's art is painted from: PLTs index the fine ones,
/// sprite palettes take the coarse ones.
#[derive(Debug, Clone, Default)]
pub(super) struct Palettes {
    fine: Option<Gradients>,
    coarse: Option<Gradients>,
}

impl Palettes {
    pub(super) fn load(game_data: &GameData) -> Self {
        Self {
            fine: Gradients::load_fine(game_data),
            coarse: Gradients::load_coarse(game_data),
        }
    }

    #[cfg(test)]
    pub(super) fn new(fine: Gradients, coarse: Gradients) -> Self {
        Self {
            fine: Some(fine),
            coarse: Some(coarse),
        }
    }
}

/// One piece of a paperdoll's art.
#[derive(Debug, Clone)]
pub(super) enum Art {
    /// Each pixel names one of the colours and a shade of it.
    Plt(Plt),
    /// A paperdoll BAM: its frames are the top and bottom halves of a canvas
    /// the size of a PLT, and its palette's ranges take the colours.
    Halves {
        frames: Vec<BamV1Frame>,
        palette: Vec<Rgb>,
    },
    /// One frame of a sprite, whose palette's ranges take the colours.
    Sprite {
        frame: BamV1Frame,
        palette: Vec<Rgb>,
    },
    /// Painted already: the colours don't apply.
    Picture(RgbaImage),
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// A BAM with a palette to paint.
fn v1(bam: Bam, name: &str) -> io::Result<BamV1> {
    match bam {
        Bam::V1(bam) => Ok(bam),
        Bam::V2(_) => Err(invalid(format!(
            "{name} is a BAM V2, which has no palette to paint"
        ))),
    }
}

impl Art {
    /// What a PLT resource holds: a PLT, or the BAM the Enhanced Editions
    /// keep under that type.
    pub(super) fn from_plt_resource(name: &str, source: &DataSource) -> io::Result<Self> {
        if is_plt(source)? {
            return Self::plt(name, source);
        }
        Self::halves(name, source)
    }

    /// A PLT, and only that.
    pub(super) fn plt(name: &str, source: &DataSource) -> io::Result<Self> {
        PltImporter { name }.import(source).map(Self::Plt)
    }

    /// A paperdoll BAM.
    pub(super) fn halves(name: &str, source: &DataSource) -> io::Result<Self> {
        let BamV1 {
            frames, palette, ..
        } = v1(BamImporter { name }.import(source)?, name)?;
        Ok(Self::Halves { frames, palette })
    }

    /// The first frame of a sprite's cycle `cycle`.
    pub(super) fn sprite(name: &str, source: &DataSource, cycle: usize) -> io::Result<Self> {
        let BamV1 {
            frames,
            palette,
            cycles,
            ..
        } = v1(BamImporter { name }.import(source)?, name)?;
        let frame = cycles
            .get(cycle)
            .and_then(|cycle| cycle.frame_indices.first())
            .and_then(|index| frames.into_iter().nth(*index))
            .ok_or_else(|| invalid(format!("{name} has no frame in cycle {cycle}")))?;
        Ok(Self::Sprite { frame, palette })
    }

    /// A painted picture, with its [`BMP_TRANSPARENT`] pixels see-through.
    pub(super) fn picture(image: &RgbaImage) -> Self {
        let mut picture = image.clone();
        for pixel in picture.pixels_mut() {
            let [r, g, b, _] = pixel.0;
            if [r, g, b] == BMP_TRANSPARENT {
                *pixel = Rgba([0; 4]);
            }
        }
        Self::Picture(picture)
    }

    /// What the art is stored as.
    pub(super) fn format(&self) -> ArtFormat {
        match self {
            Self::Plt(_) => ArtFormat::Plt,
            Self::Halves { .. } | Self::Sprite { .. } => ArtFormat::Bam,
            Self::Picture(_) => ArtFormat::Bmp,
        }
    }

    /// The size of the canvas the art is drawn on.
    pub(super) fn size(&self) -> (u32, u32) {
        match self {
            Self::Plt(plt) => (plt.width(), plt.height()),
            Self::Halves { .. } => (DOLL_WIDTH, DOLL_HEIGHT),
            Self::Sprite { frame, .. } => (frame.width, frame.height),
            Self::Picture(picture) => picture.dimensions(),
        }
    }

    /// Draw the art over `canvas` in `colors`. What falls outside the canvas
    /// is dropped.
    pub(super) fn draw(&self, canvas: &mut RgbaImage, colors: Colors, palettes: &Palettes) {
        match self {
            Self::Plt(plt) => {
                let groups = plt_groups(colors);
                let width = plt.width() as usize;
                for (y, row) in plt.pixels().chunks_exact(width).enumerate() {
                    for (x, pixel) in row.iter().enumerate() {
                        let color = plt_color(*pixel, groups, palettes.fine.as_ref());
                        blend_at(canvas, x as i64, y as i64, color);
                    }
                }
            }
            Self::Halves { frames, palette } => {
                let palette = sprite_palette(palette, palettes.coarse.as_ref(), colors);
                for (half, frame) in (0..).zip(frames.iter().take(DOLL_HALVES)) {
                    let top = half * DOLL_HALF_HEIGHT - frame.center_y;
                    place(canvas, frame, &palette, -frame.center_x, top);
                }
            }
            Self::Sprite { frame, palette } => {
                let palette = sprite_palette(palette, palettes.coarse.as_ref(), colors);
                place(canvas, frame, &palette, 0, 0);
            }
            Self::Picture(picture) => {
                for (x, y, pixel) in picture.enumerate_pixels() {
                    blend_at(canvas, i64::from(x), i64::from(y), pixel.0);
                }
            }
        }
    }
}

/// The colours in the order a PLT's pixels number them.
fn plt_groups(colors: Colors) -> [u8; 7] {
    let Colors {
        hair,
        skin,
        major,
        minor,
        armor,
        leather,
        metal,
    } = colors;
    [skin, hair, metal, armor, leather, minor, major]
}

/// The colours in the order a sprite palette holds their gradients.
fn sprite_groups(colors: Colors) -> [u8; 7] {
    let Colors {
        hair,
        skin,
        major,
        minor,
        armor,
        leather,
        metal,
    } = colors;
    [metal, minor, major, skin, leather, armor, hair]
}

/// A PLT pixel's colour: a shade of one of `groups`' gradients. Black for a
/// colour or shade the gradients don't have.
fn plt_color(pixel: PltPixel, groups: [u8; 7], gradients: Option<&Gradients>) -> [u8; 4] {
    if pixel.is_transparent() {
        return [0; 4];
    }
    if pixel.is_shadow() {
        return [0, 0, 0, SHADOW_ALPHA];
    }
    let [r, g, b] = groups
        .get(usize::from(pixel.color_group))
        .and_then(|index| gradients?.ramp(*index))
        .and_then(|ramp| ramp.get(usize::from(pixel.shade)))
        .copied()
        .unwrap_or_default();
    [r, g, b, u8::MAX]
}

/// A BAM's palette with its colour ranges replaced by the creature's colours,
/// and its shadow see-through.
fn sprite_palette(base: &[Rgb], gradients: Option<&Gradients>, colors: Colors) -> Vec<Rgb> {
    let groups = sprite_groups(colors);
    let ramp = |group: usize| gradients?.ramp(*groups.get(group)?);
    let mut palette = base.to_vec();
    if let Some(shadow) = palette.get_mut(BAM_SHADOW_ENTRY) {
        *shadow = Rgb {
            r: 0,
            g: 0,
            b: 0,
            alpha: SHADOW_ALPHA,
        };
    }
    for group in 0..groups.len() {
        if let Some(ramp) = ramp(group) {
            let start = GRADIENTS_START + group * GRADIENT_SHADES;
            paint(&mut palette, start, ramp.iter().take(GRADIENT_SHADES));
        }
    }
    for (start, group) in REPEATS {
        if let Some(ramp) = ramp(group) {
            let shades = ramp.iter().skip(REPEAT_FIRST_SHADE).take(REPEAT_SHADES);
            paint(&mut palette, start, shades);
        }
    }
    palette
}

/// Overwrite `palette`'s colours from `start` on, keeping their alpha.
fn paint<'a>(palette: &mut [Rgb], start: usize, shades: impl Iterator<Item = &'a [u8; 3]>) {
    for (entry, [r, g, b]) in palette.iter_mut().skip(start).zip(shades) {
        (entry.r, entry.g, entry.b) = (*r, *g, *b);
    }
}

/// Draw `frame`, its pixels looked up in `palette`, with its top-left corner
/// at `(left, top)`.
fn place(canvas: &mut RgbaImage, frame: &BamV1Frame, palette: &[Rgb], left: i32, top: i32) {
    let width = frame.width as usize;
    if width == 0 {
        return;
    }
    for (y, row) in frame.pixel_palette_indexes.chunks_exact(width).enumerate() {
        for (x, index) in row.iter().enumerate() {
            if let Some(Rgb { r, g, b, alpha }) = palette.get(usize::from(*index)) {
                let (x, y) = (i64::from(left) + x as i64, i64::from(top) + y as i64);
                blend_at(canvas, x, y, [*r, *g, *b, *alpha]);
            }
        }
    }
}

/// Draw `color` over the canvas pixel at `(x, y)`, if it's on the canvas.
fn blend_at(canvas: &mut RgbaImage, x: i64, y: i64, color: [u8; 4]) {
    let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
        return;
    };
    if let Some(pixel) = canvas.get_pixel_mut_checked(x, y) {
        *pixel = Rgba(over(color, pixel.0));
    }
}

/// `top` over `bottom`: a see-through colour, like a shadow, darkens what's
/// beneath it instead of replacing it.
fn over(top: [u8; 4], bottom: [u8; 4]) -> [u8; 4] {
    let [.., top_alpha] = top;
    match top_alpha {
        0 => bottom,
        u8::MAX => top,
        _ => {
            let [.., bottom_alpha] = bottom;
            let (ta, ba) = (u32::from(top_alpha), u32::from(bottom_alpha));
            let max = u32::from(u8::MAX);
            let bottom_share = ba * (max - ta) / max;
            let alpha = ta + bottom_share;
            let mix = |t: u8, b: u8| {
                let mixed = (u32::from(t) * ta + u32::from(b) * bottom_share) / alpha.max(1);
                u8::try_from(mixed).unwrap_or(u8::MAX)
            };
            let [tr, tg, tb, _] = top;
            let [br, bg, bb, _] = bottom;
            [
                mix(tr, br),
                mix(tg, bg),
                mix(tb, bb),
                u8::try_from(alpha).unwrap_or(u8::MAX),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use infinitier_plt_resource::PLT_SIGNATURE;
    use infinitier_test_utils::get_assets_path;

    use super::*;

    const COLORS: Colors = Colors {
        metal: 0,
        minor: 1,
        major: 2,
        skin: 3,
        leather: 4,
        armor: 5,
        hair: 6,
    };

    /// Seven gradients of `shades` shades: `[gradient, shade, 0]`.
    fn gradients(shades: u8) -> Gradients {
        Gradients::from_ramps(
            (0..7)
                .map(|ramp| (0..shades).map(|shade| [ramp, shade, 0]).collect())
                .collect(),
        )
    }

    fn palettes() -> Palettes {
        Palettes::new(gradients(u8::MAX), gradients(12))
    }

    fn opaque(rgb: [u8; 3]) -> Rgb {
        let [r, g, b] = rgb;
        Rgb {
            r,
            g,
            b,
            alpha: u8::MAX,
        }
    }

    fn rgb(entry: &Rgb) -> [u8; 3] {
        [entry.r, entry.g, entry.b]
    }

    /// A PLT file `width` pixels wide, its rows given bottom first as the file
    /// stores them.
    fn plt_file(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
        let mut file = PLT_SIGNATURE.to_vec();
        file.extend([0; 8]);
        file.extend(width.to_le_bytes());
        file.extend(height.to_le_bytes());
        file.extend(pixels);
        file
    }

    fn plt(width: u32, height: u32, pixels: &[u8]) -> Art {
        Art::from_plt_resource("test", &DataSource::new(plt_file(width, height, pixels)))
            .expect("a valid PLT")
    }

    /// A `width × height` frame anchored at `center`, all of palette entry 2.
    fn frame(width: u32, height: u32, center: (i32, i32)) -> BamV1Frame {
        BamV1Frame {
            width,
            height,
            center_x: center.0,
            center_y: center.1,
            pixel_palette_indexes: vec![2; (width * height) as usize],
        }
    }

    fn halves(frames: Vec<BamV1Frame>) -> Art {
        let mut palette = vec![opaque([9; 3]); 3];
        palette[0].alpha = 0;
        Art::Halves { frames, palette }
    }

    fn drawn(art: &Art) -> RgbaImage {
        let (width, height) = art.size();
        let mut canvas = RgbaImage::new(width, height);
        art.draw(&mut canvas, COLORS, &palettes());
        canvas
    }

    fn painted_at(canvas: &RgbaImage, x: u32, y: u32) -> bool {
        let [.., alpha] = canvas.get_pixel(x, y).0;
        alpha > 0
    }

    /// The sprite palette for `COLORS`, from an all-white one.
    fn painted_palette() -> Vec<Rgb> {
        sprite_palette(&[opaque([u8::MAX; 3]); 256], Some(&gradients(12)), COLORS)
    }

    #[test]
    fn plt_pixels_take_their_groups_colour() {
        // Skin (group 0), shade 5; major clothing (group 6), shade 200; a
        // transparent one.
        let canvas = drawn(&plt(3, 1, &[5, 0, 200, 6, u8::MAX, 0]));
        assert_eq!(canvas.get_pixel(0, 0).0, [COLORS.skin, 5, 0, 255]);
        assert_eq!(canvas.get_pixel(1, 0).0, [COLORS.major, 200, 0, 255]);
        assert_eq!(canvas.get_pixel(2, 0).0, [0; 4]);
    }

    #[test]
    fn plt_shadow_lets_the_background_through() {
        let canvas = drawn(&plt(1, 1, &[3, PltPixel::SHADOW_GROUP]));
        assert_eq!(canvas.get_pixel(0, 0).0, [0, 0, 0, SHADOW_ALPHA]);
    }

    #[test]
    fn plt_pixels_outside_the_gradients_are_black() {
        let (art, mut canvas) = (plt(1, 1, &[9, 0]), RgbaImage::new(1, 1));
        art.draw(
            &mut canvas,
            COLORS,
            &Palettes::new(gradients(4), gradients(4)),
        );
        assert_eq!(canvas.get_pixel(0, 0).0, [0, 0, 0, 255]);
    }

    #[test]
    fn a_plt_resource_may_hold_a_bam() -> io::Result<()> {
        let bam = std::fs::read(get_assets_path().join("BAM/V1/01/1chan03B_compressed.BAM"))?;
        let art = Art::from_plt_resource("test", &DataSource::new(bam))?;
        assert!(matches!(art, Art::Halves { .. }));
        assert_eq!(art.size(), (DOLL_WIDTH, DOLL_HEIGHT));
        Ok(())
    }

    #[test]
    fn only_a_plt_is_a_plt() -> io::Result<()> {
        let bam = std::fs::read(get_assets_path().join("BAM/V1/01/1chan03B_compressed.BAM"))?;
        assert!(Art::plt("test", &DataSource::new(bam)).is_err());
        let art = Art::plt("test", &DataSource::new(plt_file(1, 1, &[0, 0])))?;
        assert!(matches!(art, Art::Plt(_)));
        Ok(())
    }

    #[test]
    fn art_knows_what_it_is_stored_as() -> io::Result<()> {
        assert_eq!(plt(1, 1, &[0, 0]).format(), ArtFormat::Plt);
        let bam = std::fs::read(get_assets_path().join("BAM/V1/01/1chan03B_compressed.BAM"))?;
        let halves = Art::from_plt_resource("test", &DataSource::new(bam))?;
        assert_eq!(halves.format(), ArtFormat::Bam);
        assert_eq!(Art::picture(&RgbaImage::new(1, 1)).format(), ArtFormat::Bmp);
        Ok(())
    }

    #[test]
    fn a_plt_resource_holding_neither_is_an_error() {
        let garbage = DataSource::new(b"NOT A RESOURCE".to_vec());
        assert!(Art::from_plt_resource("test", &garbage).is_err());
    }

    #[test]
    fn a_dolls_second_half_goes_below_the_first() {
        // Their anchors overlap: only their halves tell them apart.
        let canvas = drawn(&halves(vec![
            frame(2, 2, (-10, -78)),
            frame(2, 2, (-10, 0)),
        ]));
        for (x, y) in [(10, 78), (11, 79), (10, 80), (11, 81)] {
            assert!(painted_at(&canvas, x, y), "({x}, {y})");
        }
        for (x, y) in [(10, 77), (10, 82), (12, 78)] {
            assert!(!painted_at(&canvas, x, y), "({x}, {y})");
        }
    }

    #[test]
    fn a_dolls_frames_past_its_halves_arent_drawn() {
        // A third frame would land at the bottom of the canvas.
        let canvas = drawn(&halves(vec![
            frame(1, 1, (0, 0)),
            frame(1, 1, (0, 0)),
            frame(1, 1, (-5, 10)),
        ]));
        assert!(!painted_at(&canvas, 5, 150));
    }

    #[test]
    fn a_dolls_transparent_pixels_dont_overwrite() {
        let clear = BamV1Frame {
            pixel_palette_indexes: vec![0],
            ..frame(1, 1, (-5, 75))
        };
        let canvas = drawn(&halves(vec![frame(1, 1, (-5, -5)), clear]));
        assert!(painted_at(&canvas, 5, 5));
    }

    #[test]
    fn what_falls_outside_the_canvas_is_dropped() {
        let width = DOLL_WIDTH as i32;
        let height = DOLL_HEIGHT as i32;
        for center in [(5, 0), (0, 5), (-width, 0), (0, -height)] {
            let canvas = drawn(&halves(vec![frame(1, 1, center)]));
            assert!(canvas.pixels().all(|pixel| pixel.0 == [0; 4]), "{center:?}");
        }
    }

    #[test]
    fn a_frame_pointing_past_its_palette_draws_nothing_there() {
        let mut palette = vec![opaque([9; 3]); 2];
        palette[0].alpha = 0;
        let art = Art::Sprite {
            frame: frame(1, 1, (0, 0)),
            palette,
        };
        assert_eq!(drawn(&art).get_pixel(0, 0).0, [0; 4]);
    }

    #[test]
    fn only_pure_green_is_see_through_in_a_picture() {
        let image = RgbaImage::from_raw(
            4,
            1,
            vec![0, 255, 0, 255, 1, 255, 0, 255, 0, 254, 0, 255, 9, 8, 7, 255],
        )
        .expect("4 pixels");
        let Art::Picture(picture) = Art::picture(&image) else {
            panic!("a picture")
        };
        assert_eq!(
            picture.into_raw(),
            [0, 0, 0, 0, 1, 255, 0, 255, 0, 254, 0, 255, 9, 8, 7, 255]
        );
    }

    #[test]
    fn a_shadow_darkens_what_it_falls_on() {
        let shadow = [0, 0, 0, SHADOW_ALPHA];
        let darkened = over(shadow, [200, 100, 50, 255]);
        assert_eq!(darkened[3], 255);
        assert!(darkened[0] < 200 && darkened[0] > 0);
        // Where there's nothing beneath, it's the shadow itself.
        assert_eq!(over(shadow, [0; 4]), shadow);
        // Opaque and see-through tops.
        assert_eq!(over([1, 2, 3, 255], [9, 9, 9, 255]), [1, 2, 3, 255]);
        assert_eq!(over([1, 2, 3, 0], [9, 9, 9, 255]), [9, 9, 9, 255]);
    }

    #[test]
    fn sprite_gradients_follow_their_order() {
        let palette = painted_palette();
        // Metal comes first and hair last, each 12 shades from 0x04.
        assert_eq!(rgb(&palette[0x04]), [0, 0, 0]);
        assert_eq!(rgb(&palette[0x0F]), [0, 11, 0]);
        assert_eq!(rgb(&palette[0x10]), [1, 0, 0]);
        assert_eq!(rgb(&palette[0x4C]), [6, 0, 0]);
        assert_eq!(rgb(&palette[0x57]), [6, 11, 0]);
    }

    #[test]
    fn sprite_repeats_skip_a_gradients_first_shade() {
        let palette = painted_palette();
        // Minor, the second gradient.
        assert_eq!(rgb(&palette[0x58]), [1, 1, 0]);
        assert_eq!(rgb(&palette[0x5F]), [1, 8, 0]);
        // Skin, the fourth.
        assert_eq!(rgb(&palette[0xB0]), [3, 1, 0]);
        // Leather, the fifth, to the palette's end.
        assert_eq!(rgb(&palette[0xFF]), [4, 8, 0]);
    }

    #[test]
    fn sprite_palette_keeps_what_it_doesnt_paint() {
        let palette = painted_palette();
        // The transparent entry, the reserved ones and the gap between the
        // repeats.
        assert_eq!(rgb(&palette[0x00]), [u8::MAX; 3]);
        assert_eq!(rgb(&palette[0x03]), [u8::MAX; 3]);
        assert_eq!(rgb(&palette[0xA8]), [u8::MAX; 3]);
    }

    #[test]
    fn sprite_shadow_lets_the_background_through() {
        // Whatever the BAM holds there: hot pink in the Enhanced Editions.
        let palette = painted_palette();
        assert_eq!(rgb(&palette[BAM_SHADOW_ENTRY]), [0, 0, 0]);
        assert_eq!(palette[BAM_SHADOW_ENTRY].alpha, SHADOW_ALPHA);
    }

    #[test]
    fn sprite_palette_keeps_the_alpha() {
        let mut base = vec![opaque([u8::MAX; 3]); 256];
        base[GRADIENTS_START].alpha = 0;
        let palette = sprite_palette(&base, Some(&gradients(12)), COLORS);
        assert_eq!(palette[GRADIENTS_START].alpha, 0);
        assert_eq!(rgb(&palette[GRADIENTS_START]), [0, 0, 0]);
    }

    #[test]
    fn sprite_palette_survives_an_unknown_color_or_no_gradients() {
        // Past the gradients: that range keeps the BAM's own colours.
        let colors = Colors {
            hair: 200,
            ..COLORS
        };
        let base = vec![opaque([u8::MAX; 3]); 256];
        let palette = sprite_palette(&base, Some(&gradients(12)), colors);
        assert_eq!(rgb(&palette[0x4C]), [u8::MAX; 3]);
        assert_eq!(rgb(&palette[0x04]), [0, 0, 0]);

        let unpainted = sprite_palette(&base, None, COLORS);
        assert_eq!(rgb(&unpainted[0x04]), [u8::MAX; 3]);
    }
}
