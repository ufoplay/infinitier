//! An effect's fields, read and written alike in both of its record
//! formats: the classic 48-byte (V1) one and the 264-byte (V2) one of BG2,
//! the Enhanced Editions and IWD2. A V1 record has fewer fields, some of
//! them narrower (a byte or a word where V2 has a dword).

use infinitier_core::resource::cre::{
    Effect, EffectList, EffectV1, EffectV1Body, EffectV2, Resistance, SaveType, TimingMode,
};

/// A field of an effect. Its place in [`Field::ALL`] is its id in the
/// editor's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Type,
    Opcode,
    Target,
    Power,
    Param1,
    Param2,
    Param3,
    Param4,
    Param5,
    Special,
    Name,
    Projectile,
    FirstApply,
    Timing,
    Duration,
    TimeApplied,
    Probability1,
    Probability2,
    Resistance,
    SaveType,
    SaveBonus,
    DiceThrown,
    DiceSides,
    PrimaryType,
    SecondaryType,
    Resource0,
    Resource1,
    Resource2,
    Resource3,
    ParentType,
    ParentFlags,
    ParentSlot,
    MinLevel,
    MaxLevel,
    CasterLevel,
    CasterX,
    CasterY,
    TargetX,
    TargetY,
}

/// How a field is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Picked by name from a dropdown (see `labels`).
    Choice,
    /// A whole number.
    Number(Number),
    /// A text of at most this many characters.
    Text(usize),
}

/// A number field's width on disk, and how it's shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Number {
    Byte,
    Word,
    /// A dword shown as an unsigned number.
    Unsigned,
    /// A dword shown as a signed one: a modifier (parameter 1, the save
    /// bonus) that may be negative, or a coordinate or slot whose "none"
    /// is -1.
    Signed,
}

impl Number {
    /// The values it takes, as shown.
    pub fn range(self) -> (i64, i64) {
        match self {
            Self::Byte => (0, u8::MAX.into()),
            Self::Word => (0, u16::MAX.into()),
            Self::Unsigned => (0, u32::MAX.into()),
            Self::Signed => (i32::MIN.into(), i32::MAX.into()),
        }
    }

    /// `raw` as shown.
    pub fn show(self, raw: u32) -> i64 {
        match self {
            Self::Signed => i32::from_le_bytes(raw.to_le_bytes()).into(),
            Self::Byte | Self::Word | Self::Unsigned => raw.into(),
        }
    }

    /// The raw value of `text`, a decimal or `0x` hexadecimal number,
    /// kept in range. A dword takes either reading of its bits: -1 for an
    /// unsigned one, 4294967295 for a signed one, both meaning
    /// `0xFFFFFFFF`. `None` for no number.
    pub fn parse(self, text: &str) -> Option<u32> {
        let text = text.trim();
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest.trim_start()),
            None => (false, text),
        };
        let magnitude = match digits
            .strip_prefix("0x")
            .or_else(|| digits.strip_prefix("0X"))
        {
            Some(hex) => i64::from_str_radix(hex, 16).ok()?,
            None => digits.parse::<i64>().ok()?,
        };
        let value = if negative { -magnitude } else { magnitude };
        let bits = |value: i32| u32::from_le_bytes(value.to_le_bytes());
        let negative_i32 = i64::from(i32::MIN)..0;
        let above_i32 = i64::from(i32::MAX) + 1..=i64::from(u32::MAX);
        match self {
            Self::Unsigned if negative_i32.contains(&value) => i32::try_from(value).ok().map(bits),
            Self::Signed if above_i32.contains(&value) => u32::try_from(value).ok(),
            Self::Byte | Self::Word | Self::Unsigned | Self::Signed => {
                let (min, max) = self.range();
                let value = value.clamp(min, max);
                match self {
                    Self::Signed => i32::try_from(value).ok().map(bits),
                    Self::Byte | Self::Word | Self::Unsigned => u32::try_from(value).ok(),
                }
            }
        }
    }
}

/// The text fields, by where they are in a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextSlot {
    Name,
    Resource0,
    Resource1,
    Resource2,
    Resource3,
}

/// A resref's characters.
const RESREF_LEN: usize = 8;
/// The V2 name field's characters.
const NAME_LEN: usize = 32;

