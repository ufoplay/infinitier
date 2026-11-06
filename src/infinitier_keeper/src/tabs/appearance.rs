//! Appearance tab.
//!
//! The creature's animation, picked from ANIMATE.IDS by name (title-cased);
//! its seven colours (hair/skin/clothing×2/armour/leather/metal) as beveled
//! swatches (see [`swatch`]), each picked from the game's gradients; the
//! paperdoll painted in them, with its equipment or without
//! ([`ImportedPlt`]); and its small and large portraits, each picked from
//! the game's pictures for that portrait ([`portrait_names`]).
//!
//! [`portrait_names`]: infinitier_core::portraits::portrait_names

mod swatch;

use std::collections::HashSet;

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::plt::{Equipment, Gradients, ImportedPlt};
use infinitier_core::portraits::PortraitSlot;
use infinitier_core::resource::cre::{Colors, Cre};
use infinitier_core::resource::ids::Ids;
use slint::{Image, SharedString};

use crate::state::Model;
use crate::{ColorSwatch, icons};

/// One of a creature's seven colours, out of all of them.
type Slot = fn(&mut Colors) -> &mut u8;

/// The colour slots, in display order: a label and the colour it is.
const SLOTS: [(&str, Slot); 7] = [
    ("Hair", |c| &mut c.hair),
    ("Skin", |c| &mut c.skin),
    ("Clothing Major", |c| &mut c.major),
    ("Clothing Minor", |c| &mut c.minor),
    ("Armor", |c| &mut c.armor),
    ("Leather", |c| &mut c.leather),
    ("Metal", |c| &mut c.metal),
];

/// The tab's content.
pub struct Page {
    /// The animations to pick from, by name, and the creature's.
    pub animations: Vec<SharedString>,
    pub animation_index: i32,
    pub swatches: Vec<ColorSwatch>,
    /// Every gradient a colour can pick, as a swatch; empty when the game
    /// has none.
    pub palette: Vec<Image>,
    /// Empty when the creature's animation has no paperdoll.
    pub doll: Image,
    /// Whether the doll has equipment to show or hide.
    pub doll_has_equipment: bool,
    /// Whether the doll shows it.
    pub equipment: Equipment,
    pub small_portrait: Portrait,
    pub large_portrait: Portrait,
}

/// One of the creature's portraits.
pub struct Portrait {
    /// The pictures to pick it from, by name, and the creature's.
    pub names: Vec<SharedString>,
    pub index: i32,
    /// Empty when the game has no picture by that name.
    pub image: Image,
}

impl Default for Portrait {
    fn default() -> Self {
        Self {
            names: Vec::new(),
            index: -1,
            image: Image::default(),
        }
    }
}

/// The animations to pick from, as `(ANIMATE.IDS value, name)`: each value
/// once, under its first name, sorted by name. `current` comes first, by
/// number, when the IDS doesn't list it.
fn animation_choices(ids: Option<&Ids>, current: u32) -> Vec<(u32, String)> {
    let mut seen = HashSet::new();
    let mut choices: Vec<(u32, String)> = ids
        .into_iter()
        .flat_map(|ids| &ids.entries)
        .filter_map(|entry| Some((u32::try_from(entry.value).ok()?, &entry.name)))
        .filter(|(value, _)| seen.insert(*value))
        .map(|(value, name)| (value, title_case(name)))
        .collect();
    choices.sort_by(|a, b| a.1.cmp(&b.1));
    if !seen.contains(&current) {
        choices.insert(0, (current, format!("0x{current:04X}")));
    }
    choices
}

