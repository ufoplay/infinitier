//! Appearance tab (read-only).
//!
//! Shows the creature's animation resolved to a name (ANIMATE.IDS,
//! title-cased), the seven colours (hair/skin/clothing×2/armour/leather/metal)
//! as beveled swatches (see [`swatch`]), the paperdoll painted in them, with
//! its equipment or without ([`ImportedPlt`]), and the portrait.

mod swatch;

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::plt::{Equipment, Gradients, ImportedPlt};
use infinitier_core::resource::cre::{Colors, Cre};
use slint::{Image, SharedString};

use crate::state::Model;
use crate::{ColorSwatch, icons};

/// The tab's content.
pub struct Page {
    pub name: SharedString,
    pub swatches: Vec<ColorSwatch>,
    /// Empty when the creature's animation has no paperdoll.
    pub doll: Image,
    /// Whether the doll has equipment to show or hide.
    pub doll_has_equipment: bool,
    /// Whether the doll shows it.
    pub equipment: Equipment,
    /// Empty when the game has no portrait for the creature.
    pub portrait: Image,
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

/// The creature's large (then small) portrait, empty when the game has
/// neither. A BMP, except for the BAM that PST and PSTEE keep in the large
/// slot.
fn portrait(game_data: &GameData, cre: &Cre) -> Image {
    [cre.large_portrait_name(), cre.small_portrait_name()]
        .into_iter()
        .filter(|name| !name.is_empty())
        .find_map(|name| {
            icons::bmp_image(game_data, name).or_else(|| icons::bam_icon(game_data, name))
        })
        .unwrap_or_default()
}

/// One swatch per colour, labelled.
fn swatches(game_data: &GameData, colors: Colors) -> Vec<ColorSwatch> {
    let gradients = Gradients::load_coarse(game_data);
    let Colors {
        hair,
        skin,
        major,
        minor,
        armor,
        leather,
        metal,
    } = colors;
    [
        ("Hair", hair),
        ("Skin", skin),
        ("Clothing Major", major),
        ("Clothing Minor", minor),
        ("Armor", armor),
        ("Leather", leather),
        ("Metal", metal),
    ]
    .into_iter()
    .map(|(label, index)| ColorSwatch {
        label: label.into(),
        image: swatch::image(gradients.as_ref(), index),
    })
    .collect()
}

/// Build the Appearance-tab data for the selected creature.
pub fn build(model: &Model) -> Page {
    let game_data = &model.game_data;
    let Some(cre) = model.selected_imported_cre() else {
        return Page {
            name: SharedString::new(),
            swatches: Vec::new(),
            doll: Image::default(),
            doll_has_equipment: false,
            equipment: model.doll_equipment,
            portrait: Image::default(),
        };
    };
    let doll = ImportedPlt::for_creature(game_data, cre);
    let picture = doll.as_ref().map(|doll| doll.render(model.doll_equipment));
    Page {
        name: resolve_name(game_data, cre.animation_id()).into(),
        swatches: swatches(game_data, cre.colors()),
        doll: picture
            .map(|picture| icons::rgba_image(picture.width(), picture.height(), picture.as_raw()))
            .unwrap_or_default(),
        doll_has_equipment: doll.as_ref().is_some_and(ImportedPlt::has_equipment),
        equipment: model.doll_equipment,
        portrait: portrait(game_data, cre),
    }
}
