//! Permanent effects an editor keeps on a creature for what its CRE has no
//! field for: a bonus to the movement rate (opcode 126, EEKeeper's
//! "Movement") and "Can Use Any Item" (opcode 302). Each is an effect on
//! the creature itself, permanent and not dispelled ([`Effect::permanent`]),
//! added first in the effect list, where EEKeeper adds the movement bonus.

use crate::{Cre, Effect, EffectList, EffectV1, EffectV1Body, EffectV2, TimingMode};

/// Opcode of the effect that changes the movement rate.
const MOVEMENT_RATE_OPCODE: u16 = 126;

/// Opcode of "Can Use Any Item" in BG2 and the Enhanced Editions (IWD2 has
/// another effect under this number; the classic BG, IWD and PST, none).
/// With parameter 2 set, the creature ignores the class, race, alignment
/// and kit restrictions of the items it uses. The bard's and thief's
/// high-level ability Use Any Item (`SPCL915`) applies it permanently, with
/// parameter 2 = 1.
const CAN_USE_ANY_ITEM_OPCODE: u16 = 302;

/// What an effect is recognised by, read from either record format.
struct Key {
    opcode: u32,
    timing: TimingMode,
    param1: u32,
    param2: u32,
}

/// An effect record, of either format.
trait Record: Sized {
    /// Its [`Key`]; `None` for a record kept as raw bytes, or a
    /// proficiency / local variable (never one of these effects).
    fn key(&self) -> Option<Key>;
    fn param1_mut(&mut self) -> Option<&mut u32>;
    /// [`Effect::permanent`] in this format.
    fn permanent(opcode: u16, param1: u32, param2: u32) -> Self;
}

impl Record for EffectV1 {
    fn key(&self) -> Option<Key> {
        match self {
            EffectV1::Effect(e) => Some(Key {
                opcode: e.opcode.into(),
                timing: e.timing_mode,
                param1: e.param1,
                param2: e.param2,
            }),
            EffectV1::Raw(_) | EffectV1::Proficiency(_) => None,
        }
    }

    fn param1_mut(&mut self) -> Option<&mut u32> {
        match self {
            EffectV1::Effect(e) => Some(&mut e.param1),
            EffectV1::Raw(_) | EffectV1::Proficiency(_) => None,
        }
    }

    fn permanent(opcode: u16, param1: u32, param2: u32) -> Self {
        EffectV1::Effect(EffectV1Body::permanent(opcode, param1, param2))
    }
}

impl Record for EffectV2 {
    fn key(&self) -> Option<Key> {
        match self {
            EffectV2::Effect(e) => Some(Key {
                opcode: e.opcode,
                timing: e.timing_mode,
                param1: e.param1,
                param2: e.param2,
            }),
            EffectV2::Raw(_) | EffectV2::LocalVariable(_) | EffectV2::Proficiency(_) => None,
        }
    }

    fn param1_mut(&mut self) -> Option<&mut u32> {
        match self {
            EffectV2::Effect(e) => Some(&mut e.param1),
            EffectV2::Raw(_) | EffectV2::LocalVariable(_) | EffectV2::Proficiency(_) => None,
        }
    }

    fn permanent(opcode: u16, param1: u32, param2: u32) -> Self {
        EffectV2::Effect(Box::new(Effect::permanent(opcode.into(), param1, param2)))
    }
}

/// The movement bonus: a permanent (timing 9) opcode-126 effect that adds
/// to the rate (parameter 2 = 0).
fn is_movement_bonus(key: &Key) -> bool {
    key.opcode == u32::from(MOVEMENT_RATE_OPCODE)
        && key.timing == TimingMode::PermanentAfterDeath
        && key.param2 == 0
}

/// Can Use Any Item, the creature's own: a permanent (timing 1 or 9)
/// opcode-302 effect with parameter 2 set. One from an equipped item
/// (timing 2) comes and goes with the item.
fn is_use_any_item(key: &Key) -> bool {
    key.opcode == u32::from(CAN_USE_ANY_ITEM_OPCODE)
        && matches!(
            key.timing,
            TimingMode::InstantPermanent | TimingMode::PermanentAfterDeath
        )
        && key.param2 != 0
}