/// [`animation_choices`] from the game's ANIMATE.IDS.
fn game_animation_choices(game_data: &GameData, current: u32) -> Vec<(u32, String)> {
    let ids = game_data.import_ids_by_name("ANIMATE").ok();
    animation_choices(ids.as_deref(), current)
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

/// The name of the creature's portrait in `slot`.
fn portrait_name(cre: &Cre, slot: PortraitSlot) -> &str {
    match slot {
        PortraitSlot::Small => cre.small_portrait_name(),
        PortraitSlot::Large => cre.large_portrait_name(),
    }
}

/// The pictures to pick a portrait from, the game's `names`, with `current`
/// first when it isn't one of them (a picture of another size, one the game
/// lacks, or none); and `current`'s place in them.
fn portrait_choices(names: &[String], current: &str) -> (Vec<String>, usize) {
    match names
        .iter()
        .position(|name| name.eq_ignore_ascii_case(current))
    {
        Some(index) => (names.to_vec(), index),
        None => (
            std::iter::once(current.to_owned())
                .chain(names.iter().cloned())
                .collect(),
            0,
        ),
    }
}

/// How the dropdown names picture `name`.
fn portrait_label(name: &str) -> SharedString {
    if name.is_empty() {
        "None".into()
    } else {
        name.into()
    }
}

/// The picture `name`, empty when the game has none: a BMP, or the BAM that
/// PST keeps as a large portrait.
fn picture(game_data: &GameData, name: &str) -> Image {
    if name.is_empty() {
        return Image::default();
    }
    icons::bmp_image(game_data, name)
        .or_else(|| icons::bam_icon(game_data, name))
        .unwrap_or_default()
}

/// The creature's portrait in `slot`, and the game's pictures to pick it
/// from.
fn portrait(model: &Model, cre: &Cre, slot: PortraitSlot) -> Portrait {
    let current = portrait_name(cre, slot);
    let (choices, index) = portrait_choices(model.portrait_names(slot), current);
    Portrait {
        names: choices.iter().map(|name| portrait_label(name)).collect(),
        index: i32::try_from(index).unwrap_or(-1),
        image: picture(&model.game_data, current),
    }
}

/// One swatch per colour slot, labelled.
fn swatches(gradients: Option<&Gradients>, mut colors: Colors) -> Vec<ColorSwatch> {
    SLOTS
        .iter()
        .map(|(label, slot)| {
            let index = *slot(&mut colors);
            ColorSwatch {
                label: (*label).into(),
                index: index.into(),
                image: swatch::image(gradients, index),
            }
        })
        .collect()
}

/// A swatch of every gradient, in colour order.
fn palette(gradients: Option<&Gradients>) -> Vec<Image> {
    let Some(gradients) = gradients else {
        return Vec::new();
    };
    (0..=u8::MAX)
        .take(gradients.len())
        .map(|index| swatch::image(Some(gradients), index))
        .collect()
}

/// Build the Appearance-tab data for the selected creature.
pub fn build(model: &Model) -> Page {
    let game_data = &model.game_data;
    let Some(cre) = model.selected_imported_cre() else {
        return Page {
            animations: Vec::new(),
            animation_index: -1,
            swatches: Vec::new(),
            palette: Vec::new(),
            doll: Image::default(),
            doll_has_equipment: false,
            equipment: model.doll_equipment,
            small_portrait: Portrait::default(),
            large_portrait: Portrait::default(),
        };
    };
    let animation = cre.animation_id();
    let choices = game_animation_choices(game_data, animation);
    let gradients = Gradients::load_coarse(game_data);
    let doll = ImportedPlt::for_creature(game_data, cre);
    let picture = doll.as_ref().map(|doll| doll.render(model.doll_equipment));
    Page {
        animation_index: choices
            .iter()
            .position(|(value, _)| *value == animation)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
        animations: choices.into_iter().map(|(_, name)| name.into()).collect(),
        swatches: swatches(gradients.as_ref(), cre.colors()),
        palette: palette(gradients.as_ref()),
        doll: picture
            .map(|picture| icons::rgba_image(picture.width(), picture.height(), picture.as_raw()))
            .unwrap_or_default(),
        doll_has_equipment: doll.as_ref().is_some_and(ImportedPlt::has_equipment),
        equipment: model.doll_equipment,
        small_portrait: portrait(model, cre, PortraitSlot::Small),
        large_portrait: portrait(model, cre, PortraitSlot::Large),
    }
}

/// Set the selected creature's animation to the `index`th of the
/// dropdown's choices.
pub fn select_animation(model: &mut Model, index: i32) {
    let Some(current) = model.selected_cre().map(Cre::animation_id) else {
        return;
    };
    let choices = game_animation_choices(&model.game_data, current);
    let Some(&(animation, _)) = usize::try_from(index)
        .ok()
        .and_then(|index| choices.get(index))
    else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        cre.set_animation_id(animation);
    }
}

