//! Permanent effects an editor keeps on a creature for what its CRE has no
//! field for (a bonus to the movement rate, an immunity, infravision…): an
//! effect on the creature itself, permanent and not dispelled
//! ([`Effect::permanent`]), added first in the effect list, where EEKeeper
//! adds its movement bonus. Which effects mean what is the editor's to say,
//! as a [`PermanentEffect`].

use crate::{Cre, Effect, EffectList, EffectV1, EffectV1Body, EffectV2, TimingMode};

/// Which effects of its opcode count as a [`PermanentEffect`], by their
/// parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Matches {
    /// The same parameter 2: a "protection from opcode" is one opcode's.
    SameParameter2,
    /// The same parameters 1 and 2: a weapon immunity by enchantment is one
    /// level's (parameter 2 = 0, parameter 1 the level).
    SameParameters,
    /// Any parameter 2 but 0: an effect setting a stat to its parameter 2
    /// does nothing with 0.
    NonZeroParameter2,
    /// Any: an effect that ignores them, or several that do one thing.
    Any,
}

/// A permanent effect an editor keeps on a creature: its opcode, and the
/// parameters it's added with. The creature has it when it has a permanent
/// (timing 1 or 9) effect of that opcode whose parameters `matches`. One
/// from an equipped item (timing 2) comes and goes with the item, so it
/// doesn't count, and stays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermanentEffect {
    pub opcode: u16,
    pub param1: u32,
    pub param2: u32,
    pub matches: Matches,
}

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
    /// proficiency / local variable (their own tabs edit those).
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

impl PermanentEffect {
    /// Whether `key` is this effect, the creature's own.
    fn is(&self, key: &Key) -> bool {
        let parameters = match self.matches {
            Matches::SameParameter2 => key.param2 == self.param2,
            Matches::SameParameters => key.param1 == self.param1 && key.param2 == self.param2,
            Matches::NonZeroParameter2 => key.param2 != 0,
            Matches::Any => true,
        };
        key.opcode == u32::from(self.opcode)
            && matches!(
                key.timing,
                TimingMode::InstantPermanent | TimingMode::PermanentAfterDeath
            )
            && parameters
    }

    fn recognises<T: Record>(&self, effect: &T) -> bool {
        effect.key().is_some_and(|key| self.is(&key))
    }

    /// The key of the first effect of `list` that is this one.
    fn first<T: Record>(&self, list: &[T]) -> Option<Key> {
        list.iter().filter_map(Record::key).find(|key| self.is(key))
    }

    fn new_record<T: Record>(&self, param1: u32) -> T {
        T::permanent(self.opcode, param1, self.param2)
    }

    fn set<T: Record>(&self, list: &mut Vec<T>, on: bool) {
        if !on {
            list.retain(|effect| !self.recognises(effect));
        } else if self.first(list).is_none() {
            list.insert(0, self.new_record(self.param1));
        }
    }

    fn set_param1<T: Record>(&self, list: &mut Vec<T>, value: u32) {
        if value == 0 {
            self.set(list, false);
            return;
        }
        let existing = list
            .iter_mut()
            .find(|effect| self.recognises(*effect))
            .and_then(Record::param1_mut);
        match existing {
            Some(param1) => *param1 = value,
            None => list.insert(0, self.new_record(value)),
        }
    }
}

impl Cre {
    /// Whether the creature has `effect` of its own (see
    /// [`PermanentEffect`]).
    pub fn has_permanent_effect(&self, effect: &PermanentEffect) -> bool {
        self.permanent_param1(effect).is_some()
    }

    /// Give the creature `effect`, adding it first in the effect list when
    /// it has none; or take it away, removing every one it has (whatever
    /// added it).
    pub fn set_permanent_effect(&mut self, effect: &PermanentEffect, on: bool) {
        match self.effect_list_mut() {
            EffectList::V1(list) => effect.set(list, on),
            EffectList::V2(list) => effect.set(list, on),
        }
    }

    /// Parameter 1 of the creature's first `effect`, the value an effect
    /// like a bonus carries; `None` without one.
    pub fn permanent_param1(&self, effect: &PermanentEffect) -> Option<u32> {
        match self.effect_list() {
            EffectList::V1(list) => effect.first(list),
            EffectList::V2(list) => effect.first(list),
        }
        .map(|key| key.param1)
    }

