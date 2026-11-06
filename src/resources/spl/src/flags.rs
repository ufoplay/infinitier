//! A spell header's numbered values and bitfields, as IESDP lists them
//! (SPL V1; SPL V2, IWD2's, shares them). Unnamed values and bits are kept
//! as read.

use num_enum::{FromPrimitive, IntoPrimitive};

/// The kind of spell (header 0x1C). Not numbered like a creature's
/// spellbooks, which count priest 0, wizard 1, innate 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum SpellType {
    Special = 0,
    Wizard = 1,
    Priest = 2,
    Psionic = 3,
    Innate = 4,
    BardSong = 5,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u16),
}

bitflags::bitflags! {
    /// A spell's flags (header 0x18).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SplFlags: u32 {
        /// (EE) Casting it breaks sanctuary and invisibility.
        const BreaksSanctuaryInvisibility = 1 << 9;
        const Hostile = 1 << 10;
        /// Needs no line of sight.
        const NoLineOfSight = 1 << 11;
        const AllowSpotting = 1 << 12;
        const OutdoorsOnly = 1 << 13;
        /// Ignores dead magic and wild surges.
        const IgnoreDeadMagic = 1 << 14;
        /// Ignores wild surges (triggers, contingencies).
        const IgnoreWildSurge = 1 << 15;
        /// Can't be cast in combat.
        const NotInCombat = 1 << 16;
        /// (EE, ToBEx) Can target invisible creatures.
        const CanTargetInvisible = 1 << 24;
        /// (EE, ToBEx) Can be cast while silenced.
        const CastableSilenced = 1 << 25;
    }
}

bitflags::bitflags! {
    /// Who can't cast a spell (header 0x1E): priests of an alignment (BG2,
    /// HoW), mages of a school. Alignment and school bits don't combine.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ExclusionFlags: u32 {
        const ChaoticPriests = 1 << 0;
        const EvilPriests = 1 << 1;
        const GoodPriests = 1 << 2;
        /// Priests neutral on the good–evil axis.
        const NeutralGoodEvilPriests = 1 << 3;
        const LawfulPriests = 1 << 4;
        /// Priests neutral on the law–chaos axis.
        const NeutralLawChaosPriests = 1 << 5;
        const Abjurers = 1 << 6;
        const Conjurers = 1 << 7;
        const Diviners = 1 << 8;
        const Enchanters = 1 << 9;
        const Illusionists = 1 << 10;
        const Invokers = 1 << 11;
        const Necromancers = 1 << 12;
        const Transmuters = 1 << 13;
        /// (BG2) Wild magic: generalists can't cast it.
        const Generalists = 1 << 14;
        const ClericsPaladins = 1 << 30;
        const DruidsRangers = 1 << 31;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_types_keep_their_values() {
        assert_eq!(SpellType::from(2), SpellType::Priest);
        assert_eq!(SpellType::from(6), SpellType::Unknown(6));
        assert_eq!(u16::from(SpellType::BardSong), 5);
    }

    #[test]
    fn unnamed_bits_are_kept() {
        let flags = SplFlags::from_bits_retain(1 << 0 | 1 << 10);
        assert!(flags.contains(SplFlags::Hostile));
        assert_eq!(flags.bits(), 1 << 0 | 1 << 10);
    }
}