impl Field {
    pub const ALL: [Self; 39] = [
        Self::Type,
        Self::Opcode,
        Self::Target,
        Self::Power,
        Self::Param1,
        Self::Param2,
        Self::Param3,
        Self::Param4,
        Self::Param5,
        Self::Special,
        Self::Name,
        Self::Projectile,
        Self::FirstApply,
        Self::Timing,
        Self::Duration,
        Self::TimeApplied,
        Self::Probability1,
        Self::Probability2,
        Self::Resistance,
        Self::SaveType,
        Self::SaveBonus,
        Self::DiceThrown,
        Self::DiceSides,
        Self::PrimaryType,
        Self::SecondaryType,
        Self::Resource0,
        Self::Resource1,
        Self::Resource2,
        Self::Resource3,
        Self::ParentType,
        Self::ParentFlags,
        Self::ParentSlot,
        Self::MinLevel,
        Self::MaxLevel,
        Self::CasterLevel,
        Self::CasterX,
        Self::CasterY,
        Self::TargetX,
        Self::TargetY,
    ];

    pub fn id(self) -> i32 {
        Self::ALL
            .iter()
            .position(|&field| field == self)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    pub fn from_id(id: i32) -> Option<Self> {
        usize::try_from(id)
            .ok()
            .and_then(|index| Self::ALL.get(index))
            .copied()
    }

    /// Where a text field is; `None` for the others.
    fn text_slot(self) -> Option<TextSlot> {
        match self {
            Self::Name => Some(TextSlot::Name),
            Self::Resource0 => Some(TextSlot::Resource0),
            Self::Resource1 => Some(TextSlot::Resource1),
            Self::Resource2 => Some(TextSlot::Resource2),
            Self::Resource3 => Some(TextSlot::Resource3),
            Self::Type
            | Self::Opcode
            | Self::Target
            | Self::Power
            | Self::Param1
            | Self::Param2
            | Self::Param3
            | Self::Param4
            | Self::Param5
            | Self::Special
            | Self::Projectile
            | Self::FirstApply
            | Self::Timing
            | Self::Duration
            | Self::TimeApplied
            | Self::Probability1
            | Self::Probability2
            | Self::Resistance
            | Self::SaveType
            | Self::SaveBonus
            | Self::DiceThrown
            | Self::DiceSides
            | Self::PrimaryType
            | Self::SecondaryType
            | Self::ParentType
            | Self::ParentFlags
            | Self::ParentSlot
            | Self::MinLevel
            | Self::MaxLevel
            | Self::CasterLevel
            | Self::CasterX
            | Self::CasterY
            | Self::TargetX
            | Self::TargetY => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Type => "Type",
            Self::Opcode => "Opcode",
            Self::Target => "Target",
            Self::Power => "Power",
            Self::Param1 => "Parameter 1",
            Self::Param2 => "Parameter 2",
            Self::Param3 => "Parameter 3",
            Self::Param4 => "Parameter 4",
            Self::Param5 => "Parameter 5",
            Self::Special => "Special",
            Self::Name => "Name",
            Self::Projectile => "Projectile",
            Self::FirstApply => "First Apply",
            Self::Timing => "Timing",
            Self::Duration => "Duration",
            Self::TimeApplied => "Time Applied",
            Self::Probability1 => "Probability 1",
            Self::Probability2 => "Probability 2",
            Self::Resistance => "Dispel / Resist",
            Self::SaveType => "Save Type",
            Self::SaveBonus => "Save Bonus",
            Self::DiceThrown => "Dice Thrown",
            Self::DiceSides => "Dice Sides",
            Self::PrimaryType => "Primary Type",
            Self::SecondaryType => "Secondary Type",
            Self::Resource0 => "Resource 0",
            Self::Resource1 => "Resource 1",
            Self::Resource2 => "Resource 2",
            Self::Resource3 => "Resource 3",
            Self::ParentType => "Parent Type",
            Self::ParentFlags => "Parent Flags",
            Self::ParentSlot => "Parent Slot",
            Self::MinLevel => "Min Level",
            Self::MaxLevel => "Max Level",
            Self::CasterLevel => "Caster Level",
            Self::CasterX => "Caster X",
            Self::CasterY => "Caster Y",
            Self::TargetX => "Target X",
            Self::TargetY => "Target Y",
        }
    }
}

/// An effect, in either format.
#[derive(Debug, Clone, Copy)]
pub enum EffectRef<'a> {
    V1(&'a EffectV1Body),
    V2(&'a Effect),
}

/// An effect being edited, a copy of one on the creature or a new one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Draft {
    V1(EffectV1Body),
    V2(Box<Effect>),
}

/// The effects the Effects tab lists, by their index in `list`: every
/// parsed one, which is all the importer reads (it keeps a record as raw
/// bytes only when too short to parse).
pub fn listed(list: &EffectList) -> Vec<(usize, EffectRef<'_>)> {
    match list {
        EffectList::V1(list) => list
            .iter()
            .enumerate()
            .filter_map(|(index, effect)| match effect {
                EffectV1::Effect(e) => Some((index, EffectRef::V1(e))),
                EffectV1::Raw(_) | EffectV1::Proficiency(_) => None,
            })
            .collect(),
        EffectList::V2(list) => list
            .iter()
            .enumerate()
            .filter_map(|(index, effect)| match effect {
                EffectV2::Effect(e) => Some((index, EffectRef::V2(e))),
                EffectV2::Raw(_) | EffectV2::LocalVariable(_) | EffectV2::Proficiency(_) => None,
            })
            .collect(),
    }
}

impl Draft {
    /// A copy of `effect`.
    pub fn of(effect: EffectRef<'_>) -> Self {
        match effect {
            EffectRef::V1(e) => Self::V1(e.clone()),
            EffectRef::V2(e) => Self::V2(Box::new(e.clone())),
        }
    }

