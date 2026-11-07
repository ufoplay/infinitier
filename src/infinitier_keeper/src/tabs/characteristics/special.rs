//! The Characteristics tab's Special Edits card: shortcuts to features
//! players often want that the creature record has no field for. Each is a
//! permanent effect on the creature (see [`PermanentEffect`]): Movement a
//! value, the rest on or off. Ticking one adds what's missing of it;
//! unticking removes it all, whatever added it.

use infinitier_core::resource::cre::{Cre, Matches, PermanentEffect};
use infinitier_core::resource::{Engine, Game};

use super::kit_features;
use crate::components::field_card::{button_row, card, editable_check_row, number_row, tooltip};
use crate::state::Model;
use crate::tabs::effects::type_text;
use crate::{FieldCard, FieldRow};

/// A Special Edits row. Its id is [`ID_BASE`] plus its place in
/// [`Field::ALL`], apart from the Identity card's ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Movement,
    Infravision,
    Regeneration,
    UseAnyItem,
    Backstab,
    Charm,
    DeathMagic,
    Disease,
    Fear,
    HoldAndSlow,
    LevelDrain,
    Petrification,
    Poison,
    TimeStop,
    NormalWeapons,
    MagicWeapons1,
    MagicWeapons2,
}

/// The first id of the card's rows.
const ID_BASE: i32 = 1000;

const TITLE: &str = "Special Edits";

const TOOLTIP: &str = "Shortcuts to features players often want, which the character record \
has no field for. Each value is worked out from something else, such as an effect on the \
character, and editing it changes that.";

/// What turning an effect off does, in every checkbox's tooltip.
const TURNING_OFF: &str = "Turning it off removes every such effect, whatever added it; an \
equipped item's stays.";

/// The weapon immunities' effect, in their tooltips.
const WEAPONS: &str = "\"Protection from Weapons\"";

/// What the weapon immunities share, in their tooltips.
const EACH_LEVEL: &str = "Each is its own effect, and covers its own weapons only: the \
game's creatures that only +3 weapons can hurt have all three.";

/// The Kit Features button's row id, after the fields'.
const KIT_FEATURES_ID: i32 = ID_BASE + 100;

const KIT_FEATURES_TOOLTIP: &str = "Add or remove what a class or kit gives a character \
from one level to another, as the game does when it levels up: the effects of the spells it \
applies, and its innate abilities.";

/// The largest movement bonus, as EEKeeper allows.
const MOVEMENT_MAX: i32 = 50;

/// A bonus to the movement rate: opcode 126, added to the rate
/// (parameter 2 = 0), EEKeeper's "Movement".
const MOVEMENT: PermanentEffect = PermanentEffect {
    opcode: 126,
    param1: 0,
    param2: 0,
    matches: Matches::SameParameter2,
};

/// An effect that sets a stat to its parameter 2: on with 1.
const fn stat_set(opcode: u16) -> PermanentEffect {
    PermanentEffect {
        opcode,
        param1: 0,
        param2: 1,
        matches: Matches::NonZeroParameter2,
    }
}

/// "Protection from opcode" (opcode 101) against `opcode`.
const fn protection(opcode: u32) -> PermanentEffect {
    PermanentEffect {
        opcode: 101,
        param1: 0,
        param2: opcode,
        matches: Matches::SameParameter2,
    }
}

/// Infravision (opcode 63), which takes no parameters.
const INFRAVISION: [PermanentEffect; 1] = [PermanentEffect {
    opcode: 63,
    param1: 0,
    param2: 0,
    matches: Matches::Any,
}];

/// Regeneration (opcode 98): 1 HP every 6 seconds (parameter 2 = 3, 1 HP
/// every parameter-1 seconds), as the Ring of Regeneration (`RING31`). Any
/// rate counts as on.
const REGENERATION: [PermanentEffect; 1] = [PermanentEffect {
    opcode: 98,
    param1: 6,
    param2: 3,
    matches: Matches::Any,
}];

const USE_ANY_ITEM: [PermanentEffect; 1] = [stat_set(302)];
const BACKSTAB: [PermanentEffect; 1] = [stat_set(292)];
const TIME_STOP: [PermanentEffect; 1] = [stat_set(310)];
// The protections of the game's own spells and items (BG2EE), named in the
// tooltips.
const CHARM: [PermanentEffect; 1] = [protection(5)];
const DEATH_MAGIC: [PermanentEffect; 4] = [
    protection(13),
    protection(55),
    protection(209),
    protection(238),
];
const DISEASE: [PermanentEffect; 1] = [protection(78)];
const FEAR: [PermanentEffect; 3] = [protection(23), protection(24), protection(106)];
/// Free Action's, but haste (16) and the movement rate (126): permanently,
/// those would stop the character's own haste and Movement bonus too.
const HOLD_AND_SLOW: [PermanentEffect; 7] = [
    protection(40),
    protection(109),
    protection(154),
    protection(157),
    protection(158),
    protection(175),
    protection(185),
];
const LEVEL_DRAIN: [PermanentEffect; 1] = [protection(216)];
const PETRIFICATION: [PermanentEffect; 1] = [protection(134)];
const POISON: [PermanentEffect; 1] = [protection(25)];