/// Whether `is` recognises `effect`.
fn recognised<T: Record>(effect: &T, is: fn(&Key) -> bool) -> bool {
    effect.key().is_some_and(|key| is(&key))
}

/// The key of the first effect of `list` that `is` recognises.
fn first<T: Record>(list: &[T], is: fn(&Key) -> bool) -> Option<Key> {
    list.iter().filter_map(Record::key).find(is)
}

/// Remove every effect of `list` that `is` recognises.
fn remove_all<T: Record>(list: &mut Vec<T>, is: fn(&Key) -> bool) {
    list.retain(|effect| !recognised(effect, is));
}

fn set_movement_bonus<T: Record>(list: &mut Vec<T>, value: u32) {
    if value == 0 {
        remove_all(list, is_movement_bonus);
        return;
    }
    let existing = list
        .iter_mut()
        .find(|effect| recognised(*effect, is_movement_bonus))
        .and_then(Record::param1_mut);
    match existing {
        Some(param1) => *param1 = value,
        None => list.insert(0, T::permanent(MOVEMENT_RATE_OPCODE, value, 0)),
    }
}

fn set_can_use_any_item<T: Record>(list: &mut Vec<T>, on: bool) {
    if !on {
        remove_all(list, is_use_any_item);
    } else if first(list, is_use_any_item).is_none() {
        list.insert(0, T::permanent(CAN_USE_ANY_ITEM_OPCODE, 0, 1));
    }
}

impl Cre {
    /// The key of the creature's first effect `is` recognises.
    fn first_effect(&self, is: fn(&Key) -> bool) -> Option<Key> {
        match self.effect_list() {
            EffectList::V1(list) => first(list, is),
            EffectList::V2(list) => first(list, is),
        }
    }

    /// The bonus to the creature's movement rate an editor keeps on it
    /// (EEKeeper's "Movement"): parameter 1 of its first permanent
    /// (timing 9) opcode-126 effect that adds to the rate (parameter 2 =
    /// 0). 0 without one. The engines take the base rate from the
    /// creature's animation, not its CRE, so an effect is where it lives.
    pub fn movement_bonus(&self) -> u32 {
        self.first_effect(is_movement_bonus)
            .map_or(0, |key| key.param1)
    }

    /// Set the movement bonus (see [`Self::movement_bonus`]): its effect is
    /// updated in place, or added first in the list (where EEKeeper adds
    /// it) when there's none; 0 removes it.
    pub fn set_movement_bonus(&mut self, value: u32) {
        match self.effect_list_mut() {
            EffectList::V1(list) => set_movement_bonus(list, value),
            EffectList::V2(list) => set_movement_bonus(list, value),
        }
    }

    /// Whether the creature can use any item, ignoring the class, race,
    /// alignment and kit restrictions of items, by an effect of its own: a
    /// permanent "Can Use Any Item" (opcode 302, BG2 and the Enhanced
    /// Editions) with parameter 2 set, as the high-level ability Use Any
    /// Item leaves, or an editor adds.
    pub fn can_use_any_item(&self) -> bool {
        self.first_effect(is_use_any_item).is_some()
    }

