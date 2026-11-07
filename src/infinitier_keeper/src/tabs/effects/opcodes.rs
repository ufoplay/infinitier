//! Generated opcode table for the Effects tab: each opcode's name, what its
//! two parameters hold, and its description.
//!
//! Source: EEKeeper 1.0.4.0 `bgEEEffects.dat`, a CSV of opcode, a number
//! unused here, name (already suffixed with `[opcode]`), parameter 1,
//! parameter 2 and description. Reproduced here so the Effects tab and its
//! dialog match EEKeeper's. Do not edit by hand.

/// What EEKeeper says of an effect opcode.
pub struct Opcode {
    /// Its name, e.g. `"Stat: AC vs. Damage Type Modifier [0]"`.
    pub name: &'static str,
    /// What parameter 1 holds, e.g. `"AC Modifier"`: the name the
    /// description calls it by. `"Irrelevant"` when the effect ignores it.
    pub param1: &'static str,
    /// What parameter 2 holds, like [`Self::param1`].
    pub param2: &'static str,
    /// What the effect does, and its parameters' known values.
    pub description: &'static str,
}

/// EEKeeper's entry for `opcode`; `None` past the last one it knows.
pub fn opcode(opcode: u32) -> Option<&'static Opcode> {
    OPCODES.get(usize::try_from(opcode).ok()?)
}

/// Every opcode EEKeeper knows, with its entry.
pub fn all() -> impl Iterator<Item = (u32, &'static Opcode)> {
    (0..).zip(OPCODES.iter())
}