/// "Protection from (melee) weapons" (opcode 120) of type `kind`, at
/// enchantment `level` for type 0.
const fn weapons(kind: u32, level: u32) -> PermanentEffect {
    PermanentEffect {
        opcode: 120,
        param1: level,
        param2: kind,
        matches: Matches::SameParameters,
    }
}

// The game's creatures that only +3 weapons hurt carry all three (e.g. the
// siren's `CSIREN`): each enchantment level is its own effect.
/// Type 2: non-magical weapons.
const NORMAL_WEAPONS: [PermanentEffect; 1] = [weapons(2, 0)];
/// Type 0: weapons of enchantment +1.
const MAGIC_WEAPONS_1: [PermanentEffect; 1] = [weapons(0, 1)];
/// Type 0: weapons of enchantment +2.
const MAGIC_WEAPONS_2: [PermanentEffect; 1] = [weapons(0, 2)];

impl Field {
    const ALL: [Self; 17] = [
        Self::Movement,
        Self::Infravision,
        Self::Regeneration,
        Self::UseAnyItem,
        Self::Backstab,
        Self::Charm,
        Self::DeathMagic,
        Self::Disease,
        Self::Fear,
        Self::HoldAndSlow,
        Self::LevelDrain,
        Self::Petrification,
        Self::Poison,
        Self::TimeStop,
        Self::NormalWeapons,
        Self::MagicWeapons1,
        Self::MagicWeapons2,
    ];

    fn id(self) -> i32 {
        Self::ALL
            .iter()
            .position(|&field| field == self)
            .and_then(|index| i32::try_from(index).ok())
            .map_or(-1, |index| ID_BASE + index)
    }

    fn from_id(id: i32) -> Option<Self> {
        id.checked_sub(ID_BASE)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| Self::ALL.get(index))
            .copied()
    }

    fn label(self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Infravision => "Infravision",
            Self::Regeneration => "Regeneration",
            Self::UseAnyItem => "Use Any Item",
            Self::Backstab => "Immune to Backstab",
            Self::Charm => "Immune to Charm",
            Self::DeathMagic => "Immune to Death Magic",
            Self::Disease => "Immune to Disease",
            Self::Fear => "Immune to Fear",
            Self::HoldAndSlow => "Immune to Hold & Slow",
            Self::LevelDrain => "Immune to Level Drain",
            Self::Petrification => "Immune to Petrification",
            Self::Poison => "Immune to Poison",
            Self::TimeStop => "Immune to Time Stop",
            Self::NormalWeapons => "Immune to Normal Weapons",
            Self::MagicWeapons1 => "Immune to Magic Weapons +1",
            Self::MagicWeapons2 => "Immune to Magic Weapons +2",
        }
    }

    /// The effects it's made of; on when the creature has them all.
    fn effects(self) -> &'static [PermanentEffect] {
        match self {
            Self::Movement => std::slice::from_ref(&MOVEMENT),
            Self::Infravision => &INFRAVISION,
            Self::Regeneration => &REGENERATION,
            Self::UseAnyItem => &USE_ANY_ITEM,
            Self::Backstab => &BACKSTAB,
            Self::Charm => &CHARM,
            Self::DeathMagic => &DEATH_MAGIC,
            Self::Disease => &DISEASE,
            Self::Fear => &FEAR,
            Self::HoldAndSlow => &HOLD_AND_SLOW,
            Self::LevelDrain => &LEVEL_DRAIN,
            Self::Petrification => &PETRIFICATION,
            Self::Poison => &POISON,
            Self::TimeStop => &TIME_STOP,
            Self::NormalWeapons => &NORMAL_WEAPONS,
            Self::MagicWeapons1 => &MAGIC_WEAPONS_1,
            Self::MagicWeapons2 => &MAGIC_WEAPONS_2,
        }
    }

    /// What it is, and does to the character's effects.
    fn tooltip(self) -> String {
        let one = |what: &str, opcode: u16, does: &str| {
            format!(
                "Adds or removes a permanent {what} effect (opcode {opcode}): {does} {TURNING_OFF}"
            )
        };
        let protections = |like: &str| {
            let against: Vec<String> = self
                .effects()
                .iter()
                .map(|effect| type_text(effect.param2))
                .collect();
            format!(
                "Adds or removes permanent \"Protection from Opcode\" effects (opcode 101) \
                 against {}.{like} {TURNING_OFF}",
                against.join(", ")
            )
        };
        match self {
            Self::Movement => format!(
                "A bonus to the character's movement rate, from 0 to {MOVEMENT_MAX} as \
                 EEKeeper allows, kept as a permanent effect that adds to it (opcode 126). \
                 0 removes the effect."
            ),
            Self::Infravision => one(
                "Infravision",
                63,
                "the character sees in the dark. A race that sees in the dark has it without \
                 the effect, so the box can be off for one.",
            ),
            Self::Regeneration => one(
                "Regeneration",
                98,
                "the character heals 1 HP every 6 seconds (a round), as with the Ring of \
                 Regeneration. Any permanent regeneration counts, a kit's too.",
            ),
            Self::UseAnyItem => one(
                "\"Can Use Any Item\"",
                302,
                "the one the bard's and thief's high-level ability Use Any Item grants. With \
                 it, the character can use any item, whatever class, race, alignment or kit \
                 the item is restricted to.",
            ),
            Self::Backstab => one(
                "Backstab protection",
                292,
                "the character can't be backstabbed.",
            ),
            Self::TimeStop => one(
                "Time Stop protection",
                310,
                "the character isn't stopped by Time Stop.",
            ),
            Self::NormalWeapons => one(
                WEAPONS,
                120,
                &format!("non-magical weapons can't hurt the character. {EACH_LEVEL}"),
            ),
            Self::MagicWeapons1 => one(
                WEAPONS,
                120,
                &format!("weapons of enchantment +1 can't hurt the character. {EACH_LEVEL}"),
            ),
            Self::MagicWeapons2 => one(
                WEAPONS,
                120,
                &format!("weapons of enchantment +2 can't hurt the character. {EACH_LEVEL}"),
            ),
            Self::Charm | Self::Disease => protections(""),
            Self::DeathMagic => protections(" As Death Ward."),
            Self::Fear => protections(" As Remove Fear."),
            Self::HoldAndSlow => protections(
                " As Free Action, but for its protection from haste and from movement-rate \
                 effects, which would stop the character's own haste and Movement bonus too.",
            ),
            Self::LevelDrain => protections(" As Negative Plane Protection."),
            Self::Petrification => protections(" As Protection From Petrification."),
            Self::Poison => protections(" As the Periapt of Proof Against Poison."),
        }
    }

    /// Its row for `cre`.
    fn row(self, cre: &Cre) -> FieldRow {
        let row = if self == Self::Movement {
            let bonus = cre.permanent_param1(&MOVEMENT).unwrap_or(0);
            number_row(self.id(), self.label(), bonus.into(), 0, MOVEMENT_MAX)
        } else {
            let on = self
                .effects()
                .iter()
                .all(|effect| cre.has_permanent_effect(effect));
            editable_check_row(self.id(), self.label(), on)
        };
        FieldRow {
            tooltip: tooltip(&self.tooltip()),
            ..row
        }
    }
}

