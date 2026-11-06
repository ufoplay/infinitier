//! Fields an item's and a spell's abilities (their extended headers) share.
//! Values follow IESDP (ITM V1, SPL V1).

use num_enum::{FromPrimitive, IntoPrimitive};

/// Where an ability is used from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum AbilityLocation {
    /// 0: nowhere the interface offers it.
    None = 0,
    /// 1: a weapon slot.
    Weapon = 1,
    /// 2: the spellbook.
    Spell = 2,
    /// 3: the item's slot.
    Equipment = 3,
    /// 4: the innate abilities.
    Innate = 4,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u16),
}

/// What an ability targets.
///
/// The names are the BG / IWD / EE meanings. PST numbers some
/// differently: 2 is a crash, 3 a character portrait (IESDP ITM V1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AbilityTarget {
    /// 0: nothing; it can't be used.
    Invalid = 0,
    /// 1: a living creature.
    LivingActor = 1,
    /// 2: an item in the inventory.
    Inventory = 2,
    /// 3: a dead creature.
    DeadActor = 3,
    /// 4: any point within range.
    Point = 4,
    /// 5: the caster.
    Caster = 5,
    /// 6: crashes the game; not to be used.
    Crash = 6,
    /// 7 (EE): the caster, at once, past several restrictions.
    CasterInstant = 7,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_keep_their_meaning() {
        assert_eq!(AbilityLocation::from(4), AbilityLocation::Innate);
        assert_eq!(AbilityLocation::from(5), AbilityLocation::Unknown(5));
        assert_eq!(AbilityTarget::from(1), AbilityTarget::LivingActor);
        assert_eq!(u8::from(AbilityTarget::Unknown(200)), 200);
    }
}