#[rustfmt::skip]
static OPCODES: [Opcode; 322] = [
    // 0
    Opcode {
        name: "Stat: AC vs. Damage Type Modifier [0]",
        param1: "AC Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'AC Modifier' field to the category specified by the 'Type' field.

Known values for 'Type' are:
0   All
1   Crushing
2   Missile
4   Piercing
8   Slashing
16  Base AC setting (sets the targets AC to the value specified by the 'AC Modifier' field. If the targets AC is already 'AC Modifier' or below, this effect will do nothing).

AC is capped to the range [-20, 20]"#,
    },
    // 1
    Opcode {
        name: "Stat: Attacks Per Round Modifier [1]",
        param1: "Key Modifier",
        param2: "Type",
        description: r#"Alters a characters Attacks per Round, by changing the Key by the modifier value specified by the 'Key Value' field, in the style specified by 'Type' field.

Key     Attacks Per Round
0        0
1        1
2        2
3        3
4        4
5        5
6        0.5
7        1.5
8        2.5
9        3.5
10      4.5

Known values for 'Type' are:
0   Cumulative Modifier  -> Key = Key + 'Key Modifier' value
1   Flat Value Modifier  -> Key = 'Key Modifier' value
2   Percentage Modifier  -> Key = (Key * 'Key Modifier' value) / 100
3   Final Value Modifier -> Key = 'Key Modifier' value

NB. When this opcode is stacked, the values of the Key Modifier are stacked, not the number of attacks."#,
    },
    // 2
    Opcode {
        name: "Cure: Sleep [2]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the state_sleeping flag from the targeted creature(s).
This effect ignores durations attributed to it.

NB. Does not remove the state_helpless flag."#,
    },
    // 3
    Opcode {
        name: "State: Berserking [3]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the berserk effect to the targeted creature(s). Target will uncontrollably attack enemies if any are in view, and will occasionally attack neutral creatures. The affected creature becomes berserk and its AI scripts stop running. Grants a +2 bonus to hit and damage with all weapons. It also sets the STATE_BERSERK flag. This opcode only affects party members.",
    },
    // 4
    Opcode {
        name: "Cure: Berserking [4]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the state_beserk flag from the targeted creature(s). This effect only affects Player Characters.
This effect ignores durations attributed to it."#,
    },
    // 5
    Opcode {
        name: "Charm: Charm Specific Creature [5]",
        param1: "General Type",
        param2: "Charm Type",
        description: r#"Charms the targeted creature(s) according to value in the 'Charm Type' field, if the target is matched by the value in the 'General Type' field. Values for 'General Type' are from General.ids.

Known values for 'Charm Type' are:
0    Charmed (Target neutral after effect ends)
1    Charmed (Target hostile after effect ends)
2    Dire charmed (Target neutral after effect ends)
3    Dire charmed (Target hostile after effect ends)
4    Controlled by Cleric
5    Target is hostile
1000 Charmed (no text) (Target neutral after effect ends)
1001 Charmed (no text) (Target hostile after effect ends)
1002 Dire charmed (no text) (Target neutral after effect ends)
1003 Dire charmed (no text) (Target hostile after effect ends)
1004 Controlled by Cleric (no text)
1005 Target is hostile (no text)"#,
    },
    // 6
    Opcode {
        name: "Stat: Charisma Modifier [6]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Charisma = Charisma + 'Statistic Modifier' value
1   Flat Value Modifier -> Charisma = 'Statistic Modifier' value
2   Percentage Modifier -> Charisma = (Charisma * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Charisma raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 7
    Opcode {
        name: "Colour: Set Character colours by Palette [7]",
        param1: "Gradient Number",
        param2: "Location",
        description: r#"Alters the colour of the area specified by the 'Location' field, to the colour gradient specified by the 'Gradient Number' field.

Known values for 'Location' are:
00   Belt Buckle (on armour) or amulets (on robes) - entire body for some animations
01   Minor colour
02   Major colour
03   Skin colour
04   Strap colour (on armour) or minor trimming (on robes - the centre strip down a robe and the hem on some robes)
05   Armor colour (and major trimming on robe - cross trim and cape trim on  robe of archmagi)
06   Hair colour
16   Quillons or Crossguard
20   Grip (usually not visible as it's in the characters hand)
21   Blade or major portion of weapon
32   Shield hub
33   Shield interior
34   Shield panelling
34   Helmet misc
37   Shield rim
48   Helm ornamentation - i.e. wings
49   Helm ornamentation - i.e. feathers/horsehair
50   Helm ornamentation - i.e. feathers/horsehair
52   Helm face
53   Helm - the main covering"#,
    },
    // 8
    Opcode {
        name: "Colour: Change by RGB [8]",
        param1: "RGB colour",
        param2: "Location",
        description: r#"Alters the colour of the area specified by the 'Location' field, to the colour specified by the 'RGB colour' field.

The 'RGB Colour' field is handled as follows:
Second byte = Red (0-255)
Third byte     = Green (0-255)
Fourth byte   = Blue (0-255)

Known values for 'Location' are:
00   Belt Buckle (on armour) or amulets (on robes) - entire body for some animations
01   Minor colour
02   Major colour
03   Skin colour
04   Strap colour (on armour) or minor trimming (on robes - the centre strip down a robe and the hem on some robes)
05   Armor colour (and major trimming on robe - cross trim and cape trim on  robe of archmagi)
06   Hair colour
16   Quillons or Crossguard
20   Grip (usually not visible as it's in the characters hand)
21   Blade or major portion of weapon
32   Shield hub
33   Shield interior
34   Shield panelling
34   Helmet misc
37   Shield rim
48   Helm ornamentation - i.e. wings
49   Helm ornamentation - i.e. feathers/horsehair
50   Helm ornamentation - i.e. feathers/horsehair
52   Helm face
53   Helm - the main covering"#,
    },
    // 9
    Opcode {
        name: "Colour: Glow Pulse [9]",
        param1: "RGB Colour",
        param2: "Location and Speed",
        description: r#"Pulses the area specified by the 'Location and Speed' field, at the rate specified by the 'Location and Speed' field, with the colour specified by the 'RGB Colour' field. If the colour values are all set to 0, the glow effect only occurs if the creature is in darkness; outside at night, inside a building, or standing in shadows.

The 'RGB Colour' field is handled as follows:
Second byte = Red (0-255)
Third byte     = Green (0-255)
Fourth byte   = Blue (0-255)

The 'Location' field is handled as follows:
First byte = Location
Third byte  = Speed (0-255)

A speed of 0 does not pulsate. A speed of 1 is fastest, and a speed of 255 is slowest.

Known values for 'Location' are:
00   Belt Buckle (on armour) or amulets (on robes) - entire body for some animations
01   Minor colour
02   Major colour
03   Skin colour
04   Strap colour (on armour) or minor trimming (on robes - the centre strip down a robe and the hem on some robes)
05   Armor colour (and major trimming on robe - cross trim and cape trim on  robe of archmagi)
06   Hair colour
16   Quillons or Crossguard
20   Grip (usually not visible as it's in the characters hand)
21   Blade or major portion of weapon
32   Shield hub
33   Shield interior
34   Shield panelling
34   Helmet misc
37   Shield rim
48   Helm ornamentation - i.e. wings
49   Helm ornamentation - i.e. feathers/horsehair
50   Helm ornamentation - i.e. feathers/horsehair
52   Helm face
53   Helm - the main covering"#,
    },
    // 10
    Opcode {
        name: "Stat: Constitution Modifier [10]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Constitution = Constitution + 'Statistic Modifier' value
1   Flat Value Modifier -> Constitution = 'Statistic Modifier' value
2   Percentage Modifier -> Constitution = (Constitution * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Constitution raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 11
    Opcode {
        name: "Cure: Poison [11]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the poison effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 12
    Opcode {
        name: "HP: Damage [12]",
        param1: "Damage Amount",
        param2: "Type",
        description: r#"Damages the target by the value specified by the 'Damage Amount' field, with the damage type specified by the 'Type' field.

Known values for 'Type' are:
0           Crushing
1           Crushing Set to Value
2           Crushing Set to Percentage
3           Crushing Reduce by Percentage
65536       Acid
65537       Acid Set to Value
65538       Acid Set to Percentage
65539       Acid Reduce by Percentage
131072      Cold
131073      Cold Set to Value
131074      Cold Set to Percentage
131075      Cold Reduce by Percentage
262144      Electricity
262145      Electricity Set to Value
262146      Electricity Set to Percentage
262147      Electricity Reduce by Percentage
524288      Fire
524289      Fire Set to Value
524290      Fire Set to Percentage
524291      Fire Reduce by Percentage
1048576     Piercing
1048577     Piercing Set to Value
1048578     Piercing Set to Percentage
1048579     Piercing Reduce by Percentage
2097152     Poison
2097153     Poison Set to Value
2097154     Poison Set to Percentage
2097155     Poison Reduce by Percentage
4194304     Magic
4194305     Magic Set to Value
4194306     Magic Set to Percentage
4194307     Magic Reduce by Percentage
8388608     Missile
8388609     Missile Set to Value
8388610     Missile Set to Percentage
8388611     Missile Reduce by Percentage
16777216    Slashing
16777217    Slashing Set to Value
16777218    Slashing Set to Percentage
16777218    Slashing Reduce by Percentage
33554432    Magic Fire
33554433    Magic Fire Set to Value
33554434    Magic Fire Set to Percentage
33554435    Magic Fire Reduce by Percentage
67108864    Magic Cold
67108865    Magic Cold Set to Value
67108866    Magic Cold Set to Value
67108867    Magic Cold Reduce by Percentage
134217728   Stunning
134217729   Stunning Set to Value
134217730   Stunning Set to Percentage
134217731   Stunning Reduce by Percentage
268435456   Soul Eater
268435457   Soul Eater Set to Value
268435458   Soul Eater Set to Percentage
268435459   Soul Eater Reduce by Percentage
1073741824  Disease
1073741825  Disease Set to Value
1073741826  Disease Set to Percentage
1073741827  Disease Reduce by Percentage

The Dice values can be used to inflict varying damage amounts - e.g. 2d6 would do 2-12 damage (Damage type is still specified by the 'Type' value).

NB. 'Stunning' damage does not stun the target. Stunning damage cannot kill a creature, creatures are reduced to 1 HP and are knocked unconscious.
NB. Using 'Magic Fire' or 'Magic Cold' as a damage type will result the game to crash if the damage kills the target.
NB. If 'Damage Amount' and the Dice values are both non-zero, the total damage inflicted is: 'Damage Amount' + Dice value.
NB. 'Damage Amount' can be positive or negative (though the damage sound will play in either case)."#,
    },
    // 13
    Opcode {
        name: "Death: Instant Death [13]",
        param1: "Text Notification",
        param2: "Death Type",
        description: r#"Instantly kills the targeted creature(s), in the style specified by the 'Death Type' value.

Known values for 'Text Notification' are:
0   Show death message
1   Don't show death message

Known values for 'Death Type' are:
0   Acid death
1   Burning death
2   Crushed death
4   Normal death
8   Chunked death
16  Stoned death
32  Freezing death
64  Exploding stoned death
128 Exploding freezing death
256 Electrified death
512 Disintegration death"#,
    },
    // 14
    Opcode {
        name: "Graphics: Defrost [14]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the frozen_death state from the targeted creature(s).",
    },
    // 15
    Opcode {
        name: "Stat: Dexterity Modifier [15]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Dexterity = Dexterity + 'Statistic Modifier' value
1   Flat Value Modifier -> Dexterity = 'Statistic Modifier' value
2   Percentage Modifier -> Dexterity = (Dexterity * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Dexterity raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 16
    Opcode {
        name: "State: Haste [16]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Applies the Haste effect, in the style specified by the 'Type' value.

Known values for 'Type' are:
0   Normal Haste
1   Improved Haste
2   Speed haste without attack bonuses

NB. This effect will cancel out the slow opcode."#,
    },
    // 17
    Opcode {
        name: "HP: Current HP Modifier [17]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> HP = HP + 'Statistic Modifier' value
1   Flat Value Modifier -> HP = 'Statistic Modifier' value
2   Percentage Modifier -> HP = (HP * 'Statistic Modifier' value) / 100

NB. If 'Statistic Modifier' and the Dice values are both non-zero, the total HP change is: 'Statistic Modifier' + Dice value.
NB. 'Statistic Modifier' can be positive or negative.

NB. The upper word of the 'Type' field can further control the style.
Known values for the upper word of'Type' are:
1 Cure death
2 Removes all non timing-mode-9 effects
3 Cure death and remove non timing-mode-9 effects"#,
    },
    // 18
    Opcode {
        name: "HP: Maximum HP Modifier [18]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> HP = HP + 'Statistic Modifier' value (affects current and max HP)
1   Flat Value Modifier -> HP = 'Statistic Modifier' value (affects current and max HP)
2   Percentage Modifier -> HP = (HP * 'Statistic Modifier' value) / 100 (affects current and max HP)

3   As 0 (does not affect current HP)
4   As 1 (does not affect current HP)
5   As 2 (does not affect current HP)

NB. For non-permanent timing modes of 'Type' 2 or 5 use the base maximum HP value, and therefor override other 'Type' 2 or 5 modifiers.
NB. If 'Statistic Modifier' and the Dice values are both non-zero, the total HP change is: 'Statistic Modifier' + Dice value.
NB. 'Statistic Modifier' can be positive or negative."#,
    },
    // 19
    Opcode {
        name: "Stat: Intelligence Modifier [19]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Intelligence = Intelligence + 'Statistic Modifier' value
1   Flat Value Modifier -> Intelligence = 'Statistic Modifier' value
2   Percentage Modifier -> Intelligence = (Intelligence * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Intelligence raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 20
    Opcode {
        name: "State: Invisibility [20]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Applies the invisibility effect, in the style specified by the 'Type' value.

Known values for 'Type' are :
0   Normal Invisibility   -> affected creature makes the first attack with +4
1   Improved Invisibility -> affected creature stays invisible after attack

Improved Invisibility does not provide saving throw bonuses
Improved Invisibility lingers after the duration is spent and act as permanent if used as a While Equipped effect
Creatures attacking an improved invisible creature suffer a -4 penalty to hit, unless they are affected by #193 Invisible Detection by Script"#,
    },
    // 21
    Opcode {
        name: "Stat: Lore Modifier [21]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Lore = Lore + 'Statistic Modifier' value
1   Flat Value Modifier -> Lore = 'Statistic Modifier' value
2   Percentage Modifier -> Lore = (Lore * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.
NB. This effect does not work with Timing Mode 1."#,
    },
    // 22
    Opcode {
        name: "Stat: Cumulative Luck Bonus [22]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field. Luck affects the minimum damage caused by a weapon (up to the normal maximum damage of the weapon) and the attack roll modifier (up to a maximum of 20, excluding natural attack roll modifiers, e.g. high STR, invisibility etc) by 'Statistic Modifier' amount. 'Statistic Modifier' also affects the damge rolls of opponents (in feature blocks, not extension headers). The effect does not set STATE_LUCK.

Known values for 'Type' are:
0   Cumulative Modifier -> Luck = Luck + 'Statistic Modifier' value
1   Flat Value Modifier -> Luck = 'Statistic Modifier' value
2   Percentage Modifier -> Luck = (Luck * 'Statistic Modifier' value) / 100"#,
    },
    // 23
    Opcode {
        name: "Stat: Morale Modifier [23]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: "Sets the Morale stat of the targetted creature(s) to 10.",
    },
    // 24
    Opcode {
        name: "State: Horror [24]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the horror effect to the targeted creature(s).",
    },
    // 25
    Opcode {
        name: "State: Poison [25]",
        param1: "Damage Amount",
        param2: "Type",
        description: r#"Damages the target by the 'Damage Amount' value, in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Inflict 1HP per second (only if 'Damage Amount' is non-zero)
1   Inflict 1HP per second (only if 'Damage Amount' is greater than 1)
2   Inflict 'Damage Amount' per second
3   Inflict 1HP every 'Damage Amount' seconds
4   Inflict Param 3 Damage every Param 1 seconds
5+  Crash

NB. The 'Damage Amount' parameter should be set a positive value."#,
    },
    // 26
    Opcode {
        name: "Item: Remove Curse [26]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes cursed items from the targeted creature(s).",
    },
    // 27
    Opcode {
        name: "Stat: Acid Resistance Modifier [27]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value
2   Percentage Modifier -> Resistance = (Resistance * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 28
    Opcode {
        name: "Stat: Cold Resistance Modifier [28]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value
2   Percentage Modifier -> Resistance = (Resistance * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 29
    Opcode {
        name: "Stat: Electricity Resistance Modifier [29]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value
2   Percentage Modifier -> Resistance = (Resistance * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 30
    Opcode {
        name: "Stat: Fire Resistance Modifier [30]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value
2   Percentage Modifier -> Resistance = (Resistance * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 31
    Opcode {
        name: "Stat: Magic Damage Resistance Modifier [31]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value
2   Percentage Modifier -> Resistance = (Resistance * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 32
    Opcode {
        name: "Cure: Death (Raise Dead) [32]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Raises the targeted creature(s) from the dead.
This effect ignores durations attributed to it."#,
    },
    // 33
    Opcode {
        name: "Stat: Save vs. Death Modifier [33]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Save = Save + 'Statistic Modifier' value
1   Flat Value Modifier -> Save = 'Statistic Modifier' value
2   Percentage Modifier -> Save = (Save * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 34
    Opcode {
        name: "Stat: Save vs. Wands Modifier [34]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Save = Save + 'Statistic Modifier' value
1   Flat Value Modifier -> Save = 'Statistic Modifier' value
2   Percentage Modifier -> Save = (Save * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 35
    Opcode {
        name: "Stat: Save vs. Petrification/Polymorph Modifier [35]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Save = Save + 'Statistic Modifier' value
1   Flat Value Modifier -> Save = 'Statistic Modifier' value
2   Percentage Modifier -> Save = (Save * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 36
    Opcode {
        name: "Stat: Save vs. Breath Weapons Modifier [36]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Save = Save + 'Statistic Modifier' value
1   Flat Value Modifier -> Save = 'Statistic Modifier' value
2   Percentage Modifier -> Save = (Save * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 37
    Opcode {
        name: "Stat: Save vs. Spells Modifier [37]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Save = Save + 'Statistic Modifier' value
1   Flat Value Modifier -> Save = 'Statistic Modifier' value
2   Percentage Modifier -> Save = (Save * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative."#,
    },
    // 38
    Opcode {
        name: "State: Silence [38]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the silence effect to the targeted creature(s). A silenced creature cannot cast spells, use special abilities or initiate dialog. The engine is hard-coded to allow SPWI219 (Vocalize) and SPIN649 (Teleport to Pocket Plane) to be cast by a silenced creature.",
    },
    // 39
    Opcode {
        name: "State: Unconsciousness [39]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the unconsciousness effect to the targeted creature(s).

NB. A permanent timing mode will set the state_helpless flag."#,
    },
    // 40
    Opcode {
        name: "State: Slow [40]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the slow effect to the targeted creature(s).
This effect will cancel out the haste opcode.

NB. This effect doubles the casting time of all spells."#,
    },
    // 41
    Opcode {
        name: "Graphics: Sparkle [41]",
        param1: "Colour",
        param2: "Type",
        description: r#"Displays the sparkle effect specified by the 'Type' field, in the colour specified by the 'Colour' field. The colour field may be connected to sprkclr.2da.

Known values for 'Colour' are:
0   Crash
1   Black
2   Blue
3   Chromatic
4   Gold
5   Green
6   Purple
7   Red
8   White
9   Ice
10  Stone
11  Magenta
12  Orange
13  Crash+

Known values for 'Type' are:
1   Explosion
2   No effect?
3   Sparkle Shower

Both of these params use only the lower 16 bits (upper bits might have other function)."#,
    },
    // 42
    Opcode {
        name: "Spell: Wizard Spell Slots Modifier [42]",
        param1: "Slot Amount Modifier",
        param2: "Spell Level",
        description: r#"Alters the number of wizard spell slots available to the targeted creature(s), as specified by the 'Slot Amount Modifier' field, at the level specified by the 'Spell Level' field.

Known values for 'Spell Level' are:
0   Double spells
1   Level 1
2   Level 2
4   Level 3
8   Level 4
16  Level 5
32  Level 6
64  Level 7
128 Level 8
256 Level 9

For a 'Spell Level' of 0, the amount of spell slots is doubled for the level specified by the 'Slot Amount Modifier' value and all levels below. The target creature(s) must have at least 1 slot at the selected level to have their slot count changed. If the resultant slot count is negative, the creature is given 12 slots.

'Statistic Modifier' can be positive or negative."#,
    },
    // 43
    Opcode {
        name: "Cure: Stone to Flesh [43]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Changes the targeted creature(s) from stone to flesh.",
    },
    // 44
    Opcode {
        name: "Stat: Strength Modifier [44]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Strength = Strength + 'Statistic Modifier' value
1   Flat Value Modifier -> Strength = 'Statistic Modifier' value
2   Percentage Modifier -> Strength = (Strength * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Strength raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 45
    Opcode {
        name: "State: Stun [45]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Stuns the targeted creature(s).",
    },
    // 46
    Opcode {
        name: "Cure: Stun (Unstun) [46]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Unstuns the targeted creature(s) (opcode #45).
This effect ignores durations attributed to it."#,
    },
    // 47
    Opcode {
        name: "Cure: Invisibility [47]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the invisibility effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 48
    Opcode {
        name: "Cure: Silence (Vocalize) [48]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the silence effect and the silenced state from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 49
    Opcode {
        name: "Stat: Wisdom Modifier [49]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Wisdom = Wisdom + 'Statistic Modifier' value
1   Flat Value Modifier -> Wisdom = 'Statistic Modifier' value
2   Percentage Modifier -> Wisdom = (Wisdom * 'Statistic Modifier' value) / 100

'Statistic Modifier' can be positive or negative.

With a Timing Modes of Permanent or Delayed, if 'Type' is 0 and 'Statistic Modifier' is positive, the statistic will alter and remain shown in white text, and a message 'Wisdom raised permanently' will be shown.
With a Timing Modes of Permanent or Delayed, if 'Type' is 1 or 2, or 'Statistic Modifier' is negative, the statistic will alter and remain shown in white text, though no message is shown.
With all other Timing Modes, the statistic will alter and be shown in red text."#,
    },
    // 50
    Opcode {
        name: "Colour: Glow by RGB (Brief) [50]",
        param1: "RGB Colour",
        param2: "Speed",
        description: r#"The targeted creature(s) entire animation glows the colour specified by the 'RGB Colour' field, at the speed specified by the (3rd byte of the) 'Speed' field.

The 'RGB Colour' field is handled as follows:
Second byte = Red (0-255)
Third byte  = Green (0-255)
Fourth byte = Blue (0-255)

The 'Speed' field is handled as follows:
Third byte  = Speed (0-255)

A speed of 0 does not pulsate. A speed of 1 is fastest, and a speed of 255 is slowest. The duration of the effect is controlled by the 'Speed' field."#,
    },
    // 51
    Opcode {
        name: "Colour: Strong/Dark by RGB [51]",
        param1: "RGB Colour",
        param2: "Location",
        description: r#"Alters the colour of the area specified by the 'Location and Speed' field, to the colour specified by the 'RGB Colour' field.

The 'RGB Colour' field is handled as follows:
Second byte = Red (0-255)
Third byte  = Green (0-255)
Fourth byte = Blue (0-255)

Known values for 'Location' are:
00   Belt Buckle (on armour) or amulets (on robes) - entire body for some animations
01   Minor colour
02   Major colour
03   Skin colour
04   Strap colour (on armour) or minor trimming (on robes - the centre strip down a robe and the hem on some robes)
05   Armor colour (and major trimming on robe - cross trim and cape trim on  robe of archmagi)
06   Hair colour
16   Quillons or Crossguard
20   Grip (usually not visible as it's in the characters hand)
21   Blade or major portion of weapon
32   Shield hub
33   Shield interior
34   Shield panelling
34   Helmet misc
37   Shield rim
48   Helm ornamentation - i.e. wings
49   Helm ornamentation - i.e. feathers/horsehair
50   Helm ornamentation - i.e. feathers/horsehair
52   Helm face
53   Helm - the main covering"#,
    },
    // 52
    Opcode {
        name: "Colour: Very Bright by RGB [52]",
        param1: "RGB Colour",
        param2: "Location",
        description: r#"Alters the colour of the area specified by the 'Location' field, to the colour specified by the 'RGB Colour' field.

The 'RGB Colour' field is handled as follows:
Second byte = Red (0-255)
Third byte  = Green (0-255)
Fourth byte = Blue (0-255)

Known values for 'Location' are:
00   Belt Buckle (on armour) or amulets (on robes) - entire body for some animations
01   Minor colour
02   Major colour
03   Skin colour
04   Strap colour (on armour) or minor trimming (on robes - the centre strip down a robe and the hem on some robes)
05   Armor colour (and major trimming on robe - cross trim and cape trim on  robe of archmagi)
06   Hair colour
16   Quillons or Crossguard
20   Grip (usually not visible as it's in the characters hand)
21   Blade or major portion of weapon
32   Shield hub
33   Shield interior
34   Shield panelling
34   Helmet misc
37   Shield rim
48   Helm ornamentation - i.e. wings
49   Helm ornamentation - i.e. feathers/horsehair
50   Helm ornamentation - i.e. feathers/horsehair
52   Helm face
53   Helm - the main covering"#,
    },
    // 53
    Opcode {
        name: "Graphics: Animation Change [53]",
        param1: "Animation ID",
        param2: "Type",
        description: r#"Alters the animation of the targeted creature(s) to the animation specified by the 'Animation ID' field, in the style specified by the 'Type' field.

Known values for Animation ID' are: (from animate.ids - in decimal)

Known values for 'Type' are:
0   Change animation - non-permanently
1   Remove any non-permanent animation change ('Animation ID' is unused)
2   Change animation - permanently"#,
    },
    // 54
    Opcode {
        name: "Stat: THAC0 Modifier [54]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100

NB. This effect adjusts the base stat, rather than modified THAC0."#,
    },
    // 55
    Opcode {
        name: "Death: Kill Creature Type [55]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Kills the targeted creature(s) with HD less than those specified by the Dice values, if the target(s) match the IDS mask.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids

'IDS Entry' is a value from the specified IDS file, indicating the specific creature type to affect."#,
    },
    // 56
    Opcode {
        name: "Alignment: Invert [56]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Inverts the alignment of the targeted creature(s).",
    },
    // 57
    Opcode {
        name: "Alignment: Change [57]",
        param1: "Irrelevant",
        param2: "Alignment",
        description: r#"Alters the alignment of the targeted creature(s) to the value specified by 'Alignment'.

Known values for 'Alignment' are:
00   None
17   Lawful Good
18   Lawful Neutral
19   Lawful Evil
33   Neutral Good
34   True Neutral
35   Neutral Evil
49   Chaotic Good
50   Chaotic Neutral
51   Chaotic Evil"#,
    },
    // 58
    Opcode {
        name: "Cure: Dispellable Effects (Dispel Magic) [58]",
        param1: "Level",
        param2: "Type",
        description: r#"Dispels magic from the targeted creature(s) (excepting magical weapons eg. Spirtual Hammer). Depending on the value of the 'Type' field, the 'Level' field can be used to set the level of the effect.

Known values for 'Type' are:
0   Always dispel
1   Use Caster Level
2   Use 'Level' field

With 'Type' of 1 or 2, the base chance of successfully dispelling is 50%. This chance is modified by the relative levels of the dispeller to the caster. Each level below gives a -10% chance, each level above gives a +5% chance. There is always a 1% chance of success or failure."#,
    },
    // 59
    Opcode {
        name: "Stat: Stealth Modifier [59]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field, altering a the Move Silently skill.

Known values for 'Type' are:
0   Cumulative Modifier -> Stealth = Stealth + 'Statistic Modifier' value
1   Flat Value Modifier -> Stealth = 'Statistic Modifier' value
2   Percentage Modifier -> Stealth = (Stealth * 'Statistic Modifier' value) / 100"#,
    },
    // 60
    Opcode {
        name: "Stat: Miscast Magic [60]",
        param1: "Percent Chance",
        param2: "Type",
        description: r#"The targeted creature(s) will fail to cast the type of spell specified by the 'Type' field, 'Percent Chance' percent of the time.

Known values for 'Type' are:
0   Mage Spells
1   Cleric Spells
2   Innate
3   Mage Spells
4   Cleric Spells
5   Innate

'Type' values of 0,1 or 2 display a message -'Spell Failed: casting failure'
'Type' values of 3,4 or 5 display a message -'Spell failed due to dead magic'"#,
    },
    // 61
    Opcode {
        name: "Crash [61]",
        param1: "Undefined",
        param2: "Undefined",
        description: r#"This effect crashes the game.
This effect has previously been referenced as Alchemy."#,
    },
    // 62
    Opcode {
        name: "Spell: Priest Spell Slots Modifier [62]",
        param1: "Slot Amount Modifier",
        param2: "Spell Level",
        description: r#"Alters the number of priest spell slots available to the targeted creature(s), as specified by the 'Slot Amount Modifier' field, at the level specified by the 'Spell Level' field.

Known values for 'Spell Level' are:
0   Double spells
1   Level 1
2   Level 2
4   Level 3
8   Level 4
16  Level 5
32  Level 6
64  Level 7

For a 'Spell Level' of 0, the amount of spell slots is doubled for the level specified by the 'Slot Amount Modifier' value and all levels below. The target creature(s) must have at least 1 slot at the selected level to have their slot count changed. If the resultant slot count is negative, the creature is given 12 slots.

'Statistic Modifier' can be positive or negative."#,
    },
    // 63
    Opcode {
        name: "State: Infravision [63]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Gives the targeted creature(s) infravision.",
    },
    // 64
    Opcode {
        name: "State: Remove Infravision [64]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes infravision from the targeted creature(s).
This effect ignores durations attributed to it.

NB. This will not remove racial infravision."#,
    },
    // 65
    Opcode {
        name: "Overlay: Blur [65]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the blur effect to the targeted creature(s). Graphical displacement is affected by movement rate.",
    },
    // 66
    Opcode {
        name: "Graphics: Transparency Fade [66]",
        param1: "Fade Amount",
        param2: "Irrelevant",
        description: r#"Fades the targeted creature(s) by the value in the amount specified by the 'Fade Amount' field.

Known values for 'Fade Amount' are:
1   Fully visible
..
255 Almost invisible

This effect sets stat #60 (TRANSLUCENT) to param #1."#,
    },
    // 67
    Opcode {
        name: "Summon: Creature Summoning [67]",
        param1: "Unknown",
        param2: "Control",
        description: r#"Summons a creature, with EA set to the 'Control' field. The Resource key should be set to the filename of the creature to summon.

Known values for 'Control' are:
0    Match target (hostile if no target)
1    Match target (hostile if no target)
2    As creature file
3    Match target (hostile if no target)
4    As creature file
5    Hostile to target
6    As creature file
8    As creature file
255  As creature file

If effect's target is set as 'Original Caster' then the 'Control' parameter covers 'Location' as well as control. In this case, nown values for 'Control' are:
2   = Summoned at location of target creature
4   = Summoned at location of caster
6   = Summoned at location of caster"#,
    },
    // 68
    Opcode {
        name: "Summon: Unsummon Creature [68]",
        param1: "Text Notification",
        param2: "Irrelevant",
        description: r#"Unsummons a creature, with feedback controlled by 'Text Notification'. The Resource key should be set to the filename of the animation to play when unsummoning the creature, and defaults to SPGFLSH1 if left empty.

Known values for 'Text Notification' are:
0   No text notification
1   Show text notification"#,
    },
    // 69
    Opcode {
        name: "Protection: From Detection (Non-Detection) [69]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the non-detection effect to the targeted creature(s), setting STATE_NONDETECTION while active. This effect is negated by effect #70 Cure: Non-Detection. While in the non-detection state, the creature is immune to effects #47: Cure Invisibility and #116: Remove invisibilty, though they are affected by effects that remove illusionary protections by school or secondary type.",
    },
    // 70
    Opcode {
        name: "Cure: Non-Detection [70]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the Non-Detection effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 71
    Opcode {
        name: "IDS: Sex Change [71]",
        param1: "Gender Modifier",
        param2: "Type",
        description: r#"Alters the gender of the targetted creature(s) to the value specified by the 'Gender.ids' field, in the style specified by the 'Type' value.

Known values for 'Type' are:
0 Reverse Gender
1 Flat Value Modifier -> Gender = 'Statistic Modifier' value (values from gender.ids)
The effect modifies stat 193."#,
    },
    // 72
    Opcode {
        name: "IDS: Set IDS State [72]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Alters the IDS state of the targeted creature(s) specified by the 'IDS File' field, to the value specified by the 'IDS Entry' field.

Known values for 'IDS File' are:
0   EA
1   Generic
2   Race
3   Class
4   Specific
5   Gender
6   Alignment"#,
    },
    // 73
    Opcode {
        name: "Stat: Extra Damage Modifier [73]",
        param1: "Statistic Modifier",
        param2: "Modifier Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field. The effect is an additive modifier, like adding +10 damage.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 74
    Opcode {
        name: "State: Blindness [74]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the blindness effect to the targeted creature(s). This sets VISUALRANGE to 2, sets STATE_BLIND to true and incurs a cummulative 10 point base THAC0 penalty.",
    },
    // 75
    Opcode {
        name: "Cure: Blindness [75]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the blindness effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 76
    Opcode {
        name: "State: Feeblemindedness [76]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the feeblemind effect to the targeted creature(s) - creatures are unselectable, immobile and do not respond to combat.
This effect ignores durations attributed to it.

NB. A creatures scripts do not while the creature is feebleminded."#,
    },
    // 77
    Opcode {
        name: "Cure: Feeblemindedness [77]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the feeblemind effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 78
    Opcode {
        name: "State: Disease [78]",
        param1: "Damage",
        param2: "Type",
        description: r#"Set the Damage parameter as the amount of damage, and the Disease Type parameter as how the disease functions.

Known values for 'Type' are:
0   1 HP per second
1   1 HP per second
2   'Damage' per Second
3   Cause 1HP every 'Damage' seconds
4   Strength Penalty
5   Dexterity Penalty
6   Constitution Penalty
7   Intelligence Penalty
8   Wisdom Penalty
9   Charisma Penalty
10  Slow Effect

'Damage' can be positive (penalty) or negative (bonus).

Permanent timing modes require a duration for this effect to work."#,
    },
    // 79
    Opcode {
        name: "Cure: Disease [79]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the disease effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 80
    Opcode {
        name: "State: Deafness [80]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the Deafness effect on the targeted creature(s) - creatures suffer a 50% casting failure rate. This is cumulative with Miscast Magic.",
    },
    // 81
    Opcode {
        name: "Cure: Deafness [81]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Remove the deafness effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 82
    Opcode {
        name: "Set AI Script [82]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Sets the script type specified by the 'Type' value to the contents of the resource key.

Known values for 'Type' are:
0   OVERRIDE
1   AREA
2   SPECIFICS
4   CLASS
5   RACE
6   GENERAL
7   DEFAULT

NB. This effect can unset scripts from a creature, which can break the game."#,
    },
    // 83
    Opcode {
        name: "Protection: From Projectile [83]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Gives the targeted creature(s) immunity to the weapon type specified by the 'Type' value.

Values for 'Type' are taken from projectl.ids."#,
    },
    // 84
    Opcode {
        name: "Stat: Magical Fire Resistance Modifier [84]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 85
    Opcode {
        name: "Stat: Magical Cold Resistance Modifier [85]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 86
    Opcode {
        name: "Stat: Slashing Resistance Modifier [86]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 87
    Opcode {
        name: "Stat: Crushing Resistance Modifier [87]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 88
    Opcode {
        name: "Stat: Piercing Resistance Modifier [88]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 89
    Opcode {
        name: "Stat: Missiles Resistance Modifier [89]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100"#,
    },
    // 90
    Opcode {
        name: "Stat: Open Locks Modifier [90]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 91
    Opcode {
        name: "Stat: Find Traps Modifier [91]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 92
    Opcode {
        name: "Stat: Pick Pockets Modifier [92]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 93
    Opcode {
        name: "Stat: Fatigue Modifier [93]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 94
    Opcode {
        name: "Stat: Drunkenness Modifier [94]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 95
    Opcode {
        name: "Stat: Tracking Skill Modifier [95]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 96
    Opcode {
        name: "Stat: Level Change [96]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Level = Level + 'Statistic Modifier' value
1   Flat Value Modifier -> Level = 'Statistic Modifier' value
2   Percentage Modifier -> Level = (Level * 'Statistic Modifier' value) / 100"#,
    },
    // 97
    Opcode {
        name: "Stat: Strength-Bonus Modifier [97]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Bonus = Bonus + 'Statistic Modifier' value
1   Flat Value Modifier -> Bonus = 'Statistic Modifier' value
2   Percentage Modifier -> Bonus = (Bonus * 'Statistic Modifier' value) / 100"#,
    },
    // 98
    Opcode {
        name: "HP: Regeneration [98]",
        param1: "Regeneration Amount",
        param2: "Type",
        description: r#"Restores HP to the value of 'Regeneration Amount' in the style specified by the 'Type' field, to the targeted creature(s).

Known values for 'Type' are:
0   Restore 'Regeneration Amount' per second
1   Restore 'Regeneration Amount' as a percentage of total HP
2   Restore 'Regeneration Amount' HP every second
3   Restore 1HP every 'Regeneration Amount' seconds
4   Restore Param 3 HP every Param 1 seconds
5+  Crash

Permanent timing modes require a duration for this effect to work."#,
    },
    // 99
    Opcode {
        name: "Spell Effect: Duration Modifier [99]",
        param1: "Duration Modifier",
        param2: "Type",
        description: r#"Alters spell durations of the targeted creature(s), to the percentage specified by the 'Duration Modifier' field, affecting the spell type specified by the 'Type' field. Timing mode 0 is the only timing mode reliably affected. Timings handled through PRO files are ignored.

eg.
'Duration Modifier' = 0   -> Spells have no duration
'Duration Modifier' = 100 -> Spells have normal duration
'Duration Modifier' = 200 -> Spells have double duration

Known values for 'Type' are:
0   Mage
1   Priest"#,
    },
    // 100
    Opcode {
        name: "Protection: from Creature Type [100]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Gives the targeted creature(s) protection from the creature type specified by the 'IDS File' and 'IDS Entry' fields.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids"#,
    },
    // 101
    Opcode {
        name: "Protection: from Opcode [101]",
        param1: "Irrelevant",
        param2: "opcode",
        description: "Gives the targeted creature(s) protection from the opcode specified by the 'opcode' field.",
    },
    // 102
    Opcode {
        name: "Protection: from Spell Levels [102]",
        param1: "Spell Level",
        param2: "Irrelevant",
        description: r#"Gives the targeted creature(s) protection from spells up to the level specified by the 'Spell Level' field. A spell is assigned to a spell level by the power level of the first effect greater than the 'Spell Level' parameter in the extended header that is being called.

Known values for 'Spell Level' are:
1
..
9"#,
    },
    // 103
    Opcode {
        name: "Text: Change Name [103]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: "Alters the name of the targeted creature(s) to the strref specified by the 'String Reference' field. When used with a duration timing, after the duration, the name will be set to a blank entry.",
    },
    // 104
    Opcode {
        name: "Stat: Experience Points [104]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> XP = XP + 'Statistic Modifier' value
1   Flat Value Modifier -> XP = 'Statistic Modifier' value
2   Percentage Modifier -> XP = (XP * 'Statistic Modifier' value) / 100"#,
    },
    // 105
    Opcode {
        name: "Stat: Gold [105]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Gold = Gold - 'Statistic Modifier' value
1   Flat Value Modifier -> Gold = 'Statistic Modifier' value
2   Percentage Modifier -> Gold = (Gold * 'Statistic Modifier' value) / 100

NB. 'Type' 1 can only remove gold. Gold amount has a lower bound of 0."#,
    },
    // 106
    Opcode {
        name: "Stat: Morale Break Modifier [23]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Morale Break = Morale Break + 'Statistic Modifier' value
1   Flat Value Modifier -> Morale Break = 'Statistic Modifier' value
2   Percentage Modifier -> Morale Break = (Morale * 'Statistic Modifier' value) / 100"#,
    },
    // 107
    Opcode {
        name: "Portrait Change [107]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Sets the portrait of the targeted creature(s) to that specified in the resource key, based on the 'Type' value. This effect ignores durations attributed to it.

Known values for 'Type' are:
0   Small portrait
1   Large portrait"#,
    },
    // 108
    Opcode {
        name: "Stat: Reputation [108]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Sets the individual reputation of the targeted creature(s) to the value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.
This does not affect the main reputation of the Party. Though it can affect ranger/paladinhood of any target.

Known values for 'Type' are:
0   Cumulative Modifier -> Reputation = Reputation + 'Statistic Modifier' value
1   Flat Value Modifier -> Reputation = 'Statistic Modifier' value
2   Percentage Modifier -> Reputation = (Reputation * 'Statistic Modifier' value) / 100"#,
    },
    // 109
    Opcode {
        name: "State: Hold [109]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Applies the hold effect to the targeted creature(s), if the targeted creature(s) match the value specified by the 'IDS Entry' field, in the specified 'IDS File'.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids

The affected creature becomes immobile and its AI scripts stop running.

This effect sets stat #58 (HELD) to 1. It also sets the STATE_HELPLESS flag."#,
    },
    // 110
    Opcode {
        name: "Empty: [110]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 111
    Opcode {
        name: "Item: Create Magical Weapon [111]",
        param1: "Amount",
        param2: "Irrelevant",
        description: "Creates the item specified by the resource key. The number of items created is controlled by the 'Amount' field. The item is created in the Magical Weapon slot.",
    },
    // 112
    Opcode {
        name: "Item: Remove Item [112]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes a single instance (or stack) of the item specified by the resource key from the targeted creature(s). Items in container items (e.g. Bags of Holding) will not be removed. If a creature has multiple instances of the item, the one in the lowest slot number (see slots.ids) is removed. EFF_M02 is hard-coded to play when an item is removed. This effect should be used to remove equipped items (use effect 123 for inventory items).",
    },
    // 113
    Opcode {
        name: "Empty: [113]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 114
    Opcode {
        name: "Empty: [114]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 115
    Opcode {
        name: "Detect: Alignment [115]",
        param1: "Irrelevant",
        param2: "Alignment Mask",
        description: r#"Detects the alignment specified by the 'Alignment Mask' field.

Known values for 'Alignment Mask' are:
0   Evil (Glow red)
1   Neutral (Glow blue)
2   Good (Glow Green)"#,
    },
    // 116
    Opcode {
        name: "State: Cure Invisibility [116]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes invisibility from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 117
    Opcode {
        name: "Spell Effect: Reveal Area [117]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Reveals the current area.

NB. This does not remove the fog of war."#,
    },
    // 118
    Opcode {
        name: "Empty: [118]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 119
    Opcode {
        name: "Spell Effect: Mirror Image [119]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the mirror image effect to the targeted creature(s). The number of images is dicated by the level of the caster.",
    },
    // 120
    Opcode {
        name: "Protection: from Melee Weapons [120]",
        param1: "Enchantment",
        param2: "Type",
        description: r#"Gives the targeted creature(s) immunity to melee weapons as specified by the 'Type' field. The 'Enchantment' field is used if 'Type' is set to 0.

Known values for 'Type' are:
0  Enchantment
1  Magical
2  Non-Magical
3  Silver
4  Non-Silver
5  Silver / Non-Magical
6  Two-Handed
7  Non Two-Handed
8  Cursed
9  Non-Cursed
10 Cold-Iron
11 Non Cold-Iron"#,
    },
    // 121
    Opcode {
        name: "Empty: [121]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 122
    Opcode {
        name: "Item: Create Inventory Item [122]",
        param1: "Charges",
        param2: "Irrelevant",
        description: "Creates the item specified by the resource key. The number of items created is controlled by the 'Charges' field. The number of secondary charge is determined by P3. The number of tertiary charges is determined by P4 (external eff's). The item is created in the Inventory.",
    },
    // 123
    Opcode {
        name: "Item: Remove Inventory Item [123]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the item specified by the resource key from the targeted creature(s).  This effect should be used to remove inventory items (use effect 112 for equipped items).",
    },
    // 124
    Opcode {
        name: "Spell Effect: Teleport (Dimension Door) [124]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Teleports the targeted creature(s).",
    },
    // 125
    Opcode {
        name: "Spell Effect: Unlock (Knock) [125]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the knock effect to the targeted creature(s).",
    },
    // 126
    Opcode {
        name: "Stat: Movement Modifier [126]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

The engine allows values for 'Statistic Modifier' from 0 (immobile) to 255 (instantaneous), though behaviour is only consistent in the range 0 - 30.

Known values for 'Type' are:
0   Cumulative Modifier -> Movement = Movement + 'Statistic Modifier' value
1   Flat Value Modifier -> Movement = 'Statistic Modifier' value
2   Percentage Modifier -> Movement = (Movement * 'Statistic Modifier' value) / 100"#,
    },
    // 127
    Opcode {
        name: "Summon: Monster Summoning [127]",
        param1: "Number",
        param2: "Type",
        description: r#"Summons a number of monsters.
Each creature files has a Creature Power Level number (at 0x18). The amount of monsters summoned by this effect is equal to the 'Number' field / Creature Power Level.
The 'Type' field set which 2da file to pick the creatures to summon from. The 2da files are monsumX and anisumX, where X = Type + 1. 0-4 are allied , 5-9 are hostile. The specific 2da files used are:

0 MONSUM01 Monster Summon 1
1 MONSUM02 Monster Summon 2
2 MONSUM03 Monster Summon 3
3 ANISUM01 Animal Summon 1
4 ANISUM02 Animal Summon 2
5 MONSUM01 Monster Summon 1 Hostile
6 MONSUM02 Monster Summon 2 Hostile
7 MONSUM03 Monster Summon 3 Hostile
8 ANISUM01 Animal Summon 1 Hostile
9 ANISUM02 Animal Summon 2 Hostile

A custom .2da file can be entered in the resource field.

If used from an external effect (.eff), the vvc field controls the bam file played when summoning."#,
    },
    // 128
    Opcode {
        name: "State: Confusion [128]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the confusion effect the targeted creature(s). The creature will run around uncontrollable, randomly attacking.",
    },
    // 129
    Opcode {
        name: "State: Aid [129]",
        param1: "Statistic Modifier",
        param2: "Irrelevant",
        description: "Applies a bless effect, a bonus to HP and a bonus to saving throws to the targeted creature(s). The bonus is a number of 8-sided dice, with the number of dice controlled by the 'Statistic Modifier' value.",
    },
    // 130
    Opcode {
        name: "State: Bless [130]",
        param1: "Modifier",
        param2: "Irrelevant",
        description: "Modifies the THAC0, damage and morale of the target creature(s) by the amount specified by the 'Modifier' parameter.",
    },
    // 131
    Opcode {
        name: "State: Positive Chant [131]",
        param1: "Modifier",
        param2: "Irrelevant",
        description: r#"Applies the chant effect to the targeted creature(s). Chant affects the minimum damage caused by a weapon (up to the normal maximum damage of the weapon) and the attack roll modifier (up to a maximum of 20, excluding natural attack roll modifiers, e.g. high STR, invisibility etc) by 'Statistic Modifier' amount. The effect sets STATE_CHANT.

'Modifier' must be in the [-6,5] range.
Multiple chant effects are not cumulative."#,
    },
    // 132
    Opcode {
        name: "State: Raise Strength, Constitution, & Dexterity Non-Cumulative [132]",
        param1: "Statistic Modifier",
        param2: "Irrelevant",
        description: "Raises the strength, constitution and dexterity of the targeted creature(s) by the value specified by the 'Statistic Modifier' field. A negative value of 'Statistic Modifier' sets the strength, constitution and dexterity of the targeted creature(s) to 25.",
    },
    // 133
    Opcode {
        name: "Spell Effect: Luck Non-Cumulative [133]",
        param1: "Statistic Modifier",
        param2: "Irrelevant",
        description: "Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field to the targetted creatures luck. Luck affects the minimum damage caused by a weapon (up to the normal maximum damage of the weapon) and the attack roll modifier (up to a maximum of 20, excluding natural attack roll modifiers, e.g. high STR, invisibility etc) by 'Statistic Modifier' amount. 'Statistic Modifier' also affects the damge rolls of opponents (in feature blocks, not extension headers). The effect sets STATE_LUCK.",
    },
    // 134
    Opcode {
        name: "State: Petrification [134]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the petrification effect to the targeted creature(s). This effects kills & removes PC's from the party, and ends the game if used on the Protagonist.",
    },
    // 135
    Opcode {
        name: "Graphics: Polymorph into Specific [135]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Polymorphs the targeted creature(s) into the creature specified by the resource key, in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Gain Resistances/statistics (spell-casting disabled)
1   Appearance only

This effect sets stat #59 (POLYMORPHED) to 1 but only if param #2 is 0."#,
    },
    // 136
    Opcode {
        name: "State: Force Visible [136]",
        param1: "Undefined",
        param2: "Undefined",
        description: r#"Renders the targeted creature(s) partially visible (does not remove improved invisibility, but the targeted is visible and attackable, but not targetable by spells).
The effect has no meaning on visible targets.
This effect has no duration."#,
    },
    // 137
    Opcode {
        name: "State: Negative Chant [137]",
        param1: "Modifier",
        param2: "Irrelevant",
        description: r#"Applies the chant effect to the targeted creature(s).
This effect works as the reverse of chant. The effect sets STATE_CHANTBAD."#,
    },
    // 138
    Opcode {
        name: "Graphics: Character Animation Change [138]",
        param1: "Irrelevant",
        param2: "Animation Sequence",
        description: r#"The targeted creature(s) shows the animation sequence specified by the 'Animation Sequence' field.

Known values for 'Type' are (from SEQ.IDS):
0   Attack
1   Awake
2   Cast
3   Conjure
4   Damage
5   Die
6   Turn head
7   Ready
8   Shoot
9   Twitch
10  Walk
11  Attack (Slash)
12  Attack (Backslash)
13  Attack (Jab)
14  Emerge
15  Hide
16  Sleep"#,
    },
    // 139
    Opcode {
        name: "Text: Display String [139]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: "Displays the strref specified by the 'String Reference' value.",
    },
    // 140
    Opcode {
        name: "Graphics: Casting Glow [140]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Creates a 2D animation for casting glow. These effects are usually used for casting glows based on the spell school.

Known values for 'Type' are:
0 .. 8 = Crash
9   Casting Glow Aqua (Necromancy)
10  Casting Glow Blue (Alteration)
11  Casting Glow Gold (Enchantment)
12  Casting Glow Green (Abjuration)
13  Casting Glow Magenta (Illusion)
14  Casting Glow Purple (Conjuration)
15  Casting Glow Red (Invocation)
16  Casting Glow White (Divination)
17+ Crashes"#,
    },
    // 141
    Opcode {
        name: "Graphics: Lighting Effects [141]",
        param1: "Target",
        param2: "Type",
        description: r#"Shows the lighting effect (a .bam file) specified by the 'Type' value at the location specified by 'Target'. These effects are usually used for spell hits based on the spell school.

Known values for 'Target' are:
0      Current Location
Non-0  Target Location

Known values for 'Type' are:
0   aqua SHAIR
1   aqua SHEARTH
2   aqua SHWATER
3   *crash*
4   blue SHAIR
5   blue SHEARTH
6   blue SHWATER
7   *crash*
8   gold SHAIR
9   gold SHEARTH
10  gold SHWATER
11  *crash*
12  green SHAIR
13  green SHEARTH
14  green SHWATER
15  *crash*
16  magenta SHAIR
17  magenta SHEARTH
18  magenta SHWATER
19  *crash*
20  purple SHAIR
21  purple SHEARTH
22  purple SHWATER
23  *crash*
24  red SHAIR
25  red SHEARTH
26  red SHWATER
27  *crash*
28  white SHAIR
29  white SHEARTH
30  white SHWATER
31  *crash*
32  red MUSHROOM
33  grey MUSHROOM
34  green MUSHROOM
35  red PILLAR
36  white PILLAR
37  white SWIRL
38  Shadow Door (SPDIMDR)
39  Finger of Death (SPFDEATH)
40+ *crash*"#,
    },
    // 142
    Opcode {
        name: "Graphics: Display Special Effect Icon [142]",
        param1: "Irrelevant",
        param2: "Icon",
        description: r#"Shows the icon specified by the 'Icon' field on the targeted creature(s) portrait.

All values are drawn directly from STATES.BAM in the GUIBam.bif starting at sequence 65.

Known values for 'Icon' are:
0   Charm
1   Dire Charm
2   Rigid Thinking
3   Confusion
4   Berserk
5   Intoxicated
6   Poisoned
7   Nauseated
8   Blind
9   Protection from Evil
10   Protection from Petrification
11   Protection from Missiles
12   Magic Armour
13   Held
14   Sleep
15   Shielded
16   Protection from Fire
17   Blessed
18   Chant
19   Free Action
20   Barkskin
21   Strength
22   Heroism
23   Invulnerable
24   Protection from Acid
25   Protection from Cold
26   Resist Fire/Cold
27   Protection from Electricity
28   Protection from Magic
29   Protection from Undead
30   Protection from Poison
31   Non-detection
32   Good luck
33   Bad luck
34   Silenced
35   Cursed
36   Panic
37   Resist Fear
38   Haste
39   Fatigue
40   Bard Song
41   Slow
42   Regenerate
43   Domination
44   Hopelessness
45   Greater Malison
46   Spirit Armour
47   Chaos
48   Feebleminded
49   Defensive Harmony
50   Champion's Strength
51   Dying
52   Mind Shield
53   Level Drain
54   Polymorph Self
55   Stun
56   Regeneration
57   Perception
58   Master Thievery
59   Energy Drain (Psionic/Mindflayer)
60   Holy Power
61   Cloak of Fear
62   Iron Skins
63   Magic Resistance
64   Righteous Magic
65   Spell Turning
66   Repulsing Undead
67   Spell Deflection
68   Fire Shield Red
69   Fire Shield Blue
70   Protection from Normal Weapons
71   Protection from Magic Weapons
72   Tenser’s Transformation
73   Protection from Magical Energy
74   Mislead
75   Contingency Active
76   Protected from the Elements
77   Projected Image
78   Maze
79   Imprisonment
80   Stoneskin
81   KAI
82   Called Shot
83   Spell Failure
84   Offensive Stance
85   Defensive Stance
86   Intelligence Drained by Mind Flayer
87   Regenerating
88   Dialog Mode
89   At a Store
90   Negative Plane Protection
91   Ability Score Drained
92   Spell Sequencer Active
93   Protected from Energy
94   Magnetized
95   Able to Poison Weapons
96   Setting Trap
97   Glass Dust
98   Blade Barrier
99   Death Ward
100  Doom
101  Decaying
102  Acid
103  Vocalize
104  Mantle
105  Miscast Magic
106  Magic Resistance Lowered
107  Spell Immunity
108  True Seeing
109  Detecting Traps
110  Improved Haste
111  Spell Trigger
112  Deaf
113  Enfeebled
114  Infravision
115  Friends
116  Shield of the Archons
117  Spell Trap
118  Absolute Immunity
119  Improved Mantle
120  Farsight
121  Globe of Invulnerability
122  Minor Globe of Invulnerability
123  Spell Shield
124  Polymorphed
125  Otiluke’s Resilient Sphere
126  Nauseated
127  Ghost Armor
128  Glitterdust
129  Webbed
130  Unconscious
131  Mental Combat
132  Physical Mirror
133  Repulse Undead
134  Chaotic Commands
135  Draw Upon Holy Might
136  Strength of One
137  Bleeding
138  Barbarian Rage
139  Boon of Lathlander
140  Storm Shield
141  Enraged
142  Stunning Blow
143  Quivering Palm
144  Entangled
145  Grease
146  Smite
147  Hardiness
148  Power Attack
149  Whirlwind Attack
150  Greater Whirlwind
151  Magic Flue
152  Critical Strike
153  Greater Deathblow
154  Deathblow
155  Avoid Death
156  Assassination
157  Evasion
158  Greater Evasion
159  Improved Alacrity
160  Aura of Flaming Death
161  Globe of Blades
162  Improved Chaos Shield*
163  Chaos Shield*
164  Fire Elemental Transformation
165  Earth Elemental Transformation

Note: The engine refuses to show portrait 79 (Imprisonment)."#,
    },
    // 143
    Opcode {
        name: "Item: Create Item in Slot [143]",
        param1: "Slot",
        param2: "Irrelevant",
        description: r#"Creates the item specified by the resource key in the slot specified by the 'Slot' field, on the targeted creature(s).

Known values for 'Slot' (from slots.ids) are:
0   AMULET
1   ARMOR
2   BELT
3   BOOTS
4   CLOAK
5   GAUNTLETS
6   HELMET
7   RING_LEFT
8   RING_RIGHT
9   SHIELD
10  FIST
11  AMMO
15  MISC
35  WEAPON
11  AMMO0
12  AMMO1
13  AMMO2
14  AMMO3
15  MISC0
16  MISC1
17  MISC2
18  MISC3
19  MISC4
20  MISC5
21  MISC6
22  MISC7
23  MISC8
24  MISC9
25  MISC10
26  MISC11
27  MISC12
28  MISC13
29  MISC14
30  MISC15
31  MISC16
32  MISC17
33  MISC18
34  MISC19
35  WEAPON0
36  WEAPON1
37  WEAPON2
38  WEAPON3"#,
    },
    // 144
    Opcode {
        name: "Button: Disable Button [144]",
        param1: "Irrelevant",
        param2: "Button",
        description: r#"Disabled the button specified by the 'Button' field. The resource field acts as a matching key to the resource field of the Enable Button, allowing separate enable/disable reasons.

Known values for 'Button' are:
0   Stealth Button
1   Thieving Skill Button
2   Spell Select Button
3   First Quick Spell Button
4   Second Quick Spell Button
5   Third Quick Spell Button
6   Turn Undead Button
7   Talk Button
8   Use Item Button
9   First Quick Item Button
10  No effect
11  Second Quick Item Button
12  Third Quick Item Button
13  Innate Ability Button"#,
    },
    // 145
    Opcode {
        name: "Spell: Disable Spell Casting Abilities [145]",
        param1: "Irrelevant",
        param2: "Spell Type",
        description: r#"Disables spell casting abilities of the targeted creature(s), in the style specified by the 'Spell Type;' field.

Known values for 'Type' are:
0   Wizard
1   Priest
2   Innate"#,
    },
    // 146
    Opcode {
        name: "Spell: Cast Spell (at Creature) [146]",
        param1: "Casting Level",
        param2: "Type",
        description: r#"Casts the spell specified by the resource key at the level specified by the 'Casting Level' field, in the style specified by the 'Type' field. A 'Casting Level' of 0 casts at the level of the casting creature.

Known values for 'Type' are:
0   Normal Casting
1   Cast Instantly"#,
    },
    // 147
    Opcode {
        name: "Spell: Learn Spell [147]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"The targeted creature(s) tries to learn the spell specified by the resource key.
Learning is successful when 1D100 <= the INT-dependent LEARN_SPELL value in INTBON.2DA.
The LEARN_SPELL value is modified for mage specialists, with +15 for same school spells, and -15 for non-same school spells."#,
    },
    // 148
    Opcode {
        name: "Spell: Cast Spell (at Point) [148]",
        param1: "Casting Level",
        param2: "Type",
        description: r#"Casts the spell specified by the resource key, at the level specified by the 'Casting Level' field, in the style specified by the 'Type' field. A 'Casting Level' of 0 casts at the level of the casting creature.

Known values for 'Type' are:
0   Normal Casting
1   Cast Instantly"#,
    },
    // 149
    Opcode {
        name: "Empty: [149]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This opcode has no effect.",
    },
    // 150
    Opcode {
        name: "Spell Effect: Find Traps [150]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the find traps effect to the targeted creature(s).",
    },
    // 151
    Opcode {
        name: "Summon: Replace Creature [151]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Replaces the targeted creature(s) with creature specified by the resource key, in the style specified by the 'Type' value.

Known values for 'Type' are:
0   Remove target silently
1   Remove target - kill with chunk death (giving experience)
2   Remove target - kill with normal death (giving experience)
3+  Don't remove the target"#,
    },
    // 152
    Opcode {
        name: "Spell Effect: Play Movie [152]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Plays the movie specified by the resource key.

Known movies are:
BG2-
  BISLOGO
  BWDRAGON
  DAYNITE
  DEATHAND
  END15FPS
  ENDCRDIT
  FLYTHR01
  FLYTHR02
  FLYTHR03
  FLYTHR04
  INTRO15F
  NITEDAY
  RESTCAMP
  RESTDUNG
  RESTINN
  WOTC

TOB-
  INTRO
  MANAFORG
  MELISSAN
  OUTRO
  SARADUSH
  POCKETZZ"#,
    },
    // 153
    Opcode {
        name: "Overlay: Sanctuary [153]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the sanctuary effect to the targeted creature(s).

This effect sets stat #63 (SANCTUARY) to 1."#,
    },
    // 154
    Opcode {
        name: "Overlay: Entangle [154]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the entangle visual effect to the targeted creature(s). Does not actually prevent the target from moving.

This effect sets stat #62 (ENTANGLE) to 1."#,
    },
    // 155
    Opcode {
        name: "Overlay: Minor Globe [155]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the minor globe visual effect to the targeted creature(s).

This effect sets stat #64 (MINORGLOBE) to 1."#,
    },
    // 156
    Opcode {
        name: "Overlay: Protection from Normal Missiles Cylinder [156]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the protection from normal missiles cylinder visual effect to the targeted creature(s).

This effect sets stat #65 (SHIELDGLOBE) to 1."#,
    },
    // 157
    Opcode {
        name: "State: Web Effect [157]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the web effect to the targeted creature(s). The affected creature becomes immobile and its AI scripts stop running.

This effect sets stats #58 (HELD) and #67 (WEB) to 1. It also sets the STATE_HELPLESS flag."#,
    },
    // 158
    Opcode {
        name: "Overlay: Grease [158]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the grease visual effect to the targeted creature(s). Does not actually affect the target's movement rate.

This effect sets stat #66 (GREASE) to 1."#,
    },
    // 159
    Opcode {
        name: "Spell Effect: Mirror Image (Exact Number) [159]",
        param1: "Amount",
        param2: "Irrelevant",
        description: "Applies the mirror image effect to the targeted creature(s), creating a number of images equal to the 'Amount' value.",
    },
    // 160
    Opcode {
        name: "Remove Sanctuary [160]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the sanctuary effect from the targeted creature(s).",
    },
    // 161
    Opcode {
        name: "Cure: Horror [161]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the horror effect from the targeted creature(s).
This effect ignores durations attributed to it."#,
    },
    // 162
    Opcode {
        name: "Cure: Hold [162]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the paralysis effect (opcodes 109 and 175) from the targeted creature(s).

This effect should be used with an instant timing mode."#,
    },
    // 163
    Opcode {
        name: "Protection: Free Action [163]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the free action effect to the targeted creature(s), restoring the creatures movement rate to its default value if the current movement rate value is below the default for the animation type.",
    },
    // 164
    Opcode {
        name: "Cure: Drunkeness [164]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Clears the drunkeness stat (as modified by opcode 94).",
    },
    // 165
    Opcode {
        name: "Spell Effect: Pause Target [165]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the pause effect to the targeted creature(s). Characters are still selectable.

This effect sets stat #70 (CasterHold) to 1."#,
    },
    // 166
    Opcode {
        name: "Stat: Magic Resistance Modifier [166]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Resistance = Resistance + 'Statistic Modifier' value
1   Flat Value Modifier -> Resistance = 'Statistic Modifier' value"#,
    },
    // 167
    Opcode {
        name: "Stat: THAC0 Modifier with Missile Weapons [167]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100"#,
    },
    // 168
    Opcode {
        name: "Summon: Remove Creature [168]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the creature specified by the resource key (giving no experience).",
    },
    // 169
    Opcode {
        name: "Graphics: Prevent Special Effect Icon [169]",
        param1: "Irrelevant",
        param2: "Icon",
        description: r#"Prevents the icon specified by the 'Icon' field from being displayed.

All values are drawn directly from STATES.BAM in the GUIBam.bif starting at sequence 65.

Known values for 'Icon' are:
0   Charm
1   Dire Charm
2   Rigid Thinking
3   Confusion
4   Berserk
5   Intoxicated
6   Poisoned
7   Nauseated
8   Blind
9   Protection from Evil
10   Protection from Petrification
11   Protection from Missiles
12   Magic Armour
13   Held
14   Sleep
15   Shielded
16   Protection from Fire
17   Blessed
18   Chant
19   Free Action
20   Barkskin
21   Strength
22   Heroism
23   Invulnerable
24   Protection from Acid
25   Protection from Cold
26   Resist Fire/Cold
27   Protection from Electricity
28   Protection from Magic
29   Protection from Undead
30   Protection from Poison
31   Non-detection
32   Good luck
33   Bad luck
34   Silenced
35   Cursed
36   Panic
37   Resist Fear
38   Haste
39   Fatigue
40   Bard Song
41   Slow
42   Regenerate
43   Domination
44   Hopelessness
45   Greater Malison
46   Spirit Armour
47   Chaos
48   Feebleminded
49   Defensive Harmony
50   Champion's Strength
51   Dying
52   Mind Shield
53   Level Drain
54   Polymorph Self
55   Stun
56   Regeneration
57   Perception
58   Master Thievery
59   Energy Drain (Psionic/Mindflayer)
60   Holy Power
61   Cloak of Fear
62   Iron Skins
63   Magic Resistance
64   Righteous Magic
65   Spell Turning
66   Repulsing Undead
67   Spell Deflection
68   Fire Shield Red
69   Fire Shield Blue
70   Protection from Normal Weapons
71   Protection from Magic Weapons
72   Tenser’s Transformation
73   Protection from Magical Energy
74   Mislead
75   Contingency Active
76   Protected from the Elements
77   Projected Image
78   Maze
79   Imprisonment
80   Stoneskin
81   KAI
82   Called Shot
83   Spell Failure
84   Offensive Stance
85   Defensive Stance
86   Intelligence Drained by Mind Flayer
87   Regenerating
88   Dialog Mode
89   At a Store
90   Negative Plane Protection
91   Ability Score Drained
92   Spell Sequencer Active
93   Protected from Energy
94   Magnetized
95   Able to Poison Weapons
96   Setting Trap
97   Glass Dust
98   Blade Barrier
99   Death Ward
100  Doom
101  Decaying
102  Acid
103  Vocalize
104  Mantle
105  Miscast Magic
106  Magic Resistance Lowered
107  Spell Immunity
108  True Seeing
109  Detecting Traps
110  Improved Haste
111  Spell Trigger
112  Deaf
113  Enfeebled
114  Infravision
115  Friends
116  Shield of the Archons
117  Spell Trap
118  Absolute Immunity
119  Improved Mantle
120  Farsight
121  Globe of Invulnerability
122  Minor Globe of Invulnerability
123  Spell Shield
124  Polymorphed
125  Otiluke’s Resilient Sphere
126  Nauseated
127  Ghost Armor
128  Glitterdust
129  Webbed
130  Unconscious
131  Mental Combat
132  Physical Mirror
133  Repulse Undead
134  Chaotic Commands
135  Draw Upon Holy Might
136  Strength of One
137  Bleeding
138  Barbarian Rage
139  Boon of Lathlander
140  Storm Shield
141  Enraged
142  Stunning Blow
143  Quivering Palm
144  Entangled
145  Grease
146  Smite
147  Hardiness
148  Power Attack
149  Whirlwind Attack
150  Greater Whirlwind
151  Magic Flue
152  Critical Strike
153  Greater Deathblow
154  Deathblow
155  Avoid Death
156  Assassination
157  Evasion
158  Greater Evasion
159  Improved Alacrity
160  Aura of Flaming Death
161  Globe of Blades
162  Improved Chaos Shield*
163  Chaos Shield*
164  Fire Elemental Transformation
165  Earth Elemental Transformation"#,
    },
    // 170
    Opcode {
        name: "Graphics: Play Damage Animation [170]",
        param1: "Irrelevant",
        param2: "Animation",
        description: r#"Plays the damage animation specified by the 'Animation' field.

Known values for 'Type' are:
0   Blood (behind)
1   Blood (front)
2   Blood (left)
3   Blood (right)
4   Fire 1
5   Fire 2
6   Fire 3
7   Electricity 1
8   Electricity 2
9   Electricity 3"#,
    },
    // 171
    Opcode {
        name: "Spell: Give Ability [171]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Gives the spell specified by the resource key to the targeted creature(s).",
    },
    // 172
    Opcode {
        name: "Spell: Remove Spell [172]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the spell specified by the resource key from the targeted creature(s) spellbook (but does not affect spells already memorised).
Both parameters should be 0 (zero)."#,
    },
    // 173
    Opcode {
        name: "Stat: Set Poison Resistance [173]",
        param1: "Statistic Modifier",
        param2: "Irrelevant",
        description: "Sets the Poison Resistance of the target creature(s) to the value specified by 'Statistic Modifier'.",
    },
    // 174
    Opcode {
        name: "Spell Effect: Play Sound Effect [174]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Plays the sound effect specified by the resource key.

Known Sound Effects are:
EFF_E01    Expire Illusionary Spell
EFF_E02    Expire Abjuration Spell
EFF_E03    Expire Conjuration Spell
EFF_E04    Expire Divination Spell
EFF_E05    Expire Enchantment Spell
EFF_E06    Expire Invocation Spell
EFF_E07    Expire Necromancy Spell
EFF_E08    Expire Alteration Spell

EFF_M02    Wizard Abjuration Spell Effect
EFF_M03    Wizard Conjuration Spell Effect
EFF_M04    Wizard Divination Spell Effect
EFF_M05    Wizard Enchantment Spell Effect
EFF_M07    Wizard Necromancy Spell Effect
EFF_M08    Wizard Alteration Spell Effect
EFF_M09    Dimension Door
EFF_M10    Dispel, Expire Illusionary Spell Effect
EFF_M11A   Expire Protection from Petrification
EFF_M11B   Expire Protection from Normal Missiles
EFF_M11C   Protection from Normal Missiles
EFF_M12A   Minor Globe Spell Effect
EFF_M12B   Expire Minor Globe Spell Effect
EFF_M13    Monster Summoning
EFF_M15    Wizard Hold Person/Undead/Monster
EFF_M16    Confusion Spell Effect
EFF_M18D   Melf's Acid Arrow
EFF_M20    Melf's Minute Meteors Impact
EFF_M20    Fireball Impact
EFF_M26    Animate Dead
EFF_M28    Haste
EFF_M29    Slow
EFF_M31A   Grease Area Effect Duration
EFF_M31B   Grease Start Area Effect
EFF_M31C   Grease Affect Creature
EFF_M32    Blur
EFF_M33    Wizard Illusionary Spell Effect
EFF_M34    Ice Storm
EFF_M35    Tree of Life Parasite Destruction
EFF_M36    Viekang Teleport
EFF_M36A   Skull Trap (untriggered)
EFF_M36B   Skull Trap shatter
EFF_M38    Teleport Without Error
EFF_M41    Disintegrate dust
EFF_M38    ic Armor
EFF_M52    Protection from Acid/Cold/Electricity/Fire
EFF_M53    Spell Protection Removal
EFF_M55    Weapon Protection Effect (e.g. Mantle, etc.)
EFF_M58    Melf's Minute Meteors creation
EFF_M59    Fire
EFF_M60    Stoneskin
EFF_M61    Polymorph/Shapeshift
EFF_M68    Melf's Acid Arrow Impact
EFF_M69    Improved Alacrity/Use Any Item
EFF_M70    Cacofiend, Gate, Summon Fiend
EFF_M71    Power Word: Blind, Kill, Silence, Sleep, Stun
EFF_M71B   Wizard Conjure Elemental Mental Combat
EFF_M71C   Wizard Conjure Elemental Mental Combat (Won)
EFF_M71D   Wizard Conjure Elemental Mental Combat (Lost)
EFF_M73    Summon Hakeashar, Nishruu
EFF_M74    Imprisonment, Maze 1
EFF_M74B   Imprisonment, Maze 2
EFF_M76    True Sight/Seeing
EFF_M77    Wizard Eye
EFF_M79    Spook
EFF_M80    Mislead, Project Image, Simulacrum
EFF_M82    Emotion, Greater Malison, Resist Magic
EFF_M86    Contingency & Sequencer -type spells
EFF_M87    Wizard Rapture of the Father
EFF_M89    Wail of the Banshee
EFF_M90    Timestop (Start)
EFF_M91    Timestop (Expire)
EFF_M92    Mirror Image/Priest Illusionary Spell Effect
EFF_M108   Dragon's Breath
EFF_M110   Bigby's Hand Spells

EFF_P01    Priest Abjuration Spell Effect
EFF_P02    Priest Conjuration Spell Effect
EFF_P03    Priest Divination Spell Effect
EFF_P04    Priest Enchantment Spell Effect
EFF_P05    Priest Invocation Spell Effect
EFF_P06    Priest Necromancy Spell Effect
EFF_P07    Priest Alteration Spell Effect
EFF_P08    Animal Summoning
EFF_P11    Hold Animal/Person (Priest)
EFF_P12    Protection from Evil/Magic Energy/The Elements/Energy
EFF_P13    Miscast Magic
EFF_P14    ic Vitriolic Sphere
EFF_P15    ic Recitation
EFF_P16    Flame Strike
EFF_P18    Raise Dead
EFF_P19    Call Lightning
EFF_P21A   Sanctuary Start
EFF_P21B   Sanctuary Duration
EFF_P21C   Sanctuary Expire
EFF_P22A   Glyph of Warding non-triggered
EFF_P22B   Glyph of Warding Shatter
EFF_P24    BG1 Armor/Shield
EFF_P25    Restoration (Lesser, Greater) casting completion
EFF_P40    Free Action
EFF_P43    Cloak of Fear start/Nature's Beauty
EFF_P51    Remove Fear
EFF_P54    Shillelagh, Spiritual Hammer
EFF_P55    Silence 15' radius
EFF_P56    Barkskin
EFF_P60    Aid
EFF_P61    Ironskins
EFF_P64A   Creature Summoning Flash
EFF_P65    Creature Summoning Circle
EFF_P66    Call Woodland Beings
EFF_P69    Unholy Word
EFF_P70    Holy Word
EFF_P72    Command Word
EFF_P73    Charm/Domination
EFF_P76    Flameblade
EFF_P77    Spiritual Hammer Hit
EFF_P80    Blade Barrier Creation
EFF_P83    Bolt of Glory
EFF_P89    Restoration (Greater, Lesser)
EFF_P90    Regeneration
EFF_P92    Earthquake
EFF_P93    Cause Wounds
EFF_P97    Armor of Faith wings
EFF_P115   Implosion
EFF_P116   Conjure Fallen Deva/Planetar
EFF_P117   Conjure Deva/Planetar"#,
    },
    // 175
    Opcode {
        name: "State: Hold [175]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Applies a yellow fade in graphic effect to the targeted creature(s), and shows the Hold icon, if the targeted creature(s) match the value specified by the 'IDS Entry' field, in the specified 'IDS File'. The affected creature becomes immobile and its AI scripts stop running.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids

This effect sets stat #58 (HELD) to 1. It also sets the STATE_HELPLESS flag."#,
    },
    // 176
    Opcode {
        name: "Stat: Movement Modifier (II) [176]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"(As #126, but unaffected by Free Action).
Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

The engine allows values for 'Statistic Modifier' from 0 (immobile) to 255 (instantaneous), though behaviour is only consistent in the range 0 - 30.

Known values for 'Type' are:
0   Cumulative Modifier -> Movement = Movement + 'Statistic Modifier' value
1   Flat Value Modifier -> Movement = 'Statistic Modifier' value
2   Percentage Modifier -> Movement = (Movement * 'Statistic Modifier' value) / 100"#,
    },
    // 177
    Opcode {
        name: "Use EFF File [177]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Applies the eff file specified by the resource key to the targeted creature(s), if the targeted creature(s) match the value specified by the 'IDS Entry' field, in the specified 'IDS File'.
NB. When using an eff to alter thac0/dmg bonus, set param1 = 0, and param2 = 2.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 178
    Opcode {
        name: "Spell Effect: THAC0 vs. Creature Type Modifier [178]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Alters the targeted creature(s) THAC0 against the creature type specified by the 'IDS Entry' field in the specified 'IDS File'.
The actual modifier value is placed in the Parameter 3 field in an external effect.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 179
    Opcode {
        name: "Spell Effect: Damage vs. Creature Type Modifier [179]",
        param1: "Reference Value",
        param2: "Reference IDS",
        description: r#"Alters the targeted creature(s) damage against the creature type specified by the 'IDS Entry' field in the specified 'IDS File'. The actual modifier value is placed in the Parameter 3 field in an external effect - dice values are ignored.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 180
    Opcode {
        name: "Item: Can't Use Item [180]",
        param1: "String Reference",
        param2: "Undefined",
        description: "The targeted creatures(s) cannot use the item specified by the resource key. The strref specified by the 'String Reference' field is displayed when an attempt to use the item is made.",
    },
    // 181
    Opcode {
        name: "Item: Can't Use Itemtype [181]",
        param1: "Item Type",
        param2: "Restriction",
        description: r#"The targeted creature(s) cannot use the item type specified by the 'Item Type' field for the purpose specified by the 'Restriction' field.

Known values for 'Restriction' are:
0   Equip
1   Use"#,
    },
    // 182
    Opcode {
        name: "Item: Unknown [182]",
        param1: "Unknown",
        param2: "Unknown",
        description: "The use of this effect is unknown, and effect of its parameters are undefined.",
    },
    // 183
    Opcode {
        name: "Item: Apply Effect Itemtype [183]",
        param1: "Undetermined",
        param2: "Type",
        description: r#"Applies the effect specified by the resource key to the targeted creature(s) when an item of the item type specified by the 'Type' field is equipped.

***This effect has never been seen to work.***"#,
    },
    // 184
    Opcode {
        name: "Graphics: Passwall (Don't Jump) [184]",
        param1: "Undefined",
        param2: "Jump",
        description: r#"Allows creatures to be placed in impassable areas, based on the value of the 'Jump' field.

Known values for 'Jump' are:
0   Do Jump from impassable areas
1+   Don't Jump from impassable areas"#,
    },
    // 185
    Opcode {
        name: "State: Hold [185]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Applies a yellow fade in graphic effect to the targeted creature(s), and shows the Hold icon, if the targeted creature(s) match the value specified by the 'IDS Entry' field, in the specified 'IDS File'. The affected creature becomes immobile and its AI scripts stop running.
This effect is not removed by effect 162.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids

This effect sets stat #58 (HELD) to 1. It also sets the STATE_HELPLESS flag."#,
    },
    // 186
    Opcode {
        name: "Script: MoveToArea [186]",
        param1: "Delay",
        param2: "Orientation",
        description: "Marks the targetted creature(s) as travelling to the area specified by the resource key. The creature appears in the specified area after a delay (specified by the 'Delay' field), with the orientation specified by the 'Orientation' field. The creature appears at the point specified by the Caster Point fields (external EFF files only).",
    },
    // 187
    Opcode {
        name: "Script: Store Local Variable [187]",
        param1: "Value",
        param2: "Irrelevant",
        description: r#"This effect stores a various data types, usually a variable in the 'variable' field.

Resistance.param3: Point
Param4: Float
Param1: Int
Param2: Any
Resource1+2+3: String

This effect can only be used in an external effect (ie. an eff file)."#,
    },
    // 188
    Opcode {
        name: "Spell Effect: Aura Cleansing [188]",
        param1: "Irrelevant",
        param2: "Constant",
        description: r#"Set the 'Constant' field to 1 if you want no time needed between spells.

This effect sets stat #76 (AURACLEANSING) to param #2."#,
    },
    // 189
    Opcode {
        name: "Stat: Casting Time Modifier [189]",
        param1: "Speed Modifier",
        param2: "Irrelevant",
        description: r#"Alters the casting time of spells by the value specified by the 'Speed Modifier' field. This effect is cummulative. Positive values decrease the casting time, negative values increase the casting time.

This effect sets stat #77 (MENTALSPEED) to param #1."#,
    },
    // 190
    Opcode {
        name: "Stat: Attack Speed Factor [190]",
        param1: "Speed Modifier",
        param2: "Irrelevant",
        description: r#"Alters the weapon's attack speed by the value specified by the 'Speed Modifier' field. This effect is not cumulative. Attack speed ranges from 0 (good) to 10 (bad). This effect acts as a bonus, so a high 'Speed Modifier' is good.

This effect sets stat #78 (PHYSICALSPEED) to param #1."#,
    },
    // 191
    Opcode {
        name: "Spell: Casting Level Modifier [191]",
        param1: "Modifier",
        param2: "Type",
        description: r#"Applies the value specified by the 'Modifier' field as a bonus to casting spell levels, of a type specified by the 'Type' field.

Known values for 'Type' are:
0  Mage
1  Cleric

This effect modifies stats #79 (CASTINGLEVELBONUSMAGE) or #80 (CASTINGLEVELBONUSCLERIC) depending on param #2."#,
    },
    // 192
    Opcode {
        name: "Spell Effect: Find Familiar [192]",
        param1: "Irrelevant",
        param2: "Upgrade Marker",
        description: r#"Summons a familiar for the Protagonist player. The familiar summoned is based on the alignment of the Protagonist, and values stored in the GAM file.

The 'Upgrade Marker' field is used internally to indicate the familiar has received the ToB upgrades.

Default creatures for familiars are:
Lawful Good     Pseudo Dragon (FAMPSD)
Lawful Neutral  Ferret (FAMFER)
Lawful Evil     Imp (FAMIMP)
Neutral Good    Pseudo Dragon (FAMPSD)
True Neutral    Rabbit (FAMRAB)
Neutral Evil    Dust Mephit (FAMDUST)
Chaotic Good    Fairy Dragon (FAMFAIR)
Chaotic Neutral Cat (FAMCAT)
Chaotic Evil    Quasit (FAMQUAS)"#,
    },
    // 193
    Opcode {
        name: "Spell Effect: Invisible Detection by Script [193]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Allows invisible creatures to be attacked (via scripts) and makes it possible to target improved invisible creatures with spells. The Constant Value parameter should be set to 1 or 2.

This effect sets stat #81 (SEEINVISIBLE) to param #2."#,
    },
    // 194
    Opcode {
        name: "Ignore Dialog Pause [194]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Allows the targeted creature(s) to carry on their actions during dialogs which pause action.
The Constant Value parameter should be set to 1."#,
    },
    // 195
    Opcode {
        name: "Spell Effect: Death Dependent Constitution Loss [195]",
        param1: "HP Modifier",
        param2: "Undefined",
        description: r#"When a creature with this effect applied to it dies, the Protagonist enacts the damage animation, loses 1 Constitution point, and has their HP modified as dictated by the 'HP Modifier' field.
If 'HP Modifier' is positive, Maximum and Current HP are increased by 'HP Modifier' (though the message ' - was immune to my damage' is displayed).
If 'HP Modifier' is negative, Maximum HP is lowered by 'HP Modifier', and Current HP is lowered by twice 'HP Modifier'."#,
    },
    // 196
    Opcode {
        name: "Spell Effect: Familiar Block [196]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"If this effect is found on any creature in the GAM file, the engine will block summoning of more familiars.

The effect should be set with a timing mode of 9."#,
    },
    // 197
    Opcode {
        name: "Spell: Bounce Projectile [197]",
        param1: "Irrelevant",
        param2: "Reflected Projectile",
        description: r#"The targeted creature(s) will reflect the projectile specified by the 'Reflected Projectile' field back to the attacker.

Values for 'Reflected Projectile' are taken from projectl.ids."#,
    },
    // 198
    Opcode {
        name: "Spell: Bounce Opcode [198]",
        param1: "Irrelevant",
        param2: "Opcode",
        description: r#"The targeted creature(s) will reflect attacks containing the opcode specified by the 'Opcode' field.

NB. This can cause a crash if two creatures end up reflecting to each other."#,
    },
    // 199
    Opcode {
        name: "Spell: Bounce Spell Level [199]",
        param1: "Reflected Level",
        param2: "Irrelevant",
        description: "The targeted creature(s) will reflect spells from the spell level specified in the 'Reflected Level' field.",
    },
    // 200
    Opcode {
        name: "Spell: Decrementing Bounce Spells [200]",
        param1: "Amount",
        param2: "Level",
        description: "The targeted creature(s) will reflect a number of spells (specified by the 'Amount' field), from the spell level specified in the 'Level' field. A spell is assigned to a spell level by the power level of the first effect greater than the 'Level' parameter in the extended header that is being called.",
    },
    // 201
    Opcode {
        name: "Spell: Decrementing Spell Immunity [201]",
        param1: "Amount",
        param2: "Level",
        description: "The targeted creature(s) will absorb a number of spells (specified by the 'Amount' field), from the spell level specified in the 'Level' field. A spell is assigned to a spell level by the power level of the first effect greater than the 'Level' parameter in the extended header that is being called.",
    },
    // 202
    Opcode {
        name: "Spell: Bounce Spell School [202]",
        param1: "Irrelevant",
        param2: "Spell School",
        description: r#"The targeted creature(s) will turn spells from the school specified by the 'Spell School' field.

Known values for 'Spell School' are:
1   Abjuration
2   Conjuration
3   Divination
4   Enchantment
5   Illusion
6   Evocation
7   Necromancy
8   Alteration"#,
    },
    // 203
    Opcode {
        name: "Spell: Bounce Secondary Type [203]",
        param1: "Irrelevant",
        param2: "Secondary Type",
        description: r#"The targeted creature(s) will turn spells with a secondary type matching that specified by the 'Secondary Type' field.

Known values for 'Secondary Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 204
    Opcode {
        name: "Spell: Protection from Spell (School) [204]",
        param1: "Irrelevant",
        param2: "Spell School",
        description: r#"Gives the targeted creature(s) immunity to spells from the school specified by the 'Spell School' field.

Known values for 'Spell School' are:
1   Abjuration
2   Conjuration
3   Divination
4   Enchantment
5   Illusion
6   Evocation
7   Necromancy
8   Alteration"#,
    },
    // 205
    Opcode {
        name: "Spell: Protection from Spell (Secondary Type) [205]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Gives the targeted creature(s) immunity to the spell type specified by the 'Type' field.

Known values for 'Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 206
    Opcode {
        name: "Spell: Protection from Spell [206]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: "Gives the targeted creature(s) immunity from the spell specified in the resource key. The strref specified by the 'String Reference' field is displayed when the spell is absorbed.",
    },
    // 207
    Opcode {
        name: "Spell: Bounce Specified Spell [207]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "The targeted creature(s) will reflect the spell specified by the resource key.",
    },
    // 208
    Opcode {
        name: "HP: Minimum Limit [208]",
        param1: "Minimum Value",
        param2: "Irrelevant",
        description: r#"The targeted creature(s) HP value will not drop below the value specified by the 'Minimum Value' field.

This effect sets stat #83 (MINHITPOINTS) to param #1."#,
    },
    // 209
    Opcode {
        name: "Death: Kill 60HP [209]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Instantly kills the targeted creature(s), if the targeted creature(s) has less than 60 HP.",
    },
    // 210
    Opcode {
        name: "Spell Effect: Stun 90HP [210]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the stun effect to the targeted creature(s), if the targeted creature(s) has less than 90 HP.

The duration of the effect is:
1 to 29  HP -> stunned for 4d4 rounds
30 to 59 HP -> stunned for 2d4 rounds
60 to 89 HP -> stunned for 1d4 rounds"#,
    },
    // 211
    Opcode {
        name: "Spell Effect: Imprisonment [211]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the imprisonment effect to the targeted creature(s).

The field at 0xcc can be used to prevent death related effects (e.g. removal from party) running on the targetted creature(s)."#,
    },
    // 212
    Opcode {
        name: "Protection: Freedom [212]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the freedom effect to the targeted creature(s).
The freedom effect frees creatures trapped by the imprisonment effect."#,
    },
    // 213
    Opcode {
        name: "Spell Effect: Maze [213]",
        param1: "Type",
        param2: "Irrelevant",
        description: r#"Applies the maze effect to the targeted creature(s).

Known values for 'Type' are:
0   Use intelligence based duration (via intmod.2da)
1   Use duration field

The duration of the effect is set in intmod.2da, default values are:
0 to 2 int   -> 2d4 rounds
3 to 5 int   -> 1d4 rounds
6 to 8 int   -> 5d4 rounds
9 to 11 int  -> 4d4 rounds
12 to 14 int -> 3d4 rounds
15 to 17 int -> 2d4 rounds
18+ int      -> 1d4 rounds"#,
    },
    // 214
    Opcode {
        name: "Spell Effect: Select Spell [214]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Uses the 2da specified by the resource key to display multiple spell options, based on the 'Type' field.

Known values for 'Type' are:
1   Show only known spells

NB: To make your own 2DA file for, use the SPWI510.2DA as a base."#,
    },
    // 215
    Opcode {
        name: "Graphics: Play 3D Effect [215]",
        param1: "Undetermined",
        param2: "Effect State",
        description: r#"Plays the BAM/VVC specified by the resource key, in the style specified by the 'Effect State' value.

Known values for 'Effect State' are:
0   Play on target (not attached)
1   Play on target (attached)
2   Play on point

Known BAM files are:
acidblob   Ankheg Spit
aciddth    Acid Hit
bloodcr    Blood Splash
bloodl     Blood Splash (Large)
bloodm     Blood Splash (Medium)
bloods     Blood Splash (Small)
burnhand   Burning Hands
cgabjura   Casting Glow (Abjuration)
cgaltera   Casting Glow (Alteration)
cgconjur   Casting Glow (Conjuration)
cgdivina   Casting Glow (Divination)
cgenchan   Casting Glow (Enchantment)
cgillusi   Casting Glow (Illusion)
cginvoca   Casting Glow (Invocation)
cgnecrom   Casting Glow (Necromancy)
colrspry   Colour Spray
explode    Crash
fire       Crash
firebrn    Small Dot of Fire
flml       Bolt Of Flame (Large)
flmm       Small Bolt Of Flame (Medium)
flms       Tiny Bolt Of Flame (Small)
flmstrk    BG Flame Strike
glphwrdh   Glyph of Warding (Part)
glphwrdt   Glyph of Warding (Full)
greased    Blob Of Grease
greaseh    Blob Of Grease
grndglow   Grey round ground Explosion
hlymite    Call Upon Holy Might
magicstn   Magic Stone
minorglb   Minor Globe of Invulnerability
mmagich    Burst of Yellowish Magic (like Bolt Of Glory)
sanctry    Sanctuary
settrap    Spell Icon
shacid     Acid Damage
shair      Green mist rises from head to air above
sharea     Exploding Yellow Ring above head
shearth    Blue drops from chest drop on ground
shglyph    Rubbish
shskull    Rubbish
shwater    Potion of Healing
sklh is    Skull trap (moving)
sklt is    Skull Trap (stationary)
skybolt    Call Lightning
spabjura   Four cornered blue magic joins in middle
spagscho   Unexplainable
spaltera   Coloured magic rising from ground (Like Teleport Field)
spanisum   Opening Plant (find familiar)
sparmour2   Magic helm forms above target
sparmouri   golden armour joins together at chest of caster (armour)
sparmour   snowball like balls swirl around target
spattck1   Globe of magic (Like ruby ray of reversal)
spattck2   swirling small ball of magic (umber hulk confusion)
spbaserd   Golden Light with white rays of light coming from middle
spbasewt   Blue Light with white rays of light coming from middle
spbdbur    Lime green blob of magic
spbdburs   Direct Vertical Strike of greenish blue magic
spbdimsp   Greenish burst of magic upon target
spbehbla   Purplish magic swirling on target
spbehray   Fire like blob
spbhandi   Small ball of burning red fire
spblbbtm   Blade Barrier (bottom half)
spblbtop   Blade Barrier (Top Half)
spbleshi   3 Golden rays converging upon target
spblood    Chunk of flesh when body explodes due to too much damage
spboom     Mushroom like cloud of fire
spbrnhnd   Burning Hands
spburn     Tiny blob of fire
spburns1   Round fire
spcagemz   Maze (Only top 2 grates)
spcallli   Strike of Lightning (not lightning strike)
spccmdsi   Purplish red burst of magic
spccolde   Round of blue ice with a trail of smoke smashing on target
spccoldl   Travelling round blue ice with a trail of smoke
spccwoui   Burst of Bluish light bursting on target
spchlgt1   Chain Lightning
spchlgt2   Chain Lightning
spchlgt3   Chain Lightning
spchchromi Orb of Translucent bluish green magic
spchrorb   Chromatic Orb
spchunks   Ice Chunks (when target is frozen and smashed)
spclcomp   Blue cloud of mist
spclkfri   Blue Cloak forming on target (cloak of fear)
spcloud1   Blue Cloud of mist
spcloud2   A better Blue cloud of mist
spcloud3   Small cloud of blue mist
spcloudi   Burst of orangey red magic
spcmwoui   Blue magic on target increases then shrinks and disappears
spcomend   Tentacle Whip
spcomett   A Purple Comet
spcomstt   A whip
spconfus   Confusion a ring of intertwined birds
spcontin   A large white star forming a book which blasts white light
spcrtwpn   Yellow and white rays circle forming hammer and anvil
spcspra2   Coloured ball of magic
spcspra3   Larger more clearer ball of magic
spcsprai   Same as last one
spcswoui   Blast of greenish yellowey white magic
spdeath3   Rising spirit (Finger of Death)
spdimdr1   Bg open dimension door
spdimdr2   Bg opening dimension door
spdimndr   Bg2 dimension door
spdispma   Swirling balls of blue magic (dispel magic)
spdivina   Five balls of blue magic separating very small
spdusty2   Disintegration dust
spelltin   Rubbish green square
spentaai   Green vine from ground
spentaci   A swarm of green vines
spentaxi   A swarm of vines that disappear after intertwining
spfbsmal   White Ball of fire
spfdeath   Rising Spirit (Finger of Death)
spfdeth3   Rising Spirit (Finger Of Death)
spfearef   Face on ground (Horror)
spfinodi   A dieing Rose
spfire     Flame Damage
spfirebl   Fireball
spfirepl   Burst of fire in middle with a circle of white light leaving
spfiresb   Fire Shield Blue
spfiresr   Fire Shield Red
spfiret    Small Ball of Fire
spfiretr   Better Small Ball Of Fire
spfirimp   Fire Damage
spfirsdi   Burst of Flames
spfirsed   Fire Seed
spfladow   Flame Strike
spflames   Burst of Flames
spflast1   Burst Of Flames (Different)
spflast2   Flame Strike (different)
spfleshs   Swirling white rays (when Irenicus petrifies people)
spflmarr is Flame Arrow (more real)
spflmbld is Rising Sword
spflmsti is Bottom half Of Icewind Dale Style Flame Strike
spflsrin is Central flaming ball ejecting 2 rings of white light
spflstri is Flamestrike with burning bright yellow at bottom
spfracti is a chain breaking in half, spfstrmi is Small meteor swarm
spgenhla is Ray of bluish light with rays of yellow light (Holy Power)
spgenhlb is Ray Above but forming (the one above forms and finishes)
spgflsh1 is Burst of yellow red magic (when monsters unsummon)
spglyphi is Travelling Coloured glyph of warding
spglypti is Floating coloured glyph
spgreaai is Purple oval of magic
spgreaci is same oval of magic but still the one above forms and goes
spgreaxi is Blob of purple goo exploding on target
spgreorb is green Orb
sphandat is A electrical looking hand appearing
sphealin is Healing
sphleft is left Half of Icewind style Heal
sphlhi01 is left half of icewinds Heal
sphlhi02 is Right half of icewinds heal
sphlybls is Aid
sphlysm2 is Lighter Ray of Holy Smite
sphlysm3 is Darker ray of Holy Smite
sphlysmt is Yellow ray of light striking target
sphollyw is A swirl of blue magic forming then exploding into many doves
spholmig is Already formed circle of magic with white rays shining out
spholymt is Yellow ray with bluish mist on side vertically on target
spholywd is Light blue ray of vertical energy with whitish mist on side
sphorpuf is Yellowish puff of smoke
sphorwil is Abi Dalzims horrid wilting
sphright is Right hands side of Icewinds spell heal
spicestm is Ice storm
spillus is a ball of blue magic swirling
spilusio is a ball of blue magic swirling
spinsect is Black swarm of Insects
spiplaai is Brown swarm of insects
spiplari is Brown swarm of insects
spiplaui is Forming swarm of brown insects
splightb is Lightning Bolt
spmagglo is Minor Spell Deflection
spmagmis is Magic Missile
spmetswa is Flaming ball of fire
spmetswm is Meteor Swarm
spmetswn is More Balls of Meteor swarm
spmindat is The swirling ball above head when you are held
spmirroi is A shining blue mirror
spmonsum is Monster Summoning
spnecro1 is Ring of circling reddish purplish magic
spnpoisi is Icewinds style of cure disease
spnwchrm is Green orbs rising from ground to head (charm)
sppfirai is Produce Fire
sppfirri is smaller burst of fire
sppfirxi is larger burst of flame
spplanar is Large yellow orb forming then shattering
sppolymp is Polymorph
spportal is Round portal forming
sppowwrd is One of the red Power Word effects
spprayri is Icewind Dales style of prayer
spprism2 is swirling orb of colour
spprism3 is swirling ball of colour
spprismt is swirling ball of colour
spproimg is Crack in ground forming some round circle
spprotec is Protection from normal weapons
spraised is animate dead
spraisei is growing rose (raise dead)
sprdrabu is Vertical strike of red magic
sprdrae2 is burst of yellow fire like magic
sprdraex is burst of yellow fire like magic
sprdraim is puff of red smoke exploding on target in a ring
sprdrasi is puff of red smoke formed and it stays formed
spresuri is growing silver rose (resurrection)
spring is grey travelling ray
sprmpari is Icewind Style of remove paralysis
sprotect is Protection from evil
sprwra2i is A blue symbol on the ground with rays of blue rising from it
sprwrati is green ray forming and bursting down on target
spsarmour is Forming wings
spschgi is Purple energy with a black ring around it swirling
spscorch is Fire thing again (so many of these)
spscoric is same as above
spsdbur is A ray of misty grey magic vertically striking fast
spsdimpa is Great sparks of white bursting
spsdrbur is a ray of misty grey magic vertically striking
spshadow is Nothing
spshield is Prot normal missiles
spshkimp is Electrical damage effect
spskullt is red eyed skull with a trail of green smoke
spsmkjet is nothing
spsmold is nothing
spsmpuff is white round puff of smoke disappearing
spsnrayi is Sunray
spsohopi is Symbol forming and ejecting a ray of magic (symbol hopeless)
spspaini is Symbol of Pain
spsparks is sparks
spspeart is Bolt of Glory
spspmaze is 4 lines forming a sharp cage of spears
spstink is small puff of smoke rising from ground
spstones is stoneskin
spstrenh is Whiteray shooting down forming a puff of magic on target (lesser restoration)
spstreni is Strength of One
spsturni is Minor Spell Turning
spsumgte is gate (ring of bones)
spsummon is Call Woodland Beings
spsunray is Larger sunray
sptrapbb is Snare Trap
sptrusee is Opening Eye (clairvoyance)
spturni2 is Already formed spell turning
spunhbl2 is unholy smite
spunhbl3 is darker version of above
spunhblt is Red bolt shooting
spunholy is Am orange glow exploding into many doves
spwi124a is Spell Icon
spwi124b is Spell Icon
spwi124c is Spell Icon
spwirbla is Some ring forming a circle with other lines coming from it with blades arching up forming some cage
spwrdfld is Teleport Field
stnkcldd is Stinking Cloud
stnkcldt is Travelling Ball of Stinking Cloud
travel is An orb of swirling magic orb that shoots out to everyone
trbloba is green blob forming and splatting on target
trblobb is green blob forming and splatting on target
trdartst is Shooting knife looking things
trgooyaa is green blob forming and splatting on target
trgooybb is green blob forming and splatting on target
trspeart is Spear rising from ground striking target
trventaa is Bottom part of opening vent
trventbb is top part of opening vent which sprays out mist
webbomb is A burst of webs
webc is entangled in web
webentd  Other version of web
webenth  Entangling web
webentt  Travelling ball of shining web"#,
    },
    // 216
    Opcode {
        name: "Spell Effect: Level Drain [216]",
        param1: "Amount",
        param2: "Irrelevant",
        description: r#"Drains levels from the targeted creature(s) as specified by the 'Amount' field. 'Amount' can be positive or negative, negative values will alter level and spellbook slots, but not experience.

Param3 is used internally to store recalculated max hp.
Param4 is used internally to store recalculated max xp.

This effect sets stat #200 (LEVELDRAIN) to param #1."#,
    },
    // 217
    Opcode {
        name: "Spell Effect: Unconsciousness 20HP [217]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the unconsciousness effect to the targeted creature(s), if the targeted creature(s) has less than 20 HP. The effect always lasts 5 rounds, and ignores the duration field.",
    },
    // 218
    Opcode {
        name: "Protection: Stoneskin [218]",
        param1: "Amount",
        param2: "Irrelevant",
        description: "Gives the targeted creature(s) a number of protections from physical attacks, as specified by the 'Amount' field.",
    },
    // 219
    Opcode {
        name: "Stat: AC vs. Creature Type Modifier [219]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Gives the targeted creature(s) a 2 point bonus AC against the creature type specified by the 'IDS Entry' in the IDS file specified by the 'IDS File' field.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 220
    Opcode {
        name: "Removal: Remove School [220]",
        param1: "Maximum Level",
        param2: "School Type",
        description: r#"Removes spells from the school specified by the 'School Type' field from the targeted creature(s), up to the level specified by the 'Maximum level' field.

Known values for 'School Type' are:
1   Abjurer
2   Conjurer
3   Diviner
4   Enchanter
5   Illusionist
6   Invoker
7   Necromancer
8   Transmuter
9   Generalist"#,
    },
    // 221
    Opcode {
        name: "Removal: Remove Secondary Type [221]",
        param1: "Maximum Level",
        param2: "Secondary Type",
        description: r#"Removes spells with a secondary type matching that specified by he 'Secondary Type' field from the targeted creature(s), up to the level specified by the 'Maximum level' field.

Known values for 'Secondary Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 222
    Opcode {
        name: "Spell Effect: Teleport Field [222]",
        param1: "Max Range",
        param2: "Irrelevant",
        description: r#"Applies the teleport field to the targeted creature(s).
The 'Max Range' field sets the maximum range of the teleporting effect."#,
    },
    // 223
    Opcode {
        name: "Spell Effect: Immunity School Decrement [223]",
        param1: "Max Level",
        param2: "School Type",
        description: r#"Known values for 'School Type' are:
1   Abjurer
2   Conjurer
3   Diviner
4   Enchanter
5   Illusionist
6   Invoker
7   Necromancer
8   Transmuter
9   Generalist"#,
    },
    // 224
    Opcode {
        name: "Cure: Level Drain (Restoration) [224]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Applies the restoration effect to the targeted creature(s), resetting the level of the target.",
    },
    // 225
    Opcode {
        name: "Spell: Reveal Magic [225]",
        param1: "Undefined",
        param2: "Undefined",
        description: "Highlights the targeted creature(s) if they are affected by dispellable magic. The effect appears to be without duration.",
    },
    // 226
    Opcode {
        name: "Spell: Immunity Secondary Type Decrement [226]",
        param1: "Max Level",
        param2: "Secondary Type",
        description: r#"Known values for 'Secondary Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 227
    Opcode {
        name: "Spell: Bounce School Decrement [227]",
        param1: "Max Level",
        param2: "School Type",
        description: r#"Known values for 'School Type' are:
1   Abjurer
2   Conjurer
3   Diviner
4   Enchanter
5   Illusionist
6   Invoker
7   Necromancer
8   Transmuter
9   Generalist"#,
    },
    // 228
    Opcode {
        name: "Spell: Bounce Secondary Type Decrement [228]",
        param1: "Max Level",
        param2: "Secondary Type",
        description: r#"Known values for 'Secondary Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 229
    Opcode {
        name: "Removal: Remove One School [229]",
        param1: "Max Level",
        param2: "School Type",
        description: r#"Removes a single spell effect of the school specified by the 'School Type' field up to the 'Max Level' value, from the targeted creature(s).

Known values for 'School Type' are:
1   Abjurer
2   Conjurer
3   Diviner
4   Enchanter
5   Illusionist
6   Invoker
7   Necromancer
8   Transmuter
9   Generalist"#,
    },
    // 230
    Opcode {
        name: "Removal: Remove One Secondary Type [230]",
        param1: "Max Level",
        param2: "Secondary Type",
        description: r#"Removes one spell effect of the school specified by the 'School Type' field up to the 'Max Level' value, from the targeted creature(s).

Known values for 'Secondary Type' are:
1   Spell Protections
2   Specific Protections
3   Illusionary Protections
4   Magic Attack
5   Divination Attack
6   Conjuration
7   Combat Protections
8   Contingency
9   Battleground
10  Offensive Damage
11  Disabling
12  Combination
13  Non-Combat"#,
    },
    // 231
    Opcode {
        name: "Spell Effect: Time Stop [231]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the time stop effect.
The timestop effect lasts half as long as the Duration specified for the effect."#,
    },
    // 232
    Opcode {
        name: "Spell Effect: Cast Spell on Condition [232]",
        param1: "Target",
        param2: "Condition",
        description: r#"Casts the spell specified by the resource field on the target specified by the 'Target' field, when the condition from the 'Condition' field is true. The conditions are akin to various triggers.

Known values for 'Target' are:
0   Effect Target
1   LastHitter
2   NearestEnemyOf
3   Nearest

Known values for 'Condition' are:
0   HitBy()
1   See(NearestEnemyOf())
2   HPPercentLT(Myself 50)
3   HPPercentLT(Myself 25)
4   HPPercentLT(Myself 10)
5   StateCheck(Myself,STATE_HELPLESS) i.e. unconscious
6   StateCheck(Myself,STATE_POISONED) i.e. poisoned
7   AttackedBy('Target')
8   PersonalSpaceDistance('Target',4)
9   PersonalSpaceDistance('Target',10)
10  Unknown - every round
11  TookDamage()
12  Kill made
13  TimeofDay

For a 'Condition' of 13, the 'Target' parameter must be 0. The time of day should be stored in 0x02c.

Parameter 3 (EFF) activates the portrait icon and immediately triggers the HitBy() trigger"#,
    },
    // 233
    Opcode {
        name: "Stat: Proficiency Modifier [233]",
        param1: "Amount",
        param2: "Type",
        description: r#"Sets the weapon proficiency points for the weapon type specified in the 'Type' field, to the value specified by the 'Amount' field, for the targeted creature(s).

NB: The Proficiency Modifier effect cannot increment proficiency points, it can only SET them.
NB. The Proficiency Modifier cannot decrement the proficiency points.

Known values for 'Type' are:
Axe                          92
Bastard Sword                89
Blackjack                    108
Club                         115
Crossbow                     103
Dagger                       96
Darts                        106
Flail/Morningstar            100
Gun                          109
Halberd                      99
Katana                       94
Long Bow                     104
Long Sword                   90
Mace                         101
Martial Arts                 110
Quarter Staff                102
Scimitar/Wakizashi/Ninja-To  95
Short Bow                    105
Short Sword                  91
Single Weapon Skill          113
Sling                        107
Spear                        98
Sword and Shield Skill       112
Two-Handed Sword             93
Two-Handed Weapon Skill      111
Two Weapon Skill             114
War Hammer                   97
Extra Proficiency 2(Unused)  116
Extra Proficiency 3(Unused)  117
Extra Proficiency 4(Unused)  118
Extra Proficiency 5(Unused)  119
Extra Proficiency 6(Unused)  120
Extra Proficiency 7(Unused)  121
Extra Proficiency 8(Unused)  122
Extra Proficiency 9(Unused)  123
Extra Proficiency 10(Unused) 124
Extra Proficiency 11(Unused) 125
Extra Proficiency 12(Unused) 126
Extra Proficiency 13(Unused) 127
Extra Proficiency 14(Unused) 128
Extra Proficiency 15(Unused) 129
Extra Proficiency 16(Unused) 130
Extra Proficiency 17(Unused) 131
Extra Proficiency 18(Unused) 132
Extra Proficiency 19(Unused) 133
Extra Proficiency 20(Unused) 134"#,
    },
    // 234
    Opcode {
        name: "Spell Effect: Contingency Creation [234]",
        param1: "Maximum Level Usable",
        param2: "Amount/Type",
        description: r#"Creates a contingency.
The 'Maximum Level' field indicates the maximum spell level that can be used in the contingency. The 'Amount/Type' field indicates the number of spells that can be used in the contingency, and the type of the contingency.

Known values for the 'Amount/Type' field are:
First word:
  0  Chain Contingeny
  1  Contingency
  2  Broken Sequencer

Second word: Amount

Note: This opcode only works from party-members."#,
    },
    // 235
    Opcode {
        name: "Spell Effect: Wing Buffet [235]",
        param1: "Speed",
        param2: "Direction",
        description: r#"Applies the wing buffet effect centred on the casting creature(s), buffeting the targeted creature(s) at the speed specified by the 'Speed' field, in the direction specified by the 'Direction' field.

Known values for 'Direction' are:
0   As 1
1   Away from the destination point (specified in EFF file)
2   Away from the source
3   Towards the destination point (specified in EFF file)
4   Towards source

NB. If used on creatures with the passwall effect applied, creatures can cross points marked as light obstacles on the search map.
NB. Some animations are immune to this effect."#,
    },
    // 236
    Opcode {
        name: "Spell Effect: Image Projection [236]",
        param1: "Undetermined",
        param2: "Type",
        description: r#"Creates an image of the caster, in the style specified by the 'Type' field. The type can be accessed via CheckStat, stat ID: PUPPETTYPE.
NB. When creating an image for use as in Mislead, the image will have a script assigned, with scriptname of 'CASTERSCRIPTM' where CASTERERSCRIPT is the death variable of the casting creature.

Known values for 'Type' are:
0   Talkative, uncontrollable
1   Can not attack, can not cast spells (Mislead.spl is cast on the creature)
2   Can attack, can not cast spells (ProjImag.spl is cast on the creature)
3   Can attack, can cast spells (Simulacrs.pl is cast on the creature)
4+  As 0"#,
    },
    // 237
    Opcode {
        name: "Spell Effect: Puppet ID [237]",
        param1: "Master ID",
        param2: "ID Number",
        description: r#"This effect stores information in a Projected Image/Simulacrum. The type can be accessed via CheckStat, stat PUPPETMASTERTYPE. Applying this effect to a non-image creature will set that creature to behave as an image (e.g. image casting, invisibility and talking behaviour).

Known values for 'ID Number' are:
1  Mislead
2  Project Image
3  Simulacrum

The 'Master ID' field stores a value (0-5) representing the owning party member."#,
    },
    // 238
    Opcode {
        name: "Death: Disintegrate [238]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Disintegrates the targeted creature(s), if the targeted creature(s) match the 'IDS Entry' and in the specified 'IDS File', and if the targeted creature(s) hit-dice is less than those specified by the Dice Values.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 239
    Opcode {
        name: "Spell Effect: Farsight [239]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Allows viewing of the current area, based on the 'Type' field.

Known values for 'Type' are:
0   View explored sections only
1   View explored section and unexplored sections."#,
    },
    // 240
    Opcode {
        name: "Graphics: Remove Special Effect Icon [240]",
        param1: "Irrelevant",
        param2: "Icon",
        description: r#"Removes the icon specified by the 'Icon' field.

Known values for 'Icon' are:
0   Charm
1   Dire Charm
2   Rigid Thinking
3   Confusion
4   Berserk
5   Intoxicated
6   Poisoned
7   Nauseated
8   Blind
9   Protection from Evil
10   Protection from Petrification
11   Protection from Missiles
12   Magic Armour
13   Held
14   Sleep
15   Shielded
16   Protection from Fire
17   Blessed
18   Chant
19   Free Action
20   Barkskin
21   Strength
22   Heroism
23   Invulnerable
24   Protection from Acid
25   Protection from Cold
26   Resist Fire/Cold
27   Protection from Electricity
28   Protection from Magic
29   Protection from Undead
30   Protection from Poison
31   Non-detection
32   Good luck
33   Bad luck
34   Silenced
35   Cursed
36   Panic
37   Resist Fear
38   Haste
39   Fatigue
40   Bard Song
41   Slow
42   Regenerate
43   Domination
44   Hopelessness
45   Greater Malison
46   Spirit Armour
47   Chaos
48   Feebleminded
49   Defensive Harmony
50   Champion's Strength
51   Dying
52   Mind Shield
53   Level Drain
54   Polymorph Self
55   Stun
56   Regeneration
57   Perception
58   Master Thievery
59   Energy Drain (Psionic/Mindflayer)
60   Holy Power
61   Cloak of Fear
62   Iron Skins
63   Magic Resistance
64   Righteous Magic
65   Spell Turning
66   Repulsing Undead
67   Spell Deflection
68   Fire Shield Red
69   Fire Shield Blue
70   Protection from Normal Weapons
71   Protection from Magic Weapons
72   Tenser’s Transformation
73   Protection from Magical Energy
74   Mislead
75   Contingency Active
76   Protected from the Elements
77   Projected Image
78   Maze
79   Imprisonment
80   Stoneskin
81   KAI
82   Called Shot
83   Spell Failure
84   Offensive Stance
85   Defensive Stance
86   Intelligence Drained by Mind Flayer
87   Regenerating
88   Dialog Mode
89   At a Store
90   Negative Plane Protection
91   Ability Score Drained
92   Spell Sequencer Active
93   Protected from Energy
94   Magnetized
95   Able to Poison Weapons
96   Setting Trap
97   Glass Dust
98   Blade Barrier
99   Death Ward
100  Doom
101  Decaying
102  Acid
103  Vocalize
104  Mantle
105  Miscast Magic
106  Magic Resistance Lowered
107  Spell Immunity
108  True Seeing
109  Detecting Traps
110  Improved Haste
111  Spell Trigger
112  Deaf
113  Enfeebled
114  Infravision
115  Friends
116  Shield of the Archons
117  Spell Trap
118  Absolute Immunity
119  Improved Mantle
120  Farsight
121  Globe of Invulnerability
122  Minor Globe of Invulnerability
123  Spell Shield
124  Polymorphed
125  Otiluke’s Resilient Sphere
126  Nauseated
127  Ghost Armor
128  Glitterdust
129  Webbed
130  Unconscious
131  Mental Combat
132  Physical Mirror
133  Repulse Undead
134  Chaotic Commands
135  Draw Upon Holy Might
136  Strength of One
137  Bleeding
138  Barbarian Rage
139  Boon of Lathlander
140  Storm Shield
141  Enraged
142  Stunning Blow
143  Quivering Palm
144  Entangled
145  Grease
146  Smite
147  Hardiness
148  Power Attack
149  Whirlwind Attack
150  Greater Whirlwind
151  Magic Flue
152  Critical Strike
153  Greater Deathblow
154  Deathblow
155  Avoid Death
156  Assassination
157  Evasion
158  Greater Evasion
159  Improved Alacrity
160  Aura of Flaming Death
161  Globe of Blades
162  Improved Chaos Shield*
163  Chaos Shield*
164  Fire Elemental Transformation
165  Earth Elemental Transformation"#,
    },
    // 241
    Opcode {
        name: "Charm: Control Creature [241]",
        param1: "Creature ID",
        param2: "Charm Type",
        description: "As opcode #5",
    },
    // 242
    Opcode {
        name: "Cure: Confusion [242]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the confusion, rigid thinking and fear effect from the targeted creature(s).",
    },
    // 243
    Opcode {
        name: "Item: Drain Item Charges [243]",
        param1: "Item Type",
        param2: "Number to Drain",
        description: r#"Magical items matching the 'Item Type' value on the targeted creature(s) will have their charges drained by Number to Drain. Non-recharging items drained of all their charges are destroyed.

Known values for 'Item Type' are:
0 Magical items excluding weapons
1 Magical items including weapons

Charges are only drained from extended headers with of type magical (i.e. 3), for magical, displayable items. Items that do not that recharge are not destroyed. Items flagged as unsellable are not drained."#,
    },
    // 244
    Opcode {
        name: "Spell: Drain Wizard Spell [244]",
        param1: "Number to Drain",
        param2: "Irrelevant",
        description: "Drains spells from a wizards memory. This effect always drains the highest level possible spell first and drains the number set by the Number to Drain parameter.",
    },
    // 245
    Opcode {
        name: "Check For Berserk [245]",
        param1: "Undefined",
        param2: "Undefined",
        description: "This effect enables a 1% chance of the targetted creature(s) going berserk on each attack it makes. This affect sets the berserk stat changes and then attaches affect 246.",
    },
    // 246
    Opcode {
        name: "Spell Effect: Berserking [246]",
        param1: "Undetermined",
        param2: "Undetermined",
        description: "This effect appears to be a combination of opcode #3 and the berserking portrait icon. This affect attaches affect 247.",
    },
    // 247
    Opcode {
        name: "Spell Effect: Attack Nearest Creature [247]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Applies the attack nearest creature effect on the targeted creature(s).
This effect causes the targeted creature(s) to attack the nearest creature. The affected creature becomes berserk and its AI scripts stop running. Grants a +2 bonus to hit and damage with all weapons. Unlike opcode #3, this one can affect enemies as well.
The 'Constant Value' parameter should be non-zero.

This effect sets stat #144 (BERSERKSTAGE2) to param #2. It also sets the STATE_BERSERK flag."#,
    },
    // 248
    Opcode {
        name: "Item: Set Melee Effect [248]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Sets the effect specified by the resource key as a melee effect.
The specified effect will be applied to the target of each successful melee attack.
The resource key references a .eff not a .spl."#,
    },
    // 249
    Opcode {
        name: "Item: Set Ranged Effect [249]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Sets the effect specified by the resource key as a ranged effect.
The specified effect will be applied to the target of each successful ranged attack.
The resource key references a .eff not a .spl."#,
    },
    // 250
    Opcode {
        name: "Spell Effect: Damage Modifier [250]",
        param1: "Damage Modifier",
        param2: "Irrelevant",
        description: "Applies the 'Damage Modifier' value to the damage caused by the targetted creature(s). The effect works as a minimum damage value, i.e. at least 'Damage Modifier' damage will always be caused (on a successful hit).",
    },
    // 251
    Opcode {
        name: "Spell Effect: Change Bard Song Effect [251]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Sets the Bard Song of the targeted creature to the spell specified by the resource key.

Note: The default Bard Song applies the following effects:
  All effects are added to the entire party with timing 4096 (end at ticks) and duration (nGameTime + 100)

  Display Special Effect Icon (0x8E)
  param2 (Icon) = 40 Bard Song

  Morale Modifier (0x17) - Sets the Morale stat of the targetted creature(s) to 10.
  param1 (Ignored) = 10
  param2 (Ignored) = 0

  Cure Horror (0xA1)
  param1 (Irrelevant) = 1
  param2 (Irrelevant) = 0

  Protection from Opcode (0x65)
  param1 (Irrelevant) = 1
  param2 (Opcode) = 24 (State: Horror)"#,
    },
    // 252
    Opcode {
        name: "Spell Effect: Set Trap [252]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Attempts to set the spell specified by the resource key as a trap.
This effect will fail if the targeted creature(s) does not have sufficient points in the 'Set Traps' skill."#,
    },
    // 253
    Opcode {
        name: "Spell Effect: Add Map Marker [253]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: r#"Applies the set map marker effect.
The String Reference parameter is a string reference to the dialog.tlk file that sets what the marker says when the marker is read. The Target of this effect should be set to Self to function correctly.

NB. This effect works in SoA but not ToB"#,
    },
    // 254
    Opcode {
        name: "Spell Effect: Remove Map Marker [254]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: r#"Applies the remove map marker effect. This effect removes the map marker (if any) at the position the effect is cast. The Target of this effect should be set to Pre-Target to function correctly.

NB. This effect works in SoA but not ToB"#,
    },
    // 255
    Opcode {
        name: "Item: Create Inventory Item (days) [255]",
        param1: "Charges",
        param2: "Irrelevant",
        description: r#"Creates the item specified by the resource key. The number of items created is controlled by the 'Charges' field. The number of secondary charge is determined by P3. The number of tertiary charges is determined by P4 (external eff's). The item is created in the Inventory, and has the 'can not steal' bit set.

NB. The difference from opcode #122 is that when used in an external .eff, the duration (from the calling file) is specified in days."#,
    },
    // 256
    Opcode {
        name: "Spell: Spell Sequencer Active [256]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This effect stores the spell resources for an active spell sequencer. The three spell resources are stored in the resource key, VVC field, and eff 3rd parameter.",
    },
    // 257
    Opcode {
        name: "Spell: Spell Sequencer Creation [257]",
        param1: "Maximum Level Usable",
        param2: "Amount",
        description: r#"Creates a spell sequencer.
The 'Maximum Level' field indicates the maximum spell level that can be used in the sequencer. The 'Amount' field indicates the number of spells that can be used in the sequencer.
The filename of a spell containing this effect should be no more than 7 characters."#,
    },
    // 258
    Opcode {
        name: "Spell: Spell Sequencer Activation [258]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Activates a spell sequencer.

The resource key must be set to the filename of the spell which contains the Spell Sequencer Creation effect, which must be an innate ability.
The spell which contains the Spell Sequencer Creation effect can have a maximum filename of 7 characters.
The spell which contains the Spell Sequencer Activation effect must have the same filename, suffixed with the D character.
eg:
  Spell Sequencer Creation filename = SPSS101
  Spell Sequencer Activation filename = SPSS101D"#,
    },
    // 259
    Opcode {
        name: "Protection: Spell Trap [259]",
        param1: "Amount",
        param2: "Level",
        description: "Applies the spell-trap effect to the targeted creature(s), absorbing a number of spells as specified by the 'Amount' field, of the level specified by the 'Level' field, and recalling previously cast spells. A spell is assigned to a spell level by the power level of the first effect greater than the 'Level' parameter in the extended header that is being called.",
    },
    // 260
    Opcode {
        name: "Spell: Spell Sequencer Activation [258]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Activates a spell sequencer at a target point.

The resource key must be set to the filename of the spell which contains the Spell Sequencer Creation effect, which must be an innate ability.
The spell which contains the Spell Sequencer Creation effect can have a maximum filename of 7 characters.
The spell which contains the Spell Sequencer Activation effect must have the same filename, suffixed with the D character.
eg:
  Spell Sequencer Creation filename = SPSS101
  Spell Sequencer Activation filename = SPSS101D"#,
    },
    // 261
    Opcode {
        name: "Spell: Restore Lost Spells [261]",
        param1: "Spell Level",
        param2: "Spell Type",
        description: r#"Applies the restore lost spell effect to the targeted creature(s).
This effect will restore 1 spell to memory, at the level specified by the 'Spell Level' field, and of the type specified by the 'Spell Type' filed. If there is no free slot at the specified level, the effect restores a spell at the next lowest level possible.

Known values for 'Spell Type' are:
0   Wizard / Bard
1   Cleric / Druid"#,
    },
    // 262
    Opcode {
        name: "Stat: Visual Range [262]",
        param1: "Visual Range",
        param2: "Type",
        description: r#"Applies the visual range effect to the targeted creature(s).
This effect modifies the visual range of the targeted creature(s).
The Visual Range parameter seems to be in the 0-15 interval.

Known values for 'Type' are:
0   Cumulative Modifier -> Visual Range = Visual Range + 'Visual Range' value
1   Flat Value Modifier -> Visual Range = 'Visual Range' value"#,
    },
    // 263
    Opcode {
        name: "Stat: Backstab [263]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Backstab = Backstab + 'Statistic Modifier' value
1   Flat Value Modifier -> Backstab = 'Statistic Modifier' value
2   Percentage Modifier -> Backstab = (Backstab * 'Statistic Modifier' value) / 100"#,
    },
    // 264
    Opcode {
        name: "Spell Effect: Drop Weapons in Panic [264]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Applies the drop weapons effect to the targeted creature(s), in the style specified by the 'Type' field.
This effect causes the targeted creature(s) to drop a piece of equipment.
It won't affect items marked as 'critical'

Known values for 'Type' are:
0   Drop random equipment
1   Drop a quickslot item (won't drop the left-hand weapon)."#,
    },
    // 265
    Opcode {
        name: "Script: Modify Global Variable [265]",
        param1: "Value",
        param2: "Type",
        description: r#"Alters the global variable specified by the resource key, in the style specified by the 'Type' field, by the value specified by the 'Value' field.

Known values for 'Type' are:
0   Set Global (to 'Value')
1   Increment Global (by 'Value')

NB. Since the Resource Key can only be 8 characters long maximum, you can not set a Local Variable with this effect this has a name longer than this amount.
NB. You can use the VVC field of external EFF files for an additional 8 characters of variable name.
NB. You can use the 3rd resource field of EFF files for an additional 8 characters.

NB. All timing modes act as permanent for this effect."#,
    },
    // 266
    Opcode {
        name: "Spell: Remove Protection from Spell [266]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the protection from specific spell (as granted by #206) from the targeted creature(s) based on a matching resource key.",
    },
    // 267
    Opcode {
        name: "Text: Protection from Display Specific String [267]",
        param1: "String Reference",
        param2: "Irrelevant",
        description: "Prevents the strref specified by the 'String Reference' field being displayed by the targeted creature(s).",
    },
    // 268
    Opcode {
        name: "Spell Effect: Clear Fog of War (Wizard Eye) [268]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Applies the clear fog of war effect on the targeted creature(s).
The targeted creature(s) will be able to explore like a PC does.

This effect sets stat #148 (EXPLORE) to 1."#,
    },
    // 269
    Opcode {
        name: "Spell Effect: Shake Window [269]",
        param1: "Strength",
        param2: "Undetermined",
        description: "Applies the shake windows effect, with an intensity specified by the 'Strength' field",
    },
    // 270
    Opcode {
        name: "Cure: Unpause Target [270]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the pause effect (#165) from the targeted creature(s).",
    },
    // 271
    Opcode {
        name: "Graphics: Avatar Removal [271]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Removes the animation of the targeted creature(s).",
    },
    // 272
    Opcode {
        name: "Spell: Apply Repeating EFF [272]",
        param1: "Frequency",
        param2: "Type",
        description: r#"Applies the eff specified by the resource key to the targeted creature(s) as specified by the 'Type' field. 'Type' mode 4 makes use of Parameter 3, and so can only be used in external eff files.

Known values for 'Type' are:
0   Apply once per second
1   Crash
2   Apply 'Frequency' times per second
3   Apply once per 'Frequency' seconds
4   Apply Param 3 times per 'Frequency' rounds
5+  Crash

NB. The timings are relative to the movement rate of the affected creature(s). Effects 40 (Slow) and 78 (Disease) doubles the timing rate, effect 16 (Haste) halves the timing rate.
NB. If two or more effects of this type affect a creature, only the shortest duration effect will occur."#,
    },
    // 273
    Opcode {
        name: "Remove: Specific Area Effect(Zone of Sweet Air) [273]",
        param1: "Undetermined",
        param2: "Undetermined",
        description: "Removes projectiles listed in clearair.2da.",
    },
    // 274
    Opcode {
        name: "Spell Effect: Teleport to Target [274]",
        param1: "Undetermined",
        param2: "Undetermined",
        description: "Teleports the affected creature(s) to a random enemy (i.e. party members would teleport to a red circled creature, red circled creatures teleport to the party). If there are no enemies within visual range, no effect will occur.",
    },
    // 275
    Opcode {
        name: "Stat: Hide in Shadows Modifier [275]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 276
    Opcode {
        name: "Stat: Detect Illusion Modifier [276]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 277
    Opcode {
        name: "Stat: Set Traps Modifier [277]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Skill = Skill + 'Statistic Modifier' value
1   Flat Value Modifier -> Skill = 'Statistic Modifier' value
2   Percentage Modifier -> Skill = (Skill * 'Statistic Modifier' value) / 100"#,
    },
    // 278
    Opcode {
        name: "Stat: To Hit Modifier [278]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> To-hit = To-hit + 'Statistic Modifier' value
1   Flat Value Modifier -> To-hit = 'Statistic Modifier' value
2   Percentage Modifier -> To-hit = (To-hit * 'Statistic Modifier' value) / 100

NB. This effect adjusts modified THAC0, rather than the base stat."#,
    },
    // 279
    Opcode {
        name: "Button: Enable Button [279]",
        param1: "Irrelevant",
        param2: "Button",
        description: r#"Enables the button specified by the 'Button Type' field. The resource field acts as a matching key to the resource field of the Enable Button, allowing separate enable/disable reasons.

Known values for 'Button' are:
0   Stealth Button
1   Thieving Skill Button
2   Spell Select Button
3   First Quick Spell Button
4   Second Quick Spell Button
5   Third Quick Spell Button
6   Turn Undead Button
7   Talk Button
8   Use Item Button
9   First Quick Item Button
11  Second Quick Item Button
12  Third Quick Item Button
13  Innate Ability Button"#,
    },
    // 280
    Opcode {
        name: "Spell Effect: Wild Magic [280]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Applies the wild magic effect to the targeted creature(s).
This effect causes other spells to be subject to wild magic throws.

Known values for 'Type' are:
1   Affect next spell cast
2   Affect all spells cast for effect duration"#,
    },
    // 281
    Opcode {
        name: "Stat: Wild Magic [281]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"This effect will alter the wild-surge roll, making the resultant effect either more or less beneficial.
Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Chance = Chance + 'Statistic Modifier' value
1   Flat Value Modifier -> Chance = 'Statistic Modifier' value
2   Percentage Modifier -> Chance = (Chance * 'Statistic Modifier' value) / 100"#,
    },
    // 282
    Opcode {
        name: "Script: Scripting State Modifier [282]",
        param1: "Value",
        param2: "Scripting State",
        description: r#"Modifies Scripting state to the given value. Scripting State range from 0 to 35. The scripting state can be checked via scripts (see stats.ids), subtract 156 from the stat value to get the scripting state.

This effect does not have proper bounds checking, and can therefore be used in unintended ways. For example scripting state 13 equals with the NO_CIRCLE attribute (stat=169), and scripting state 1139 ties to PICKPOCKET (stat=29)."#,
    },
    // 283
    Opcode {
        name: "Use EFF File (Cursed) [283]",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"Applies the eff file specified by the resource key to the targeted creature(s), if the targeted creature(s) match the 'IDS Entry' in the specified 'IDS File'. The eff can be removed via the Remove Curse opcode (in addition to other standard methods).

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids"#,
    },
    // 284
    Opcode {
        name: "Stat: Melee THAC0 Modifier [284]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100
The effect modifies stat 166."#,
    },
    // 285
    Opcode {
        name: "Stat: Melee Weapon Damage Modifier [285]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100
The effect modifies stat 167."#,
    },
    // 286
    Opcode {
        name: "Stat: Missile Weapon Damage Modifier [286]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100
The effect modifies stat 168."#,
    },
    // 287
    Opcode {
        name: "Graphics: Selection Circle Removal [287]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"Removes the selection circle from the targeted creature(s).
The effect modifies stat 169."#,
    },
    // 288
    Opcode {
        name: "Stat: Fist THAC0 Modifier [288]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100
The effect modifies stat 170."#,
    },
    // 289
    Opcode {
        name: "Stat: Fist Damage Modifier [289]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> Damage = Damage + 'Statistic Modifier' value
1   Flat Value Modifier -> Damage = 'Statistic Modifier' value
2   Percentage Modifier -> Damage = (Damage * 'Statistic Modifier' value) / 100
The effect modifies stat 171."#,
    },
    // 290
    Opcode {
        name: "Text: Change Title [290]",
        param1: "String Reference",
        param2: "Position",
        description: r#"Changes the title of the targeted creature(s), to the strref specified by the 'String Reference' field, at the location specified by the 'Position' field.

Known values for 'Position' are:
0   Records Screen information section
1   Various character screens (eg. Inventory and Records Screen)
The effect modifies stats 172 and 173."#,
    },
    // 291
    Opcode {
        name: "Graphics: Disable Visual Effect [291]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Disables the glowing disk from some effect on the targeted creature(s). Does not affect those visual effects which use BAM files - eg. Fire Shield (red).
The Constant Value parameter should be set to 1.
The effect modifies stat 174 to param2."#,
    },
    // 292
    Opcode {
        name: "Protection: Backstab [292]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Makes the targeted creature(s) immune to backstabbing.
The Constant Value parameter should be set to 1.
The effect modifies stat 175 to param2."#,
    },
    // 293
    Opcode {
        name: "Script: Enable Offscreen AI [293]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"The targeted creature(s) will still run scripts when off-screen and out of combat.
The Constant Value parameter should be set to 1.
The effect modifies stat 183 to param2."#,
    },
    // 294
    Opcode {
        name: "Spell Effect: Existance Delay Override [294]",
        param1: "Undefined",
        param2: "Constant Value",
        description: r#"Existance is the verbal constant group from 69 to 74 (or 70-74). These sounds are randomly played (for non global, alive characters, when not in cutscene or dialog). This stat tunes the frequency of these random strings.

Known values for 'Constant Value' are:
-1 - no existance sounds
0  - standard existance sound frequency
Other values - random existance sound frequency (the higher the rarer)
 The effect modifies stat 184, unless param2 is set to 1."#,
    },
    // 295
    Opcode {
        name: "Graphics: Disable Permanent Death [295]",
        param1: "Undefined",
        param2: "Constant Value",
        description: r#"This targeted creature(s) will never suffer a permanent death (and will therefore always remain resurrectable) - exceptions include petrification and freeze-death.
The Constant Value parameter should be set to 1, and timing mode should be 9 (Permanent after Death).
The effect modifies stat 186."#,
    },
    // 296
    Opcode {
        name: "Graphics: Protection from Specific Animation [296]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "Provides the targeted creature(s) with immunity to the animation specified by the resource key",
    },
    // 297
    Opcode {
        name: "Spell Effect: Immunity to Turn Undead [297]",
        param1: "Undefined",
        param2: "Constant Value",
        description: r#"Makes the targeted creature(s) immune to the Turn Undead effect.
Constant Value should be non-zero.
The effect modifies stat 187."#,
    },
    // 298
    Opcode {
        name: "Spell Effect: Execute Script cut250a [298]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"Executes script cut250a.bcs. By default this script teleports the party to the Pocket Plane area (AR4500).
The Constant Value parameter should be set to 1."#,
    },
    // 299
    Opcode {
        name: "Spell Effect: Chaos Shield [299]",
        param1: "Shield Amount",
        param2: "Icon",
        description: r#"This effect defends against wild surges and displays an associated icon as specified by the 'Icon' parameter.

Known values for 'Icon' are:
0  Normal Chaos Shield
1  Improved Chaos Shield

The effect modifies stat 189."#,
    },
    // 300
    Opcode {
        name: "Spell Effect: NPCBump [300]",
        param1: "Unknown",
        param2: "Constant Value",
        description: r#"This effect allows the targetted creature(s) to push other creatures out of the way while moving.
Constant Value should be set to 1.
The effect modifies stat 190 to param 2."#,
    },
    // 301
    Opcode {
        name: "Stat: Critical Hit Modifier [301]",
        param1: "Statistic Modifier",
        param2: "Irrelevant",
        description: r#"This effect improves the Critical Hit chances.
Critical hits will occur on a roll (on a 1d20) greater or equal to:
20 - 'Statistic Modifier'"#,
    },
    // 302
    Opcode {
        name: "Item: Can Use Any Item [302]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"The targeted creature(s) can use any item.
The Constant Value parameter should be set to 1.
The effect modifies stat 191."#,
    },
    // 303
    Opcode {
        name: "Spell Effect: Backstab Every Hit [303]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"The targeted creature(s) will backstab with every hit.
The Constant Value parameter should be set to 1.
The effect modifies stat 192."#,
    },
    // 304
    Opcode {
        name: "Mass Raise Dead [304]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This effect will raise multiple dead PCs and NPCs.",
    },
    // 305
    Opcode {
        name: "Stat: THAC0 Modifier (Off-Hand) [305]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100"#,
    },
    // 306
    Opcode {
        name: "Stat: THAC0 Modifier (On-Hand) [306]",
        param1: "Statistic Modifier",
        param2: "Type",
        description: r#"Applies the modifier value specified by the 'Statistic Modifier' field in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Cumulative Modifier -> THAC0 = THAC0 + 'Statistic Modifier' value
1   Flat Value Modifier -> THAC0 = 'Statistic Modifier' value
2   Percentage Modifier -> THAC0 = (THAC0 * 'Statistic Modifier' value) / 100"#,
    },
    // 307
    Opcode {
        name: "Ranger Tracking Ability [307]",
        param1: "Range",
        param2: "Irrelevant",
        description: r#"Activates the Tracking Ability. A string from tracking.2da is shown, and arrows appear along the screen edge pointing to any creatures within range. The units of the range parameter are unknown.
The arrow effect may be hardcoded to spell SPCL922.

NB: The file 'tracking.2da' lists the string reference that shows what information is learned from using this effect for each area."#,
    },
    // 308
    Opcode {
        name: "Protection: From Tracking [308]",
        param1: "Irrelevant",
        param2: "Constant value",
        description: r#"Protects the targetted creature(s) from the Tracking arrows effect (has no affect on the area string displayed).
The effect modifies stat 195 to the value of param type."#,
    },
    // 309
    Opcode {
        name: "Script: Modify Local Variable [309]",
        param1: "Value",
        param2: "Type",
        description: r#"Set both the local variable specified by the resource key to the value specified by the 'Value' field.

Known values for 'Type' are:
0   Set Local (to 'Value')
1   Increment Local (by 'Value')

NB: Since the Resource Key can only be 8 characters long maximum, you can not set a variable with this Effect this has a name longer than this amount.
You can use the VVC field of external EFF files for an additional 8 characters of variable name.
You can use the 3rd resource field of EFF files for an additional 8 characters."#,
    },
    // 310
    Opcode {
        name: "Protection: from Timestop [310]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"The targeted creature(s) becomes immune to time-stop.
The Constant Value parameter should be set to 1.
The effect modifies stat 197."#,
    },
    // 311
    Opcode {
        name: "Spell: Random Wish Spell [311]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: r#"This effect casts a wish spell (SPWISH01 - SPWISH25), selected based on the casting creatures Wisdom attribute, as outlined below.

SPWISHxx    MinWis    MaxWis
01          10        25
02          10        25
03          10        25
04          10        25
05          0         25
06          10        25
07          0         25
08          15        25
09          0         25
10          0         25
11          0         25
12          0         25
13          0         25
14          0         25
15          15        25
16          0         25
17          0         25
18          0         17
19          0         9
20          0         17
21          0         17
22          0         9
23          0         9
24          0         9
25          0         25"#,
    },
    // 312
    Opcode {
        name: "Crash [312]",
        param1: "Undefined",
        param2: "Undefined",
        description: r#"This effect crashes the game.
This opcode is also known as ImmunityToSequester."#,
    },
    // 313
    Opcode {
        name: "High-Level Ability Denotation [313]",
        param1: "Undetermined",
        param2: "Undetermined",
        description: "This effect denotes that the high level ability specified by the resource key has been gained. Used for pre-requisites/count limits.",
    },
    // 314
    Opcode {
        name: "Spell: Golem Stoneskin [314]",
        param1: "Amount",
        param2: "Irrelevant",
        description: r#"Gives the targeted creature(s) a number of protections from physical attacks, as specified by the 'Amount' field.

NB: Do not use this effect in conjunction with the Juggernaut Golem Manual as it will corrupt the use of that item."#,
    },
    // 315
    Opcode {
        name: "Graphics: Animation Removal [315]",
        param1: "Irrelevant",
        param2: "Constant Value",
        description: r#"This effect will remove the animation of the targeted creature(s).
The Constant Value parameter should be set to a non-zero value.
The effect modifies stat 201."#,
    },
    // 316
    Opcode {
        name: "Spell: Magical Rest [316]",
        param1: "Irrelevant",
        param2: "Irrelevant",
        description: "This effect will refresh spells and remove any fatigue.",
    },
    // 317
    Opcode {
        name: "State: Haste 2 [317]",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Applies the Haste effect, in the style specified by the 'Type' value.

Known values for 'Type' are:
0   Normal Haste
1   Improved Haste
2   Speed haste without attack bonuses

The effect modifies stat 155.
NB. This effect will cancel out the slow opcode."#,
    },
    // 318
    Opcode {
        name: "Unused",
        param1: "Unknown",
        param2: "Unknown",
        description: "Unknown",
    },
    // 319
    Opcode {
        name: "Usability: Item Usability",
        param1: "IDS Entry",
        param2: "IDS File",
        description: r#"This effect restricts the item to creature(s) matching the object identifiers. The string to display when an invalid creature attempts to use the item is held in 0x2c.

Known values for 'IDS File' are:
2   EA.ids
3   General.ids
4   Race.ids
5   Class.ids
6   Specific.ids
7   Gender.ids
8   Align.ids
9   Kit.ids
10  Actor's name
11  Actor's scripting name (8 maximum in resource field)


If 'IDS File' is set to 10, the item is restricted to characters with the name specified by the strref held in parameter1.
If 'IDS File' is set to 11, the item is restricted to characters with the death variable specified by the resource field."#,
    },
    // 320
    Opcode {
        name: "Change Weather",
        param1: "Weather",
        param2: "Irrelevant",
        description: r#"Changes the weather of the current area to the weather specified by 'Weather'.

Known values for 'Weather' are:
0 - Normal
1 - Rain
2 - Snow
3 - Fog (unimplemented)"#,
    },
    // 321
    Opcode {
        name: "Removal: Effects specified by Resource",
        param1: "Irrelevant",
        param2: "Type",
        description: r#"Removes effects from the resource specified by the resouce key, in the style specified by the 'Type' field.

Known values for 'Type' are:
0   Default
1   Equipped effects list only
2   Timed effects list only

If used with a timing mode other than Duration (0) the effect behaves as per effect 206."#,
    },
];
