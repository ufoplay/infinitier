//! The Characteristics tab's Special Edits card: shortcuts to features
//! players often want that the creature record has no field for. Each is
//! worked out from something else, and editing it changes that: Use Any
//! Item is a permanent effect (see [`Cre::can_use_any_item`]).

use infinitier_core::resource::cre::Cre;
use infinitier_core::resource::{Engine, Game};

use crate::FieldCard;
use crate::components::field_card::{card, editable_check_row};
use crate::state::Model;

/// A Special Edits field. Its row's id is [`ID_BASE`] plus its place in
/// [`Field::ALL`], apart from the Identity card's ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    UseAnyItem,
}

/// The first id of the card's rows.
const ID_BASE: i32 = 1000;

const TITLE: &str = "Special Edits";

/// The card's tooltip. A tooltip doesn't wrap: its lines are broken here.
const TOOLTIP: &str = "Shortcuts to features players often want, which the character\n\
record has no field for. Each value is worked out from something else,\n\
such as an effect on the character, and editing it changes that.";

const USE_ANY_ITEM_TOOLTIP: &str = "Adds or removes a permanent \"Can Use Any Item\" effect (opcode 302)\n\
on the character: the one the bard's and thief's high-level ability\n\
Use Any Item grants. With it, the character can use any item, whatever\n\
class, race, alignment or kit the item is restricted to.\n\
Turning it off removes the effect, the ability's too.";

impl Field {
    const ALL: [Self; 1] = [Self::UseAnyItem];

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
}

/// The card, in display `column`; `None` where the game has none of its
/// features: "Can Use Any Item" is BG2's and the Enhanced Editions' (IWD2
/// has another effect under its opcode; the classic BG, IWD and PST, none).
pub fn card_for(game: Game, cre: &Cre, column: i32) -> Option<FieldCard> {
    if !matches!(game.engine(), Engine::Bg2 | Engine::Ee) {
        return None;
    }
    let use_any_item = crate::FieldRow {
        tooltip: USE_ANY_ITEM_TOOLTIP.into(),
        ..editable_check_row(
            Field::UseAnyItem.id(),
            "Use Any Item",
            cre.can_use_any_item(),
        )
    };
    Some(FieldCard {
        tooltip: TOOLTIP.into(),
        ..card(TITLE, column, vec![use_any_item])
    })
}

/// Apply an edit of the card's row `id` (a checkbox's `"1"` / `"0"`).
/// Returns whether `id` is one of the card's.
pub fn commit(model: &mut Model, id: i32, value: &str) -> bool {
    let Some(field) = Field::from_id(id) else {
        return false;
    };
    if let Some(cre) = model.selected_cre_mut() {
        match field {
            Field::UseAnyItem => cre.set_can_use_any_item(value == "1"),
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_ids_are_apart_from_the_identity_cards() {
        assert_eq!(Field::UseAnyItem.id(), ID_BASE);
        assert_eq!(Field::from_id(ID_BASE), Some(Field::UseAnyItem));
        assert_eq!(Field::from_id(0), None);
        assert_eq!(Field::from_id(ID_BASE + 1), None);
    }
}
