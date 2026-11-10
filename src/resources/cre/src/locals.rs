//! Editing a creature's local script variables (`LOCALS` scope): each is an
//! `op187` effect, its name and value (see [`Cre::local_variables`]). Only
//! V2 effects can hold one: a classic V1 record has no room for the name.
//! A variable is addressed by its place among them, in
//! [`Cre::local_variables`] order.

use crate::{Cre, EffectList, EffectV2, LocalVariable};

/// Whether `effect` is a local variable.
fn is_local(effect: &EffectV2) -> bool {
    match effect {
        EffectV2::LocalVariable(_) => true,
        EffectV2::Effect(e) => e.opcode == EffectV2::LOCAL_VARIABLE_OPCODE,
        EffectV2::Raw(_) | EffectV2::Proficiency(_) => false,
    }
}

impl Cre {
    /// The V2 effects, which local variables are among; `None` for classic
    /// V1 ones.
    fn local_effects_mut(&mut self) -> Option<&mut Vec<EffectV2>> {
        match self.effect_list_mut() {
            EffectList::V2(list) => Some(list),
            EffectList::V1(_) => None,
        }
    }

    /// Whether the creature can hold local variables: its effects are V2.
    pub fn has_local_variables(&self) -> bool {
        matches!(self.effect_list(), EffectList::V2(_))
    }

    /// Add the local variable `name` with `value`, after the others.
    /// Returns whether it was added (see [`Self::has_local_variables`]).
    pub fn add_local_variable(&mut self, name: &str, value: i32) -> bool {
        let Some(list) = self.local_effects_mut() else {
            return false;
        };
        list.push(EffectV2::LocalVariable(LocalVariable {
            name: name.to_owned(),
            value,
        }));
        true
    }

    /// Set the `index`-th local variable's name and value, in place.
    /// Returns whether there was one.
    pub fn set_local_variable(&mut self, index: usize, name: &str, value: i32) -> bool {
        let Some(effect) = self
            .local_effects_mut()
            .and_then(|list| list.iter_mut().filter(|effect| is_local(effect)).nth(index))
        else {
            return false;
        };
        match effect {
            EffectV2::LocalVariable(local) => {
                local.name = name.to_owned();
                local.value = value;
            }
            EffectV2::Effect(e) => {
                e.variable = name.to_owned();
                e.param1 = u32::from_le_bytes(value.to_le_bytes());
            }
            EffectV2::Raw(_) | EffectV2::Proficiency(_) => return false,
        }
        true
    }

    /// Remove the `index`-th local variable. Returns whether there was one.
    pub fn remove_local_variable(&mut self, index: usize) -> bool {
        let Some(list) = self.local_effects_mut() else {
            return false;
        };
        let position = list
            .iter()
            .enumerate()
            .filter(|(_, effect)| is_local(effect))
            .nth(index)
            .map(|(position, _)| position);
        match position {
            Some(position) => {
                list.remove(position);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{import_fixture, round_trip};

    fn locals(cre: &Cre) -> Vec<(String, i32)> {
        cre.local_variables()
            .map(|local| (local.name, local.value))
            .collect()
    }

    #[test]
    fn a_local_is_added_last_set_in_place_and_removed() {
        // V2 effects.
        let mut cre = import_fixture("v1_0/IMOEN_DUAL.cre");
        let before = locals(&cre);
        assert!(cre.add_local_variable("TALKED", 3));
        assert!(cre.add_local_variable("MET_ELMINSTER", -1));
        let rt = round_trip(&cre);
        let added = locals(&rt);
        assert_eq!(added.len(), before.len() + 2);
        assert_eq!(added[before.len()], ("TALKED".to_owned(), 3));

        let mut rt = rt;
        let last = added.len() - 1;
        assert!(rt.set_local_variable(last, "MET_DRIZZT", 7));
        assert!(rt.remove_local_variable(before.len()));
        let rt = round_trip(&rt);
        let mut expected = before.clone();
        expected.push(("MET_DRIZZT".to_owned(), 7));
        assert_eq!(locals(&rt), expected);
        assert!(!rt.clone().remove_local_variable(expected.len()));
    }

    #[test]
    fn classic_effects_hold_no_locals() {
        let mut cre = import_fixture("v9_0/BARBWAR2.cre");
        assert!(!cre.has_local_variables());
        assert!(!cre.add_local_variable("TALKED", 1));
    }
}