    /// A new effect for `list`, in its format: opcode 0, permanent, on the
    /// creature itself (see [`Effect::permanent`]).
    pub fn new_for(list: &EffectList) -> Self {
        match list {
            EffectList::V1(_) => Self::V1(EffectV1Body::permanent(0, 0, 0)),
            EffectList::V2(_) => Self::V2(Box::new(Effect::permanent(0, 0, 0))),
        }
    }

    pub fn as_ref(&self) -> EffectRef<'_> {
        match self {
            Self::V1(e) => EffectRef::V1(e),
            Self::V2(e) => EffectRef::V2(e),
        }
    }

    /// Write the draft into `list`: over the effect at `index`, or after the
    /// last one for `None` (or an `index` past the end). Nothing when the
    /// list is in the other format.
    pub fn store(self, list: &mut EffectList, index: Option<usize>) {
        match (list, self) {
            (EffectList::V1(list), Self::V1(e)) => put(list, index, EffectV1::Effect(e)),
            (EffectList::V2(list), Self::V2(e)) => put(list, index, EffectV2::Effect(e)),
            (EffectList::V1(_), Self::V2(_)) | (EffectList::V2(_), Self::V1(_)) => {}
        }
    }

    /// Set the number `field` holds to `raw`, which fits it (see
    /// [`EffectRef::number`]). Nothing for a field the format lacks.
    pub fn set_number(&mut self, field: Field, raw: u32) {
        let byte = || u8::try_from(raw).unwrap_or(u8::MAX);
        let word = || u16::try_from(raw).unwrap_or(u16::MAX);
        match self {
            Self::V1(e) => match field {
                Field::Type | Field::Opcode => e.opcode = word(),
                Field::Target => e.target = byte(),
                Field::Power => e.power = byte(),
                Field::Param1 => e.param1 = raw,
                Field::Param2 => e.param2 = raw,
                Field::Special => e.special = raw,
                Field::Timing => e.timing_mode = TimingMode::from(raw),
                Field::Duration => e.duration = raw,
                Field::Probability1 => e.probability1 = byte(),
                Field::Probability2 => e.probability2 = byte(),
                Field::Resistance => e.resistance = Resistance::from_bits_retain(raw),
                Field::SaveType => e.save_type = SaveType::from_bits_retain(raw),
                Field::SaveBonus => e.save_bonus = raw,
                Field::DiceThrown => e.dice_thrown = raw,
                Field::DiceSides => e.dice_sides = raw,
                Field::Param3
                | Field::Param4
                | Field::Param5
                | Field::Name
                | Field::Projectile
                | Field::FirstApply
                | Field::TimeApplied
                | Field::PrimaryType
                | Field::SecondaryType
                | Field::Resource0
                | Field::Resource1
                | Field::Resource2
                | Field::Resource3
                | Field::ParentType
                | Field::ParentFlags
                | Field::ParentSlot
                | Field::MinLevel
                | Field::MaxLevel
                | Field::CasterLevel
                | Field::CasterX
                | Field::CasterY
                | Field::TargetX
                | Field::TargetY => {}
            },
            Self::V2(e) => match field {
                Field::Type | Field::Opcode => e.opcode = raw,
                Field::Target => e.target = raw,
                Field::Power => e.power = raw,
                Field::Param1 => e.param1 = raw,
                Field::Param2 => e.param2 = raw,
                Field::Param3 => e.param3 = raw,
                Field::Param4 => e.param4 = raw,
                Field::Param5 => e.param5 = raw,
                Field::Special => e.special = raw,
                Field::Projectile => e.projectile = raw,
                Field::FirstApply => e.first_apply = raw,
                Field::Timing => e.timing_mode = TimingMode::from(raw),
                Field::Duration => e.duration = raw,
                Field::TimeApplied => e.time_applied = raw,
                Field::Probability1 => e.probability1 = word(),
                Field::Probability2 => e.probability2 = word(),
                Field::Resistance => e.resistance = Resistance::from_bits_retain(raw),
                Field::SaveType => e.save_type = SaveType::from_bits_retain(raw),
                Field::SaveBonus => e.save_bonus = raw,
                Field::DiceThrown => e.dice_thrown = raw,
                Field::DiceSides => e.dice_sides = raw,
                Field::PrimaryType => e.primary_type = raw,
                Field::SecondaryType => e.secondary_type = raw,
                Field::ParentType => e.parent_resource_type = raw,
                Field::ParentFlags => e.parent_resource_flags = raw,
                Field::ParentSlot => e.parent_resource_slot = raw,
                Field::MinLevel => e.parent_lowest_level = raw,
                Field::MaxLevel => e.parent_highest_level = raw,
                Field::CasterLevel => e.caster_level = raw,
                Field::CasterX => e.caster_x = raw,
                Field::CasterY => e.caster_y = raw,
                Field::TargetX => e.target_x = raw,
                Field::TargetY => e.target_y = raw,
                Field::Name
                | Field::Resource0
                | Field::Resource1
                | Field::Resource2
                | Field::Resource3 => {}
            },
        }
    }

