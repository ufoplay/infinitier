//! An item header's bitfields: its flags (0x18), who can't use it (0x1E) and
//! which kits can't (0x29, 0x2B, 0x2D, 0x2F). Their bits mean different
//! things in each version (V1: BG / BG2 / IWD and the EEs; V1.1: PST; V2.0:
//! IWD2), so each version has its own types. Bit `8 × n + b` is bit `b` of
//! the field's byte `n`, as IESDP lays them out. Unnamed bits are kept as
//! read.

bitflags::bitflags! {
    /// An ITM V1 item's flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ItmFlags: u32 {
        /// Unsellable: a critical item.
        const Critical = 1 << 0;
        const TwoHanded = 1 << 1;
        /// Can be moved and dropped.
        const Movable = 1 << 2;
        const Displayable = 1 << 3;
        const Cursed = 1 << 4;
        /// A scroll that can't be written into a spellbook.
        const CannotScribe = 1 << 5;
        const Magical = 1 << 6;
        /// Disables the off hand: its attacks and equipped effects.
        const LeftHanded = 1 << 7;
        const Silver = 1 << 8;
        const ColdIron = 1 << 9;
        /// Stolen, so unsellable; in the EEs, off-hand animations.
        const Stolen = 1 << 10;
        /// Can be talked to (unsellable in BG1 and IWD).
        const Conversable = 1 << 11;
        /// (EE) Two-handed for animations only.
        const FakeTwoHanded = 1 << 12;
        /// (EE) Forbids a weapon in the off hand.
        const ForbidOffHand = 1 << 13;
        /// (PSTEE) Usable from the inventory.
        const UsableInInventory = 1 << 14;
        /// (EE) Adamantine.
        const Adamantine = 1 << 15;
        /// (EE, ToBEx) Not dispellable in the magical weapon slot.
        const Undispellable = 1 << 24;
        /// (EE, ToBEx) Toggles critical hit aversion.
        const ToggleCriticalAversion = 1 << 25;
    }
}

bitflags::bitflags! {
    /// An ITM V1.1 (PST) item's flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ItmFlagsV1_1: u32 {
        /// Unsellable: a critical item.
        const Critical = 1 << 0;
        const TwoHanded = 1 << 1;
        /// Can be moved and dropped.
        const Movable = 1 << 2;
        const Displayable = 1 << 3;
        const Cursed = 1 << 4;
        /// A scroll that can't be written into a spellbook.
        const CannotScribe = 1 << 5;
        const Magical = 1 << 6;
        const Silver = 1 << 8;
        const ColdIron = 1 << 9;
        const Steel = 1 << 10;
        /// Can be talked to.
        const Conversable = 1 << 11;
        const Pulsating = 1 << 12;
    }
}

bitflags::bitflags! {
    /// An ITM V2.0 (IWD2) item's flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ItmFlagsV2: u32 {
        /// Unsellable: a critical item, kept on the ground.
        const Critical = 1 << 0;
        const TwoHanded = 1 << 1;
        /// Can be moved and dropped.
        const Movable = 1 << 2;
        const Displayable = 1 << 3;
        const Cursed = 1 << 4;
        const Magical = 1 << 6;
        const Bow = 1 << 7;
        const Silver = 1 << 8;
        const ColdIron = 1 << 9;
        /// Stolen, so unsellable.
        const Stolen = 1 << 10;
        /// Can't be put in a container that can be carried.
        const NoContainer = 1 << 11;
        const Pulsating = 1 << 12;
    }
}

bitflags::bitflags! {
    /// Who can't use an ITM V1 item: alignments, classes, races.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Usability: u32 {
        const Chaotic = 1 << 0;
        const Evil = 1 << 1;
        const Good = 1 << 2;
        /// Neutral on the good–evil axis.
        const NeutralGoodEvil = 1 << 3;
        const Lawful = 1 << 4;
        /// Neutral on the law–chaos axis.
        const NeutralLawChaos = 1 << 5;
        const Bard = 1 << 6;
        const Cleric = 1 << 7;
        const ClericMage = 1 << 8;
        const ClericThief = 1 << 9;
        const ClericRanger = 1 << 10;
        const Fighter = 1 << 11;
        const FighterDruid = 1 << 12;
        const FighterMage = 1 << 13;
        const FighterCleric = 1 << 14;
        const FighterMageCleric = 1 << 15;
        const FighterMageThief = 1 << 16;
        const FighterThief = 1 << 17;
        /// Mages and sorcerers.
        const Mage = 1 << 18;
        const MageThief = 1 << 19;
        const Paladin = 1 << 20;
        const Ranger = 1 << 21;
        const Thief = 1 << 22;
        const Elf = 1 << 23;
        const Dwarf = 1 << 24;
        const HalfElf = 1 << 25;
        const Halfling = 1 << 26;
        const Human = 1 << 27;
        const Gnome = 1 << 28;
        const Monk = 1 << 29;
        /// Druids (and shamans in the EEs).
        const Druid = 1 << 30;
        const HalfOrc = 1 << 31;
    }
}

