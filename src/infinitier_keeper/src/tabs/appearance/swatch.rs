//! Colour swatches: a colour's gradient (12 shades, brightest first; see
//! [`Gradients::load_coarse`]) painted as a centre-bright, edge-dark bevel
//! by Chebyshev distance, matching EEKeeper.

use infinitier_core::imported_resource::plt::Gradients;
use slint::Image;

use crate::icons;

/// Side length, in pixels, of a rendered swatch.
pub const SWATCH_PX: usize = 24;

/// The swatch of colour `index`: transparent when the game has no
/// gradients, flat black for an index it has no gradient for.
pub fn image(gradients: Option<&Gradients>, index: u8) -> Image {
    let rgba = match gradients {
        Some(gradients) => swatch_rgba(gradients.ramp(index)),
        None => vec![0; SWATCH_PX * SWATCH_PX * 4],
    };
    icons::rgba_image(SWATCH_PX as u32, SWATCH_PX as u32, &rgba)
}

/// A swatch as `SWATCH_PX × SWATCH_PX` RGBA bytes: a centre-bright,
/// edge-dark bevel walking the ramp outward by Chebyshev distance. No ramp
/// renders as a flat black square.
fn swatch_rgba(ramp: Option<&[[u8; 3]]>) -> Vec<u8> {
    let n = SWATCH_PX;
    let center = (n - 1) as f32 / 2.0;
    let mut pixels = vec![0u8; n * n * 4];

    for y in 0..n {
        for x in 0..n {
            let d = (x as f32 - center).abs().max((y as f32 - center).abs());
            let rgb = match ramp {
                Some(shades) => {
                    let t = if center > 0.0 { d / center } else { 0.0 };
                    let idx = ((t * shades.len() as f32) as usize).min(shades.len() - 1);
                    shades[idx]
                }
                None => [0, 0, 0],
            };
            let i = (y * n + x) * 4;
            pixels[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    pixels
}