    /// Set the text `field` holds, cut to its length (see [`Kind::Text`]):
    /// a resref in capitals, as the games write them. Nothing for a field
    /// the format lacks.
    pub fn set_text(&mut self, field: Field, text: &str) {
        let Some(Kind::Text(len)) = self.as_ref().kind(field) else {
            return;
        };
        let text = text.trim().chars().take(len);
        let text: String = if field == Field::Name {
            text.collect()
        } else {
            text.collect::<String>().to_ascii_uppercase()
        };
        let slot = field.text_slot().and_then(|slot| match (self, slot) {
            (Self::V1(e), TextSlot::Resource0) => Some(&mut e.resource),
            (
                Self::V1(_),
                TextSlot::Name | TextSlot::Resource1 | TextSlot::Resource2 | TextSlot::Resource3,
            ) => None,
            (Self::V2(e), TextSlot::Name) => Some(&mut e.variable),
            (Self::V2(e), TextSlot::Resource0) => Some(&mut e.resource),
            (Self::V2(e), TextSlot::Resource1) => Some(&mut e.resource2),
            (Self::V2(e), TextSlot::Resource2) => Some(&mut e.resource3),
            (Self::V2(e), TextSlot::Resource3) => Some(&mut e.parent_resource),
        });
        if let Some(slot) = slot {
            *slot = text;
        }
    }
}

/// Put `effect` at `index` of `list`, over the one there, or last.
fn put<T>(list: &mut Vec<T>, index: Option<usize>, effect: T) {
    match index.and_then(|index| list.get_mut(index)) {
        Some(slot) => *slot = effect,
        None => list.push(effect),
    }
}