    /// Keep `effect` on the creature with parameter 1 `value`: the first
    /// one is updated in place, or one added first in the list when there's
    /// none; 0 removes every one.
    pub fn set_permanent_param1(&mut self, effect: &PermanentEffect, value: u32) {
        match self.effect_list_mut() {
            EffectList::V1(list) => effect.set_param1(list, value),
            EffectList::V2(list) => effect.set_param1(list, value),
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

    /// A bonus to the movement rate (opcode 126, added: parameter 2 = 0).
    const MOVEMENT: PermanentEffect = PermanentEffect {
        opcode: 126,
        param1: 0,
        param2: 0,
        matches: Matches::SameParameter2,
    };

    /// Can Use Any Item (opcode 302), a stat set to parameter 2.
    const USE_ANY_ITEM: PermanentEffect = PermanentEffect {
        opcode: 302,
        param1: 0,
        param2: 1,
        matches: Matches::NonZeroParameter2,
    };

    fn protection(opcode: u32) -> PermanentEffect {
        PermanentEffect {
            opcode: 101,
            param1: 0,
            param2: opcode,
            matches: Matches::SameParameter2,
        }
    }

    fn first_opcode(cre: &Cre) -> Option<u32> {
        match cre.effect_list() {
            EffectList::V1(list) => list.first().map(EffectV1::opcode),
            EffectList::V2(list) => list.first().map(EffectV2::opcode),
        }
    }

    /// `effect`, applied with `timing` and parameter 2 `param2`.
    fn applied(effect: &PermanentEffect, timing: TimingMode, param2: u32) -> EffectV2 {
        let mut record = EffectV2::permanent(effect.opcode, effect.param1, param2);
        if let EffectV2::Effect(e) = &mut record {
            e.timing_mode = timing;
        }
        record
    }

    fn counts(effect: &PermanentEffect, record: &EffectV2) -> bool {
        effect.recognises(record)
    }

    #[test]
    fn a_value_is_added_first_updated_and_removed_in_every_version() {
        for fixture in FIXTURES {
            let mut cre = import_fixture(fixture);
            let count = cre.effect_list().len();
            assert_eq!(cre.permanent_param1(&MOVEMENT), None, "{fixture}");

            cre.set_permanent_param1(&MOVEMENT, 12);
            let rt = round_trip(&cre);
            assert_eq!(rt.permanent_param1(&MOVEMENT), Some(12), "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");
            assert_eq!(first_opcode(&rt), Some(126), "{fixture}");

            // The same effect, updated.
            cre.set_permanent_param1(&MOVEMENT, 50);
            let rt = round_trip(&cre);
            assert_eq!(rt.permanent_param1(&MOVEMENT), Some(50), "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");

            cre.set_permanent_param1(&MOVEMENT, 0);
            let rt = round_trip(&cre);
            assert_eq!(rt.permanent_param1(&MOVEMENT), None, "{fixture}");
            assert_eq!(rt.effect_list().len(), count, "{fixture}");
        }
    }

    #[test]
    fn an_effect_is_added_first_once_and_removed_in_every_version() {
        for fixture in FIXTURES {
            let mut cre = import_fixture(fixture);
            let count = cre.effect_list().len();
            assert!(!cre.has_permanent_effect(&USE_ANY_ITEM), "{fixture}");

            cre.set_permanent_effect(&USE_ANY_ITEM, true);
            // Already on: nothing more.
            cre.set_permanent_effect(&USE_ANY_ITEM, true);
            let rt = round_trip(&cre);
            assert!(rt.has_permanent_effect(&USE_ANY_ITEM), "{fixture}");
            assert_eq!(rt.effect_list().len(), count + 1, "{fixture}");
            assert_eq!(first_opcode(&rt), Some(302), "{fixture}");

            cre.set_permanent_effect(&USE_ANY_ITEM, false);
            let rt = round_trip(&cre);
            assert!(!rt.has_permanent_effect(&USE_ANY_ITEM), "{fixture}");
            assert_eq!(rt.effect_list().len(), count, "{fixture}");
        }
    }

    #[test]
    fn only_a_permanent_effect_counts() {
        let permanent = applied(&USE_ANY_ITEM, TimingMode::PermanentAfterDeath, 1);
        let until_death = applied(&USE_ANY_ITEM, TimingMode::InstantPermanent, 1);
        let equipped = applied(&USE_ANY_ITEM, TimingMode::InstantWhileEquipped, 1);
        let for_a_while = applied(&MOVEMENT, TimingMode::InstantLimited, 0);
        assert!(counts(&USE_ANY_ITEM, &permanent));
        assert!(counts(&USE_ANY_ITEM, &until_death));
        assert!(!counts(&USE_ANY_ITEM, &equipped));
        assert!(!counts(&MOVEMENT, &for_a_while));
    }

    #[test]
    fn parameter_2_counts_as_the_effect_says() {
        let permanent = TimingMode::PermanentAfterDeath;
        // A stat set to 0 does nothing.
        assert!(!counts(
            &USE_ANY_ITEM,
            &applied(&USE_ANY_ITEM, permanent, 0)
        ));
        assert!(counts(&USE_ANY_ITEM, &applied(&USE_ANY_ITEM, permanent, 2)));
        // A protection is from one opcode; a movement effect that sets the
        // rate isn't a bonus added to it.
        let poison = protection(25);
        assert!(!counts(&poison, &applied(&poison, permanent, 78)));
        assert!(counts(&poison, &applied(&poison, permanent, 25)));
        assert!(!counts(&MOVEMENT, &applied(&MOVEMENT, permanent, 1)));
    }

    #[test]
    fn a_weapon_immunity_is_one_enchantment_levels() {
        let plus = |level| PermanentEffect {
            opcode: 120,
            param1: level,
            param2: 0,
            matches: Matches::SameParameters,
        };
        let permanent = TimingMode::PermanentAfterDeath;
        let plus_one = applied(&plus(1), permanent, 0);
        assert!(counts(&plus(1), &plus_one));
        assert!(!counts(&plus(2), &plus_one));
        let mut list = vec![plus_one];
        plus(2).set(&mut list, true);
        assert_eq!(list.len(), 2);
        plus(1).set(&mut list, false);
        assert!(plus(1).first(&list).is_none());
        assert!(plus(2).first(&list).is_some());
    }

    #[test]
    fn turning_an_effect_off_leaves_an_items() {
        let mut list = vec![
            applied(&USE_ANY_ITEM, TimingMode::InstantWhileEquipped, 1),
            applied(&USE_ANY_ITEM, TimingMode::PermanentAfterDeath, 1),
        ];
        USE_ANY_ITEM.set(&mut list, false);
        assert_eq!(list.len(), 1);
        assert!(USE_ANY_ITEM.first(&list).is_none());
    }
}