    /// Give the creature Can Use Any Item, adding the effect first in the
    /// list when it has none; or take it away, removing every such effect
    /// (the high-level ability's too). One from an equipped item stays.
    pub fn set_can_use_any_item(&mut self, on: bool) {
        match self.effect_list_mut() {
            EffectList::V1(list) => set_can_use_any_item(list, on),
            EffectList::V2(list) => set_can_use_any_item(list, on),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{import_fixture, round_trip};

    /// A fixture of each CRE version: V1 effects in V1.0 and V1.2, V2 ones
    /// in V9.0 and V2.2.
    const FIXTURES: [&str; 4] = [
        "v1_0/IMOEN_DUAL.cre",
        "v1_2/THIEF3.cre",
        "v9_0/BARBWAR2.cre",
        "v2_2/52SERSA.cre",
    ];

    fn first_opcode(cre: &Cre) -> Option<u32> {
        match cre.effect_list() {
            EffectList::V1(list) => list.first().map(EffectV1::opcode),
            EffectList::V2(list) => list.first().map(EffectV2::opcode),
        }
    }

    #[test]
    fn the_movement_bonus_is_added_first_updated_and_removed_in_every_version() {
        for fixture in FIXTURES {
            let mut cre = import_fixture(fixture);
            let count = cre.effect_list().len();
            assert_eq!(cre.movement_bonus(), 0, "{fixture}");

            cre.set_movement_bonus(12);
            let rt = round_trip(&cre);
            assert_eq!(rt.movement_bonus(), 12, "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");
            assert_eq!(first_opcode(&rt), Some(126), "{fixture}");

            // The same effect, updated.
            cre.set_movement_bonus(50);
            let rt = round_trip(&cre);
            assert_eq!(rt.movement_bonus(), 50, "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");

            cre.set_movement_bonus(0);
            let rt = round_trip(&cre);
            assert_eq!(rt.movement_bonus(), 0, "{fixture}");
            assert_eq!(rt.effect_list().len(), count, "{fixture}");
        }
    }

    #[test]
    fn only_a_permanent_increment_is_the_movement_bonus() {
        let bonus = || EffectV2::permanent(MOVEMENT_RATE_OPCODE, 3, 0);
        let mut haste = bonus();
        let mut set = bonus();
        if let (EffectV2::Effect(haste), EffectV2::Effect(set)) = (&mut haste, &mut set) {
            haste.timing_mode = TimingMode::InstantLimited;
            set.param2 = 1;
        }
        assert!(!recognised(&haste, is_movement_bonus));
        assert!(!recognised(&set, is_movement_bonus));
        assert!(recognised(&bonus(), is_movement_bonus));
    }

    #[test]
    fn use_any_item_is_added_first_once_and_removed_in_every_version() {
        for fixture in FIXTURES {
            let mut cre = import_fixture(fixture);
            let count = cre.effect_list().len();
            assert!(!cre.can_use_any_item(), "{fixture}");

            cre.set_can_use_any_item(true);
            // Already on: nothing more.
            cre.set_can_use_any_item(true);
            let rt = round_trip(&cre);
            assert!(rt.can_use_any_item(), "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");
            assert_eq!(first_opcode(&rt), Some(302), "{fixture}");

            cre.set_can_use_any_item(false);
            let rt = round_trip(&cre);
            assert!(!rt.can_use_any_item(), "{fixture}");
            assert_eq!(rt.effect_list().len(), count, "{fixture}");
        }
    }

    #[test]
    fn use_any_item_is_the_creatures_own_and_on() {
        let with = |timing, param2| {
            let mut effect = EffectV2::permanent(CAN_USE_ANY_ITEM_OPCODE, 0, param2);
            if let EffectV2::Effect(e) = &mut effect {
                e.timing_mode = timing;
            }
            effect
        };
        // The high-level ability's, and one applied "permanent until death".
        assert!(recognised(
            &with(TimingMode::PermanentAfterDeath, 1),
            is_use_any_item
        ));
        assert!(recognised(
            &with(TimingMode::InstantPermanent, 1),
            is_use_any_item
        ));
        // An equipped item's; and one with parameter 2 unset, which does nothing.
        assert!(!recognised(
            &with(TimingMode::InstantWhileEquipped, 1),
            is_use_any_item
        ));
        assert!(!recognised(
            &with(TimingMode::PermanentAfterDeath, 0),
            is_use_any_item
        ));

        // Turning it off leaves the item's.
        let mut list = vec![
            with(TimingMode::InstantWhileEquipped, 1),
            with(TimingMode::PermanentAfterDeath, 1),
        ];
        set_can_use_any_item(&mut list, false);
        assert_eq!(list.len(), 1);
        assert!(first(&list, is_use_any_item).is_none());
    }
}