impl<'a> EffectRef<'a> {
    /// How `field` is edited in this format; `None` where it has no such
    /// field.
    pub fn kind(self, field: Field) -> Option<Kind> {
        let v1 = matches!(self, Self::V1(_));
        let v2_only = |kind| (!v1).then_some(kind);
        let narrow = |v1_width, v2_width| Some(Kind::Number(if v1 { v1_width } else { v2_width }));
        match field {
            Field::Type | Field::Target | Field::Timing | Field::Resistance => Some(Kind::Choice),
            Field::Resource0 => Some(Kind::Text(RESREF_LEN)),
            Field::Opcode => narrow(Number::Word, Number::Unsigned),
            Field::Power => narrow(Number::Byte, Number::Unsigned),
            Field::Probability1 | Field::Probability2 => narrow(Number::Byte, Number::Word),
            Field::Param1 | Field::SaveBonus => Some(Kind::Number(Number::Signed)),
            Field::Param2
            | Field::Special
            | Field::Duration
            | Field::SaveType
            | Field::DiceThrown
            | Field::DiceSides => Some(Kind::Number(Number::Unsigned)),
            Field::Name => v2_only(Kind::Text(NAME_LEN)),
            Field::Resource1 | Field::Resource2 | Field::Resource3 => {
                v2_only(Kind::Text(RESREF_LEN))
            }
            Field::ParentSlot
            | Field::CasterX
            | Field::CasterY
            | Field::TargetX
            | Field::TargetY => v2_only(Kind::Number(Number::Signed)),
            Field::Param3
            | Field::Param4
            | Field::Param5
            | Field::Projectile
            | Field::FirstApply
            | Field::TimeApplied
            | Field::PrimaryType
            | Field::SecondaryType
            | Field::ParentType
            | Field::ParentFlags
            | Field::MinLevel
            | Field::MaxLevel
            | Field::CasterLevel => v2_only(Kind::Number(Number::Unsigned)),
        }
    }

    /// The number `field` holds (a choice's value too); `None` for a text
    /// field, or one the format lacks.
    pub fn number(self, field: Field) -> Option<u32> {
        Some(match self {
            Self::V1(e) => match field {
                Field::Type | Field::Opcode => e.opcode.into(),
                Field::Target => e.target.into(),
                Field::Power => e.power.into(),
                Field::Param1 => e.param1,
                Field::Param2 => e.param2,
                Field::Special => e.special,
                Field::Timing => e.timing_mode.into(),
                Field::Duration => e.duration,
                Field::Probability1 => e.probability1.into(),
                Field::Probability2 => e.probability2.into(),
                Field::Resistance => e.resistance.bits(),
                Field::SaveType => e.save_type.bits(),
                Field::SaveBonus => e.save_bonus,
                Field::DiceThrown => e.dice_thrown,
                Field::DiceSides => e.dice_sides,
                Field::Param3
                | Field::Param4
                | Field::Param5
                | Field::Name
                | Field::Projectile
                | Field::FirstApply
                | Field::TimeApplied
                | Field::PrimaryType
                | Field::SecondaryType
                | Field::Resource0
                | Field::Resource1
                | Field::Resource2
                | Field::Resource3
                | Field::ParentType
                | Field::ParentFlags
                | Field::ParentSlot
                | Field::MinLevel
                | Field::MaxLevel
                | Field::CasterLevel
                | Field::CasterX
                | Field::CasterY
                | Field::TargetX
                | Field::TargetY => return None,
            },
            Self::V2(e) => match field {
                Field::Type | Field::Opcode => e.opcode,
                Field::Target => e.target,
                Field::Power => e.power,
                Field::Param1 => e.param1,
                Field::Param2 => e.param2,
                Field::Param3 => e.param3,
                Field::Param4 => e.param4,
                Field::Param5 => e.param5,
                Field::Special => e.special,
                Field::Projectile => e.projectile,
                Field::FirstApply => e.first_apply,
                Field::Timing => e.timing_mode.into(),
                Field::Duration => e.duration,
                Field::TimeApplied => e.time_applied,
                Field::Probability1 => e.probability1.into(),
                Field::Probability2 => e.probability2.into(),
                Field::Resistance => e.resistance.bits(),
                Field::SaveType => e.save_type.bits(),
                Field::SaveBonus => e.save_bonus,
                Field::DiceThrown => e.dice_thrown,
                Field::DiceSides => e.dice_sides,
                Field::PrimaryType => e.primary_type,
                Field::SecondaryType => e.secondary_type,
                Field::ParentType => e.parent_resource_type,
                Field::ParentFlags => e.parent_resource_flags,
                Field::ParentSlot => e.parent_resource_slot,
                Field::MinLevel => e.parent_lowest_level,
                Field::MaxLevel => e.parent_highest_level,
                Field::CasterLevel => e.caster_level,
                Field::CasterX => e.caster_x,
                Field::CasterY => e.caster_y,
                Field::TargetX => e.target_x,
                Field::TargetY => e.target_y,
                Field::Name
                | Field::Resource0
                | Field::Resource1
                | Field::Resource2
                | Field::Resource3 => return None,
            },
        })
    }

