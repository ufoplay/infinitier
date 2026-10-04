//! Appearance tab for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/appearance/`. Shows the creature's
//! animation resolved to a name (ANIMATE.IDS, title-cased) and the seven
//! colour gradients (hair/skin/clothing×2/armour/leather/metal) as beveled
//! swatch images built from the palette BMP. `appearance_data` and the
//! `palette` swatch algorithm are duplicated from the egui sources; swatches
//! are wrapped in Slint `Image`s via `SharedPixelBuffer`. Display-only.

mod palette;

use infinitier_core::game::GameData;
use infinitier_core::resource::cre::{Cre, CreHeader};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};

use crate::ColorSwatch;
use crate::state::Model;
use palette::{Palette, SWATCH_PX};

struct ColorSlot {
    label: &'static str,
    index: u8,
}

struct AppearanceData {
    animation_id: u32,
    colors: Vec<ColorSlot>,
}

fn appearance_data(cre: &Cre) -> Option<AppearanceData> {
    macro_rules! colour_slots {
        ($h:expr, $( $label:literal => $field:ident ),+ $(,)?) => {
            vec![ $( ColorSlot { label: $label, index: $h.$field } ),+ ]
        };
    }
    let (animation_id, colors) = match &cre.header {
        CreHeader::V10(h) => (
            h.animation_id,
            colour_slots!(
                h,
                "Hair" => hair_colour_index,
                "Skin" => skin_colour_index,
                "Clothing Major" => major_colour_index,
                "Clothing Minor" => minor_colour_index,
                "Armor" => armor_colour_index,
                "Leather" => leather_colour_index,
                "Metal" => metal_colour_index,
            ),
        ),
        CreHeader::V90(h) => (
            h.animation_id_animate_ids,
            colour_slots!(
                h,
                "Hair" => hair_colour_index,
                "Skin" => skin_colour_index,
                "Clothing Major" => major_colour_index,
                "Clothing Minor" => minor_colour_index,
                "Armor" => armor_colour_index,
                "Leather" => leather_colour_index,
                "Metal" => metal_colour_index,
            ),
        ),
        CreHeader::V12(h) => (
            h.animation_id_animate_ids,
            colour_slots!(
                h,
                "Hair" => hair_colour_index_bg1_animations,
                "Skin" => skin_colour_index_bg1_animations,
                "Clothing Major" => major_colour_index_bg1_animations,
                "Clothing Minor" => minor_colour_index_bg1_animations,
                "Armor" => armor_colour_index_bg1_animations,
                "Leather" => leather_colour_index_bg1_animations,
                "Metal" => metal_colour_index_bg1_animations,
            ),
        ),
        CreHeader::V22(h) => (
            h.animation_id_animate_ids_0x002c,
            colour_slots!(
                h,
                "Hair" => hair_colour_index_bg1_animations,
                "Skin" => skin_colour_index_bg1_animations,
                "Clothing Major" => major_colour_index_bg1_animations,
                "Clothing Minor" => minor_colour_index_bg1_animations,
                "Armor" => armor_colour_index_bg1_animations,
                "Leather" => leather_colour_index_bg1_animations,
                "Metal" => metal_colour_index_bg1_animations,
            ),
        ),
    };
    Some(AppearanceData {
        animation_id,
        colors,
    })
}

/// Resolve an animation id to a prettified ANIMATE.IDS name, falling back to
/// the raw hex id.
fn resolve_name(game_data: &GameData, animation_id: u32) -> String {
    game_data
        .import_ids_by_name("ANIMATE")
        .ok()
        .and_then(|ids| ids.of_value(animation_id as i32).map(title_case))
        .unwrap_or_else(|| format!("0x{animation_id:04X}"))
}

fn title_case(symbol: &str) -> String {
    symbol
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Wrap `SWATCH_PX²` RGBA bytes in a Slint `Image`.
fn rgba_image(rgba: &[u8]) -> Image {
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(SWATCH_PX as u32, SWATCH_PX as u32);
    buf.make_mut_bytes().copy_from_slice(rgba);
    Image::from_rgba8(buf)
}

fn placeholder_image() -> Image {
    rgba_image(&vec![0u8; SWATCH_PX * SWATCH_PX * 4])
}

/// Build the Appearance-tab data for the selected creature: the animation
/// name and the seven colour swatches.
pub fn build(model: &Model) -> (SharedString, Vec<ColorSwatch>) {
    let Some(cre) = model.selected_cre() else {
        return (SharedString::new(), Vec::new());
    };
    let Some(data) = appearance_data(cre) else {
        return (
            "Appearance is unavailable for this creature's format.".into(),
            Vec::new(),
        );
    };
    let name = resolve_name(&model.game_data, data.animation_id);
    let palette = Palette::load(&model.game_data);
    let swatches = data
        .colors
        .iter()
        .map(|slot| ColorSwatch {
            label: slot.label.into(),
            image: match &palette {
                Some(p) => rgba_image(&p.swatch_rgba(slot.index)),
                None => placeholder_image(),
            },
        })
        .collect();
    (name.into(), swatches)
}
