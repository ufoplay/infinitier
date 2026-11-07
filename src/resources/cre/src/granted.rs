//! What a class or kit grants a creature as it levels up: the effects of a
//! spell applied to it, which name the spell as their source (V2 effects
//! only: a classic V1 effect has no source field), and innate abilities,
//! one memorised use per grant.

use crate::{
    Cre, Effect, EffectList, EffectV2, KnownSpell, SpellMemorizationInfo, SpellType, SubSections,
};

/// Innate abilities' spell level, as the creature keeps it (0-based).
const INNATE_LEVEL: u16 = 0;

impl Cre {
    /// Add `effects` after the creature's others. Returns whether they
    /// were added: a creature whose effects are classic V1 records has no
    /// room for their source, and gets none.
    pub fn add_effects(&mut self, effects: Vec<Effect>) -> bool {
        match self.effect_list_mut() {
            EffectList::V2(list) => {
                list.extend(
                    effects
                        .into_iter()
                        .map(|effect| EffectV2::Effect(Box::new(effect))),
                );
                true
            }
            EffectList::V1(_) => false,
        }
    }

    /// Remove every effect whose source (parent resource) is `source`, a
    /// spell's resref, however many times it was applied. Returns how many
    /// went; none for classic V1 records, which have no source.
    pub fn remove_effects_from(&mut self, source: &str) -> usize {
        let EffectList::V2(list) = self.effect_list_mut() else {
            return 0;
        };
        let before = list.len();
        list.retain(|effect| {
            !matches!(effect, EffectV2::Effect(e) if e.parent_resource.eq_ignore_ascii_case(source))
        });
        before - list.len()
    }

    /// Grant the innate ability `resref` one more use: known, if it wasn't,
    /// and one more memorised copy. The innate slot is made when the
    /// creature has none, and holds at least as many copies as it has.
    /// Returns whether it was granted (only AD&D creatures, not IWD2's).
    pub fn add_innate_use(&mut self, resref: &str) -> bool {
        let resref = resref.to_ascii_uppercase();
        let copies = {
            let SubSections::V1(sub) = &mut self.sub_sections else {
                return false;
            };
            let known = sub.known_spells.iter().any(|k| {
                k.spell_type == SpellType::Innate && k.spell.eq_ignore_ascii_case(&resref)
            });
            if !known {
                sub.known_spells.push(KnownSpell {
                    spell: resref.clone(),
                    level: INNATE_LEVEL,
                    spell_type: SpellType::Innate,
                });
            }
            let slot = innate_slot(&mut sub.spell_memorization_info, sub.memorized_spells.len());
            let info = &sub.spell_memorization_info[slot];
            let start = info.spell_table_index as usize;
            let end = start + info.spell_count as usize;
            sub.memorized_spells.get(start..end).map_or(0, |slot| {
                slot.iter()
                    .filter(|m| m.spell.eq_ignore_ascii_case(&resref))
                    .count()
            })
        };
        let count = u32::try_from(copies + 1).unwrap_or(u32::MAX);
        if !self.set_known_spell_memorized_count(SpellType::Innate, &resref, count) {
            return false;
        }
        // The slot holds as many uses as it has copies, at least.
        if let SubSections::V1(sub) = &mut self.sub_sections
            && let Some(info) = sub
                .spell_memorization_info
                .iter_mut()
                .find(|m| m.spell_type == SpellType::Innate && m.level == INNATE_LEVEL)
        {
            let held = u16::try_from(info.spell_count).unwrap_or(u16::MAX);
            info.num_memorizable_total = info.num_memorizable_total.max(held);
            info.num_memorizable_current = info.num_memorizable_current.max(held);
        }
        true
    }

    /// Take the innate ability `resref` away: every memorised copy, and the
    /// known spell. Returns whether the creature had it.
    pub fn remove_innate(&mut self, resref: &str) -> bool {
        let copies_gone = self.set_known_spell_memorized_count(SpellType::Innate, resref, 0);
        self.remove_known_spell(SpellType::Innate, resref) || copies_gone
    }
}

/// The index of the innate slot in `infos`, made last (its copies starting
/// after the `memorized` ones) when there's none.
fn innate_slot(infos: &mut Vec<SpellMemorizationInfo>, memorized: usize) -> usize {
    if let Some(slot) = infos
        .iter()
        .position(|m| m.spell_type == SpellType::Innate && m.level == INNATE_LEVEL)
    {
        return slot;
    }
    infos.push(SpellMemorizationInfo {
        level: INNATE_LEVEL,
        num_memorizable_total: 0,
        num_memorizable_current: 0,
        spell_type: SpellType::Innate,
        spell_table_index: u32::try_from(memorized).unwrap_or(u32::MAX),
        spell_count: 0,
    });
    infos.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{import_fixture, round_trip};

    fn innate_copies(cre: &Cre, resref: &str) -> usize {
        let SubSections::V1(sub) = &cre.sub_sections else {
            return 0;
        };
        sub.memorized_spells
            .iter()
            .filter(|m| m.spell.eq_ignore_ascii_case(resref))
            .count()
    }

    fn knows(cre: &Cre, resref: &str) -> bool {
        let SubSections::V1(sub) = &cre.sub_sections else {
            return false;
        };
        sub.known_spells
            .iter()
            .any(|k| k.spell_type == SpellType::Innate && k.spell.eq_ignore_ascii_case(resref))
    }

    #[test]
    fn an_innate_gains_a_use_per_grant_and_goes_whole() {
        let mut cre = import_fixture("v1_0/IMOEN_DUAL.cre");
        assert!(!knows(&cre, "SPCL212"));
        assert!(cre.add_innate_use("spcl212"));
        assert!(cre.add_innate_use("SPCL212"));
        let rt = round_trip(&cre);
        assert!(knows(&rt, "SPCL212"));
        assert_eq!(innate_copies(&rt, "SPCL212"), 2);

        let mut rt = rt;
        assert!(rt.remove_innate("SPCL212"));
        let rt = round_trip(&rt);
        assert!(!knows(&rt, "SPCL212"));
        assert_eq!(innate_copies(&rt, "SPCL212"), 0);
    }

    #[test]
    fn a_spells_effects_go_by_their_source() {
        // V2 effects.
        let mut cre = import_fixture("v1_0/IMOEN_DUAL.cre");
        let count = cre.effect_list().len();
        let from = |source: &str| {
            let mut effect = Effect::permanent(101, 0, 5);
            effect.parent_resource = source.to_owned();
            effect
        };
        assert!(cre.add_effects(vec![from("SPCL233B"), from("SPCL233B"), from("SPCL234")]));
        let mut rt = round_trip(&cre);
        assert_eq!(rt.effect_list().len(), count + 3);
        assert_eq!(rt.remove_effects_from("spcl233b"), 2);
        assert_eq!(rt.effect_list().len(), count + 1);
    }

    #[test]
    fn classic_effects_have_no_source() {
        let mut cre = import_fixture("v9_0/BARBWAR2.cre");
        assert!(matches!(cre.effect_list(), EffectList::V1(_)));
        assert!(!cre.add_effects(vec![Effect::permanent(101, 0, 5)]));
        assert_eq!(cre.remove_effects_from("SPCL233B"), 0);
    }
}