bitflags::bitflags! {
    /// Who can't use an ITM V1.1 (PST) item: alignments, factions,
    /// classes, companions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct UsabilityV1_1: u32 {
        const Chaotic = 1 << 0;
        const Evil = 1 << 1;
        const Good = 1 << 2;
        /// Neutral on the good–evil axis.
        const NeutralGoodEvil = 1 << 3;
        const Lawful = 1 << 4;
        /// Neutral on the law–chaos axis.
        const NeutralLawChaos = 1 << 5;
        const Sensates = 1 << 6;
        const Priest = 1 << 7;
        const Godsmen = 1 << 8;
        const Anarchists = 1 << 9;
        const Chaosmen = 1 << 10;
        const Fighter = 1 << 11;
        const NoFaction = 1 << 12;
        const FighterMage = 1 << 13;
        const Dustmen = 1 << 14;
        const Mercykillers = 1 << 15;
        const Indeps = 1 << 16;
        const FighterThief = 1 << 17;
        const Mage = 1 << 18;
        const MageThief = 1 << 19;
        const Dakkon = 1 << 20;
        const FallFromGrace = 1 << 21;
        const Thief = 1 << 22;
        const Vhailor = 1 << 23;
        const Ignus = 1 << 24;
        const Morte = 1 << 25;
        const Nordom = 1 << 26;
        const Annah = 1 << 28;
        const NamelessOne = 1 << 30;
    }
}

bitflags::bitflags! {
    /// Who can't use an ITM V2.0 (IWD2) item: classes, alignments, races.
    /// IESDP's table is unclear for the two neutral alignments (bits 15 and
    /// 17) and the elf (bit 31), so those stay unnamed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct UsabilityV2: u32 {
        const Barbarian = 1 << 0;
        const Bard = 1 << 1;
        const Cleric = 1 << 2;
        const Druid = 1 << 3;
        const Fighter = 1 << 4;
        const Monk = 1 << 5;
        const Paladin = 1 << 6;
        const Ranger = 1 << 7;
        const Rogue = 1 << 8;
        const Sorcerer = 1 << 9;
        const Wizard = 1 << 10;
        const Chaotic = 1 << 12;
        const Evil = 1 << 13;
        const Good = 1 << 14;
        const Lawful = 1 << 16;
        const Dwarf = 1 << 24;
        const HalfElf = 1 << 25;
        const Halfling = 1 << 26;
        const Human = 1 << 27;
        const Gnome = 1 << 28;
        const HalfOrc = 1 << 29;
    }
}

bitflags::bitflags! {
    /// Which kits can't use an ITM V1 item. Its four bytes are apart in the
    /// header (0x29, 0x2B, 0x2D, 0x2F); bits 0 to 7 are the first.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct KitUsability: u32 {
        const ClericOfTalos = 1 << 0;
        const ClericOfHelm = 1 << 1;
        const ClericOfLathander = 1 << 2;
        const TotemicDruid = 1 << 3;
        const ShapeshifterDruid = 1 << 4;
        const AvengerDruid = 1 << 5;
        const Barbarian = 1 << 6;
        const Wildmage = 1 << 7;
        const Stalker = 1 << 8;
        const Beastmaster = 1 << 9;
        const Assassin = 1 << 10;
        const BountyHunter = 1 << 11;
        const Swashbuckler = 1 << 12;
        const Blade = 1 << 13;
        const Jester = 1 << 14;
        const Skald = 1 << 15;
        const Diviner = 1 << 16;
        const Enchanter = 1 << 17;
        const Illusionist = 1 << 18;
        const Invoker = 1 << 19;
        const Necromancer = 1 << 20;
        const Transmuter = 1 << 21;
        /// Characters without a kit.
        const NoKit = 1 << 22;
        const Ferlain = 1 << 23;
        const Berserker = 1 << 24;
        const Wizardslayer = 1 << 25;
        const Kensai = 1 << 26;
        const Cavalier = 1 << 27;
        const Inquisitor = 1 << 28;
        const UndeadHunter = 1 << 29;
        const Abjurer = 1 << 30;
        const Conjurer = 1 << 31;
    }
}

impl KitUsability {
    /// The bitfield from its four bytes, in header order.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self::from_bits_retain(u32::from_le_bytes(bytes))
    }

    /// Its four bytes, in header order.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 4] {
        self.bits().to_le_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kit_byte_is_its_own_bits() {
        let kits = KitUsability::from_bytes([0b1, 0, 0, 0b1000_0000]);
        assert_eq!(kits, KitUsability::ClericOfTalos | KitUsability::Conjurer);
        assert_eq!(kits.to_bytes(), [0b1, 0, 0, 0b1000_0000]);
    }

    #[test]
    fn a_usability_bit_is_its_bytes_bit() {
        // Byte 3, bit 7: the half-orc.
        let usability = Usability::from_bits_retain(u32::from_le_bytes([0, 0, 0, 0b1000_0000]));
        assert_eq!(usability, Usability::HalfOrc);
    }

    #[test]
    fn unnamed_bits_are_kept() {
        let flags = ItmFlagsV2::from_bits_retain(1 << 5 | 1 << 1);
        assert!(flags.contains(ItmFlagsV2::TwoHanded));
        assert_eq!(flags.bits(), 1 << 5 | 1 << 1);
    }
}