    /// The text `field` holds; `None` for a number field, or one the format
    /// lacks.
    pub fn text(self, field: Field) -> Option<&'a str> {
        match (self, field.text_slot()?) {
            (Self::V1(e), TextSlot::Resource0) => Some(&e.resource),
            (
                Self::V1(_),
                TextSlot::Name | TextSlot::Resource1 | TextSlot::Resource2 | TextSlot::Resource3,
            ) => None,
            (Self::V2(e), TextSlot::Name) => Some(&e.variable),
            (Self::V2(e), TextSlot::Resource0) => Some(&e.resource),
            (Self::V2(e), TextSlot::Resource1) => Some(&e.resource2),
            (Self::V2(e), TextSlot::Resource2) => Some(&e.resource3),
            (Self::V2(e), TextSlot::Resource3) => Some(&e.parent_resource),
        }
    }

    /// `field`'s number as shown: signed where it may be negative.
    pub fn shown(self, field: Field) -> Option<i64> {
        let raw = self.number(field)?;
        Some(match self.kind(field) {
            Some(Kind::Number(number)) => number.show(raw),
            Some(Kind::Choice | Kind::Text(_)) | None => raw.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dword_takes_either_reading_of_its_bits() {
        assert_eq!(Number::Unsigned.parse("-1"), Some(u32::MAX));
        assert_eq!(Number::Signed.parse("4294967295"), Some(u32::MAX));
        assert_eq!(Number::Signed.parse("-2"), Some(u32::MAX - 1));
        assert_eq!(Number::Signed.show(u32::MAX - 1), -2);
        assert_eq!(Number::Unsigned.parse("0x10"), Some(16));
        assert_eq!(Number::Unsigned.parse(" 12 "), Some(12));
        assert_eq!(Number::Unsigned.parse("x"), None);
    }

    #[test]
    fn a_number_is_kept_in_its_range() {
        assert_eq!(Number::Byte.parse("300"), Some(255));
        assert_eq!(Number::Byte.parse("-5"), Some(0));
        assert_eq!(Number::Word.parse("70000"), Some(65535));
        assert_eq!(Number::Unsigned.parse("99999999999"), Some(u32::MAX));
        assert_eq!(Number::Signed.parse("-99999999999"), Some(0x8000_0000));
    }

    #[test]
    fn every_field_round_trips_in_v2_and_v1_has_its_own() {
        let mut v2 = Draft::V2(Box::new(Effect::permanent(0, 0, 0)));
        let mut v1 = Draft::V1(EffectV1Body::permanent(0, 0, 0));
        for (n, field) in (1..).zip(Field::ALL) {
            for draft in [&mut v2, &mut v1] {
                match draft.as_ref().kind(field) {
                    Some(Kind::Text(_)) => {
                        draft.set_text(field, "spwi112");
                        assert_eq!(draft.as_ref().text(field).map(str::to_owned), {
                            let expected = if field == Field::Name {
                                "spwi112"
                            } else {
                                "SPWI112"
                            };
                            Some(expected.to_owned())
                        });
                    }
                    Some(Kind::Choice | Kind::Number(_)) => {
                        draft.set_number(field, n);
                        assert_eq!(draft.as_ref().number(field), Some(n), "{field:?}");
                    }
                    None => {
                        assert_eq!(draft.as_ref().number(field), None, "{field:?}");
                        assert_eq!(draft.as_ref().text(field), None, "{field:?}");
                    }
                }
            }
        }
        // Type and Opcode are the same number.
        assert_eq!(v2.as_ref().number(Field::Type), Some(2));
    }

    #[test]
    fn a_draft_replaces_its_effect_or_is_added_last() {
        let mut list = EffectList::V2(vec![
            EffectV2::Effect(Box::new(Effect::permanent(1, 0, 0))),
            EffectV2::Effect(Box::new(Effect::permanent(2, 0, 0))),
        ]);
        Draft::V2(Box::new(Effect::permanent(9, 0, 0))).store(&mut list, Some(1));
        Draft::V2(Box::new(Effect::permanent(7, 0, 0))).store(&mut list, None);
        // The other format: ignored.
        Draft::V1(EffectV1Body::permanent(5, 0, 0)).store(&mut list, None);
        let EffectList::V2(effects) = &list else {
            panic!("still V2");
        };
        let opcodes: Vec<u32> = effects.iter().map(EffectV2::opcode).collect();
        assert_eq!(opcodes, [1, 9, 7]);
    }
}
