//! The fields every effect record has, wherever it's stored (an item's or a
//! spell's feature blocks, a creature's effects): when it applies and for how
//! long, whether it can be dispelled or resisted, and the saving throws
//! against it. Values follow IESDP (EFF V1 / V2).

use num_enum::{FromPrimitive, IntoPrimitive};

/// When an effect applies and how long it lasts (IESDP "timing mode").
///
/// The names are the BG / IWD / EE meanings. PST names 5 to 10
/// differently (IESDP ITM V1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum TimingMode {
    /// 0: at once, for its duration in seconds.
    InstantLimited = 0,
    /// 1: at once, until removed.
    InstantPermanent = 1,
    /// 2: at once, while its item is equipped.
    InstantWhileEquipped = 2,
    /// 3: after its duration, then for a limited time.
    DelayLimited = 3,
    /// 4: after its duration, then until removed.
    DelayPermanent = 4,
    /// 5: after its duration, then while its item is equipped.
    DelayWhileEquipped = 5,
    /// 6: for a limited time once its duration (in ticks) is over.
    LimitedAfterDuration = 6,
    /// 7: until removed once its duration (in ticks) is over.
    PermanentAfterDuration = 7,
    /// 8: while equipped once its duration (in ticks) is over.
    EquippedAfterDuration = 8,
    /// 9: at once, and still after death.
    PermanentAfterDeath = 9,
    /// 10 (EE): as [`Self::InstantLimited`], its duration in ticks.
    InstantLimitedTicks = 10,
    /// 4096: its duration is an absolute expiry time, in ticks.
    AbsoluteDuration = 4096,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u32),
}

bitflags::bitflags! {
    /// Whether an effect can be dispelled and whether magic resistance
    /// stops it (IESDP "dispel / resistance"). Unnamed bits are kept as
    /// read.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Resistance: u32 {
        /// Bit 0: can be dispelled, and magic resistance applies.
        const Dispellable = 1 << 0;
        /// Bit 1: ignores magic resistance (with bit 0).
        const BypassResistance = 1 << 1;
        /// Bit 2 (EE): bypasses spell turning, reflection, deflection and
        /// spell traps (opcodes 199, 200, 201, 259).
        const BypassSpellProtections = 1 << 2;
        /// Bit 3 (EE): applied by targeting oneself.
        const SelfTargeted = 1 << 3;
        /// Bit 31 (EE): applied by an identified item.
        const AppliedByItem = 1 << 31;
    }
}

bitflags::bitflags! {
    /// The saving throws against an effect (IESDP "saving throw type"); with
    /// several, the target rolls its best. Unnamed bits (5 to 9, the EE's
    /// copies of 0 to 4 for opcode 322; 26, ToBEx's) are kept as read.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SaveType: u32 {
        const Spells = 1 << 0;
        const Breath = 1 << 1;
        /// Paralysis, poison and death.
        const Death = 1 << 2;
        const Wands = 1 << 3;
        /// Petrification and polymorph.
        const Polymorph = 1 << 4;
        /// Bit 10 (EE): a line area projectile spares its end target.
        const IgnorePrimaryTarget = 1 << 10;
        /// Bit 11 (EE): a line area projectile spares bystanders.
        const IgnoreSecondaryTarget = 1 << 11;
        /// Bit 24 (EE, ToBEx): bypasses mirror images.
        const BypassMirrorImage = 1 << 24;
        /// Bit 25: ignores the difficulty setting (EE), limits stacking
        /// (ToBEx).
        const IgnoreDifficulty = 1 << 25;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_modes_keep_their_values() {
        assert_eq!(TimingMode::from(2), TimingMode::InstantWhileEquipped);
        assert_eq!(TimingMode::from(4096), TimingMode::AbsoluteDuration);
        assert_eq!(TimingMode::from(11), TimingMode::Unknown(11));
        for raw in (0..=11).chain([4096, 4097, u32::MAX]) {
            assert_eq!(u32::from(TimingMode::from(raw)), raw);
        }
    }

    #[test]
    fn unnamed_bits_are_kept() {
        let resistance = Resistance::from_bits_retain(0b1_0001 | 1 << 31);
        assert!(resistance.contains(Resistance::Dispellable | Resistance::AppliedByItem));
        assert_eq!(resistance.bits(), 0b1_0001 | 1 << 31);
        let save = SaveType::from_bits_retain(1 << 5 | 1 << 2);
        assert!(save.contains(SaveType::Death));
        assert_eq!(save.bits(), 1 << 5 | 1 << 2);
    }
}
