//! An item ability's (extended header's) numbered values and bitfields,
//! as IESDP lists them for ITM V1. PST's meanings, where they differ, are
//! noted. Unnamed values and bits are kept as read.

use num_enum::{FromPrimitive, IntoPrimitive};

/// How an ability attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AttackType {
    None = 0,
    Melee = 1,
    Ranged = 2,
    Magical = 3,
    Launcher = 4,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u8),
}

bitflags::bitflags! {
    /// Whether an ability needs its item identified.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct IdRequired: u8 {
        /// Usable once identified.
        const Identified = 1 << 0;
        /// Usable while unidentified.
        const Unidentified = 1 << 1;
        /// (PST) A friendly ability.
        const Friendly = 1 << 2;
    }
}

/// The damage an ability deals. The combinations deal whichever of the
/// two the target resists less (more, for crushing/slashing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum DamageType {
    None = 0,
    Piercing = 1,
    Crushing = 2,
    Slashing = 3,
    Missile = 4,
    Fist = 5,
    PiercingCrushing = 6,
    PiercingSlashing = 7,
    CrushingSlashing = 8,
    BluntMissile = 9,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u16),
}

/// What happens to an item when its ability's charges run out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ChargeDepletion {
    /// 0: the item remains.
    Remains = 0,
    /// 1: the item vanishes.
    Vanishes = 1,
    /// 2: replaced by its used-up item; in PST, it vanishes silently.
    Replaced = 2,
    /// 3: the item recharges.
    Recharges = 3,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u16),
}

bitflags::bitflags! {
    /// An ability's flags. In ITM V2.0 (IWD2) the upper word is instead an
    /// attack type (normal, bypass armour, keen), kept as read.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct AbilityFlags: u32 {
        const AddStrengthBonus = 1 << 0;
        const Breakable = 1 << 1;
        /// (EE) The strength bonus applies to damage.
        const DamageStrengthBonus = 1 << 2;
        /// (EE) The strength bonus applies to THAC0.
        const ThacoStrengthBonus = 1 << 3;
        /// (EE) Breaks sanctuary and invisibility.
        const BreaksSanctuaryInvisibility = 1 << 9;
        const Hostile = 1 << 10;
        const RechargeAfterResting = 1 << 11;
        /// (ToBEx) Toggles backstabbing.
        const ToggleBackstab = 1 << 25;
        /// (EE, ToBEx) Can't target invisible creatures.
        const CannotTargetInvisible = 1 << 26;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_keep_their_meaning() {
        assert_eq!(AttackType::from(4), AttackType::Launcher);
        assert_eq!(DamageType::from(9), DamageType::BluntMissile);
        assert_eq!(DamageType::from(10), DamageType::Unknown(10));
        assert_eq!(ChargeDepletion::from(2), ChargeDepletion::Replaced);
    }

    #[test]
    fn the_v2_attack_type_word_is_kept() {
        let flags = AbilityFlags::from_bits_retain(2 << 16 | 1 << 10);
        assert!(flags.contains(AbilityFlags::Hostile));
        assert_eq!(flags.bits(), 2 << 16 | 1 << 10);
    }
}