/// Set the selected creature's portrait in `slot` to the `index`th of its
/// dropdown's choices.
pub fn select_portrait(model: &mut Model, slot: PortraitSlot, index: i32) {
    let Some(current) = model
        .selected_cre()
        .map(|cre| portrait_name(cre, slot).to_owned())
    else {
        return;
    };
    let (choices, _) = portrait_choices(model.portrait_names(slot), &current);
    let Some(name) = usize::try_from(index)
        .ok()
        .and_then(|index| choices.into_iter().nth(index))
    else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        match slot {
            PortraitSlot::Small => cre.set_small_portrait_name(&name),
            PortraitSlot::Large => cre.set_large_portrait_name(&name),
        }
    }
}

/// Set the selected creature's colour in slot `slot` (a swatch row) to
/// gradient `gradient`.
pub fn pick_color(model: &mut Model, slot: i32, gradient: i32) {
    let (Some((_, slot)), Ok(gradient)) = (
        usize::try_from(slot).ok().and_then(|slot| SLOTS.get(slot)),
        u8::try_from(gradient),
    ) else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        let mut colors = cre.colors();
        *slot(&mut colors) = gradient;
        cre.set_colors(colors);
    }
}

#[cfg(test)]
mod tests {
    use infinitier_core::resource::ids::IdsEntry;

    use super::*;

    fn ids(entries: &[(i32, &str)]) -> Ids {
        Ids {
            entries: entries
                .iter()
                .map(|&(value, name)| IdsEntry {
                    value,
                    value_str: format!("0x{value:04X}"),
                    name: name.to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn choices_are_each_value_once_by_name() {
        let ids = ids(&[
            (0x6100, "FIGHTER_MALE_HUMAN"),
            (0x6000, "CLERIC_MALE_HUMAN"),
            (0x6100, "FIGHTER_MALE_HUMAN_ALIAS"),
            (0x7F00, "ANKHEG"),
        ]);
        assert_eq!(
            animation_choices(Some(&ids), 0x6100),
            [
                (0x7F00, "Ankheg".to_owned()),
                (0x6000, "Cleric Male Human".to_owned()),
                (0x6100, "Fighter Male Human".to_owned()),
            ]
        );
    }

    #[test]
    fn an_unlisted_animation_comes_first_by_number() {
        let ids = ids(&[(0x6000, "CLERIC_MALE_HUMAN")]);
        let choices = animation_choices(Some(&ids), 0x1234);
        assert_eq!(choices[0], (0x1234, "0x1234".to_owned()));
        assert_eq!(choices.len(), 2);
        assert_eq!(
            animation_choices(None, 0x6000),
            [(0x6000, "0x6000".to_owned())]
        );
    }

    #[test]
    fn a_listed_portrait_is_picked_where_it_is() {
        let names = ["AJANTISM".to_owned(), "NMINSCM".to_owned()];
        assert_eq!(portrait_choices(&names, "nminscm"), (names.to_vec(), 1));
    }

    #[test]
    fn an_unlisted_or_missing_portrait_comes_first() {
        let names = ["NMINSCM".to_owned()];
        assert_eq!(
            portrait_choices(&names, "CUSTOMS"),
            (vec!["CUSTOMS".to_owned(), "NMINSCM".to_owned()], 0)
        );
        let (choices, index) = portrait_choices(&names, "");
        assert_eq!((choices[index].as_str(), choices.len()), ("", 2));
        assert_eq!(portrait_label(""), "None");
        assert_eq!(portrait_label("NMINSCM"), "NMINSCM");
    }

    #[test]
    fn slots_edit_their_own_colour() {
        let colors = Colors::default();
        for (index, (label, slot)) in SLOTS.iter().enumerate() {
            let mut edited = colors;
            *slot(&mut edited) = 9;
            let swatch = &swatches(None, edited)[index];
            assert_eq!(swatch.label, *label);
            assert_eq!(swatch.index, 9, "{label}");
            let set = swatches(None, edited)
                .iter()
                .filter(|swatch| swatch.index == 9)
                .count();
            assert_eq!(set, 1, "{label} edits only itself");
        }
    }
}
