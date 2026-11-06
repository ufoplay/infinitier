//! The colours equipment changes: an item's "set colour" effects (opcode 7)
//! hold while it's equipped. Each names a part of the figure and one of its
//! colours, and the gradient that colour takes there, so an item can paint
//! itself (a red helmet, a fiery sword) or the creature's armour.

use infinitier_cre_resource::Colors;
use infinitier_itm_resource::{Itm, ItmEffect};

/// The "set colour" opcode.
const SET_COLOR: u16 = 7;

/// A part of a paperdoll, which equipment colours separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The figure, in its armour.
    Body,
    Weapon,
    /// The shield, or a weapon in the off hand.
    OffHand,
    Helmet,
}

const ALL_PARTS: [Part; 4] = [Part::Body, Part::Weapon, Part::OffHand, Part::Helmet];

/// A colour equipment changes: in `part`, colour `color` (in the order
/// metal, minor, major, skin, leather, armor, hair) takes gradient
/// `gradient`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ColorChange {
    pub(super) part: Part,
    pub(super) color: u8,
    pub(super) gradient: u8,
}

/// The colours `itm` changes while it's equipped.
pub(super) fn changes(itm: &Itm) -> Vec<ColorChange> {
    let start = usize::from(itm.header.equipping_feature_offset());
    let count = usize::from(itm.header.equipping_feature_count());
    itm.effects
        .iter()
        .skip(start)
        .take(count)
        .flat_map(effect_changes)
        .collect()
}

/// What one effect changes. Its second parameter is the location: the low
/// nibble the colour, the high one the part (0 body, 1 weapon, 2 shield,
/// 3 helmet, 15 all of them).
fn effect_changes(effect: &ItmEffect) -> Vec<ColorChange> {
    if effect.opcode != SET_COLOR {
        return Vec::new();
    }
    let location = effect.parameter2;
    let parts: &[Part] = match location >> 4 {
        0 => &[Part::Body],
        1 => &[Part::Weapon],
        2 => &[Part::OffHand],
        3 => &[Part::Helmet],
        0xF => &ALL_PARTS,
        _ => &[],
    };
    let color = (location & 0xF) as u8;
    let gradient = (effect.parameter1 & 0xFF) as u8;
    if color > HAIR {
        return Vec::new();
    }
    parts
        .iter()
        .map(|part| ColorChange {
            part: *part,
            color,
            gradient,
        })
        .collect()
}

/// The last colour, in the order the location numbers them.
const HAIR: u8 = 6;

/// `colors` with the changes to `part` made, in order.
pub(super) fn colors_of(part: Part, colors: Colors, changes: &[ColorChange]) -> Colors {
    changes
        .iter()
        .filter(|change| change.part == part)
        .fold(colors, |colors, change| {
            set(colors, change.color, change.gradient)
        })
}

/// `colors` with colour `color` (metal, minor, major, skin, leather, armor,
/// hair) set to `gradient`.
fn set(colors: Colors, color: u8, gradient: u8) -> Colors {
    match color {
        0 => Colors {
            metal: gradient,
            ..colors
        },
        1 => Colors {
            minor: gradient,
            ..colors
        },
        2 => Colors {
            major: gradient,
            ..colors
        },
        3 => Colors {
            skin: gradient,
            ..colors
        },
        4 => Colors {
            leather: gradient,
            ..colors
        },
        5 => Colors {
            armor: gradient,
            ..colors
        },
        6 => Colors {
            hair: gradient,
            ..colors
        },
        _ => colors,
    }
}

#[cfg(test)]
mod tests {
    use infinitier_itm_resource::{Resistance, SaveType, TimingMode};

    use super::*;

    fn effect(opcode: u16, gradient: u32, location: u32) -> ItmEffect {
        ItmEffect {
            opcode,
            target_type: 1,
            power: 0,
            parameter1: gradient,
            parameter2: location,
            timing_mode: TimingMode::InstantWhileEquipped,
            resistance: Resistance::empty(),
            duration: 0,
            probability1: 100,
            probability2: 0,
            resource: String::new(),
            dice_thrown: 0,
            dice_sides: 0,
            save_type: SaveType::empty(),
            save_bonus: 0,
            trailing_extra: 0,
        }
    }

    fn change(part: Part, color: u8, gradient: u8) -> ColorChange {
        ColorChange {
            part,
            color,
            gradient,
        }
    }

    #[test]
    fn the_location_names_the_part_and_the_colour() {
        // Keldorn's red helmet, Minsc's red armour, a sword's blade.
        assert_eq!(
            effect_changes(&effect(7, 227, 0x30)),
            [change(Part::Helmet, 0, 227)]
        );
        assert_eq!(
            effect_changes(&effect(7, 210, 0x05)),
            [change(Part::Body, 5, 210)]
        );
        assert_eq!(
            effect_changes(&effect(7, 249, 0x14)),
            [change(Part::Weapon, 4, 249)]
        );
        assert_eq!(
            effect_changes(&effect(7, 123, 0x22)),
            [change(Part::OffHand, 2, 123)]
        );
    }

    #[test]
    fn a_location_of_all_parts_changes_each() {
        let changes = effect_changes(&effect(7, 9, 0xF6));
        assert_eq!(
            changes,
            ALL_PARTS.map(|part| change(part, HAIR, 9)).to_vec()
        );
    }

    #[test]
    fn only_set_colour_effects_at_known_locations_count() {
        // A glow, a colour past hair, a part past the helmet.
        assert!(effect_changes(&effect(8, 0, 0x15)).is_empty());
        assert!(effect_changes(&effect(7, 1, 0x07)).is_empty());
        assert!(effect_changes(&effect(7, 1, 0x45)).is_empty());
        assert!(effect_changes(&effect(7, 1, 0x500015)).is_empty());
    }

    #[test]
    fn a_part_takes_only_its_own_changes_the_last_winning() {
        let colors = Colors {
            armor: 1,
            ..Colors::default()
        };
        let changes = [
            change(Part::Body, 5, 210),
            change(Part::Helmet, 5, 88),
            change(Part::Body, 5, 211),
        ];
        assert_eq!(colors_of(Part::Body, colors, &changes).armor, 211);
        assert_eq!(colors_of(Part::Helmet, colors, &changes).armor, 88);
        assert_eq!(colors_of(Part::Weapon, colors, &changes), colors);
    }

    #[test]
    fn every_colour_can_be_set() {
        let set_all = (0..=HAIR).fold(Colors::default(), |colors, color| {
            set(colors, color, color + 1)
        });
        assert_eq!(
            set_all,
            Colors {
                metal: 1,
                minor: 2,
                major: 3,
                skin: 4,
                leather: 5,
                armor: 6,
                hair: 7,
            }
        );
    }
}
