//! [`ItemType`]: an item's category (header 0x1C), as IESDP numbers them.

use num_enum::{FromPrimitive, IntoPrimitive};

/// An item's category. Some values mean different things in different
/// games, as noted; the names follow BG / BG2 / IWD, and IWD2 for the
/// values only it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ItemType {
    /// 0x00: books and miscellaneous.
    Misc = 0x00,
    Amulet = 0x01,
    Armor = 0x02,
    Belt = 0x03,
    Boots = 0x04,
    Arrows = 0x05,
    /// 0x06: bracers and gauntlets.
    Bracers = 0x06,
    /// 0x07: helmets, hats and other headgear.
    Headgear = 0x07,
    Key = 0x08,
    Potion = 0x09,
    Ring = 0x0A,
    Scroll = 0x0B,
    Shield = 0x0C,
    Food = 0x0D,
    /// 0x0E: bullets, for a sling.
    Bullets = 0x0E,
    Bow = 0x0F,
    Dagger = 0x10,
    /// 0x11: maces (in BG, clubs too).
    Mace = 0x11,
    Sling = 0x12,
    SmallSword = 0x13,
    /// 0x14: large swords (in BG, two-handed and bastard swords too).
    LargeSword = 0x14,
    Hammer = 0x15,
    MorningStar = 0x16,
    Flail = 0x17,
    Dart = 0x18,
    /// 0x19: one-handed axes.
    Axe = 0x19,
    Quarterstaff = 0x1A,
    Crossbow = 0x1B,
    /// 0x1C: fists, fist irons, punch daggers.
    HandToHand = 0x1C,
    Spear = 0x1D,
    /// 0x1E: halberds and other two-handed polearms.
    Halberd = 0x1E,
    Bolts = 0x1F,
    /// 0x20: cloaks and robes.
    Cloak = 0x20,
    /// 0x21: gold pieces, not an inventory item.
    Gold = 0x21,
    Gem = 0x22,
    Wand = 0x23,
    /// 0x24: containers; eyes (PST); broken armour.
    Containers = 0x24,
    /// 0x25: books; bracelets (PST); broken shields.
    Books = 0x25,
    /// 0x26: familiars; earrings (PST); broken swords.
    Familiars = 0x26,
    /// 0x27: tattoos (PST).
    Tattoo = 0x27,
    /// 0x28: lenses (PST).
    Lens = 0x28,
    /// 0x29: bucklers; teeth (PST).
    Buckler = 0x29,
    Candle = 0x2A,
    /// 0x2B: a child's body (IWD2).
    ChildBody = 0x2B,
    /// 0x2C: clubs (IWD).
    Club = 0x2C,
    /// 0x2D: a female body (IWD2).
    FemaleBody = 0x2D,
    /// 0x2E: keys (IWD2).
    Iwd2Key = 0x2E,
    /// 0x2F: large shields (IWD).
    LargeShield = 0x2F,
    /// 0x30: a male body (IWD2).
    MaleBody = 0x30,
    /// 0x31: medium shields (IWD).
    MediumShield = 0x31,
    Notes = 0x32,
    /// 0x33: rods (IWD2).
    Rod = 0x33,
    /// 0x34: skulls (IWD2).
    Skull = 0x34,
    /// 0x35: small shields (IWD).
    SmallShield = 0x35,
    /// 0x36: a spider's body (IWD2).
    SpiderBody = 0x36,
    /// 0x37: telescopes (IWD).
    Telescope = 0x37,
    /// 0x38: drinks (IWD).
    Drink = 0x38,
    /// 0x39: great swords (IWD).
    GreatSword = 0x39,
    Container = 0x3A,
    /// 0x3B: fur, pelts.
    Fur = 0x3B,
    LeatherArmor = 0x3C,
    StuddedLeatherArmor = 0x3D,
    ChainMail = 0x3E,
    SplintMail = 0x3F,
    HalfPlate = 0x40,
    FullPlate = 0x41,
    HideArmor = 0x42,
    Robe = 0x43,
    /// 0x44: scale mail (IWD2).
    ScaleMail = 0x44,
    BastardSword = 0x45,
    Scarf = 0x46,
    /// 0x47: food (IWD2).
    Iwd2Food = 0x47,
    Hat = 0x48,
    Gauntlet = 0x49,
    /// Any other value, kept as read.
    #[num_enum(catch_all)]
    Unknown(u16),
}

impl ItemType {
    /// A shield of any size, as opposed to a weapon held in the off hand.
    #[must_use]
    pub fn is_shield(self) -> bool {
        matches!(
            self,
            Self::Shield
                | Self::Buckler
                | Self::LargeShield
                | Self::MediumShield
                | Self::SmallShield
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_value_round_trips() {
        for raw in 0..=u16::MAX {
            assert_eq!(u16::from(ItemType::from(raw)), raw);
        }
    }

    #[test]
    fn values_match_iesdp() {
        assert_eq!(ItemType::from(0x0C), ItemType::Shield);
        assert_eq!(ItemType::from(0x2F), ItemType::LargeShield);
        assert_eq!(ItemType::from(0x39), ItemType::GreatSword);
        assert_eq!(ItemType::from(0x49), ItemType::Gauntlet);
        assert_eq!(ItemType::from(0x4A), ItemType::Unknown(0x4A));
    }

    #[test]
    fn shields_are_told_from_weapons() {
        assert!(ItemType::MediumShield.is_shield());
        assert!(!ItemType::SmallSword.is_shield());
    }
}