/// The card, in display `column`. Movement is every game's; the other
/// effects' opcodes are BG2's and the Enhanced Editions' (IWD2 has others
/// under some of those numbers; the classic BG, IWD and PST, none).
pub fn card_for(game: Game, cre: &Cre, column: i32) -> FieldCard {
    let bg2_effects = matches!(game.engine(), Engine::Bg2 | Engine::Ee);
    let mut rows: Vec<FieldRow> = Field::ALL
        .iter()
        .filter(|&&field| field == Field::Movement || bg2_effects)
        .map(|field| field.row(cre))
        .collect();
    // The games with kits whose tables give spells (BG2's and the Enhanced
    // Editions').
    if bg2_effects && kit_features::supports(cre) {
        rows.push(FieldRow {
            tooltip: tooltip(KIT_FEATURES_TOOLTIP),
            ..button_row(KIT_FEATURES_ID, "Add/Remove Kit Features…")
        });
    }
    FieldCard {
        tooltip: tooltip(TOOLTIP),
        ..card(TITLE, column, rows)
    }
}

/// Apply an edit of the card's row `id`: Movement's number, or a
/// checkbox's `"1"` / `"0"`; or open the Kit Features dialog. Returns
/// whether `id` is one of the card's.
pub fn commit(model: &mut Model, id: i32, value: &str) -> bool {
    if id == KIT_FEATURES_ID {
        kit_features::open(model);
        return true;
    }
    let Some(field) = Field::from_id(id) else {
        return false;
    };
    let Some(cre) = model.selected_cre_mut() else {
        return true;
    };
    if field == Field::Movement {
        if let Ok(number) = value.trim().parse::<f64>() {
            // In range, so a whole number from 0 to 50.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let bonus = number.round().clamp(0.0, f64::from(MOVEMENT_MAX)) as u32;
            cre.set_permanent_param1(&MOVEMENT, bonus);
        }
    } else {
        for effect in field.effects() {
            cre.set_permanent_effect(effect, value == "1");
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_ids_are_apart_from_the_identity_cards() {
        assert_eq!(Field::Movement.id(), ID_BASE);
        for field in Field::ALL {
            assert_eq!(Field::from_id(field.id()), Some(field));
        }
        assert_eq!(Field::from_id(0), None);
        assert_eq!(Field::from_id(ID_BASE + 17), None);
    }

    #[test]
    fn a_protection_tooltip_names_what_it_protects_from() {
        let text = Field::Poison.tooltip();
        assert!(text.contains("State: Poison [25]"), "{text}");
        assert!(Field::HoldAndSlow.tooltip().contains("State: Hold [185]"));
    }
}
