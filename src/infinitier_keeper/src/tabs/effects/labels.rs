//! The names an effect's coded fields are shown by, in the table and in the
//! editor's dropdowns: its type (opcode), timing mode, target, and whether
//! it can be dispelled and resisted; and, by its type, what its two
//! parameters hold and what it does (EEKeeper's, see [`opcodes`]).

use infinitier_core::resource::cre::TimingMode;

use super::fields::{EffectRef, Field};
use super::opcodes;

/// The timing modes, with EEKeeper's names for them.
const TIMINGS: [(TimingMode, &str); 12] = [
    (TimingMode::InstantLimited, "Duration"),
    (TimingMode::InstantPermanent, "Permanent"),
    (TimingMode::InstantWhileEquipped, "While Equipped"),
    (TimingMode::DelayLimited, "Delayed Duration"),
    (TimingMode::DelayPermanent, "Delayed"),
    (TimingMode::DelayWhileEquipped, "Delayed, Equipped"),
    (
        TimingMode::LimitedAfterDuration,
        "Delayed, Absolute Duration",
    ),
    (TimingMode::PermanentAfterDuration, "Delayed, Permanent"),
    (TimingMode::EquippedAfterDuration, "Equipped, Permanent"),
    (TimingMode::PermanentAfterDeath, "Instant, Permanent"),
    (TimingMode::InstantLimitedTicks, "Duration (Ticks)"),
    (TimingMode::AbsoluteDuration, "Absolute Duration"),
];

/// The targets 0 to 9, by name.
const TARGETS: [&str; 10] = [
    "None",
    "Self",
    "Preset Target",
    "Party",
    "Everyone",
    "Everyone Except Party",
    "Caster Group",
    "Target Group",
    "Everyone Except Self",
    "Original Caster",
];

/// The dispel / resistance bits 0 and 1, by their four values (EEKeeper's
/// names). The others (EE flags) are kept as they are.
const RESISTANCES: [&str; 4] = [
    "Nonmagical",
    "Can Dispel/Cannot Bypass Resistance",
    "Cannot Dispel/Can Bypass Resistance",
    "Can Dispel/Can Bypass Resistance",
];

/// The dispel / resistance bits [`RESISTANCES`] names.
pub const RESISTANCE_BITS: u32 = 0b11;

/// The table's name for an effect's type (EEKeeper's).
pub fn type_text(opcode: u32) -> String {
    opcodes::opcode(opcode).map_or_else(|| format!("Unknown [{opcode}]"), |op| op.name.to_owned())
}

/// The label of the row of parameter `n` (1 or 2) of an effect `opcode`:
/// what EEKeeper says it holds, the name its description calls it by; "not
/// used" where the effect ignores it; just its number where that's unknown.
pub fn parameter_label(opcode: u32, n: u8) -> String {
    let held = opcodes::opcode(opcode).map(|op| if n == 1 { op.param1 } else { op.param2 });
    match held.unwrap_or_default() {
        "Irrelevant" => format!("Parameter {n} (not used)"),
        "" | "Unknown" | "Undefined" | "Undetermined" => format!("Parameter {n}"),
        held => held.to_owned(),
    }
}

/// What an effect `opcode` does, by EEKeeper; empty where it doesn't say.
pub fn description(opcode: u32) -> &'static str {
    opcodes::opcode(opcode).map_or("", |op| op.description)
}

pub fn timing_text(mode: TimingMode) -> String {
    TIMINGS.iter().find(|(each, _)| *each == mode).map_or_else(
        || u32::from(mode).to_string(),
        |(_, name)| (*name).to_owned(),
    )
}

pub fn target_text(target: u32) -> String {
    usize::try_from(target)
        .ok()
        .and_then(|index| TARGETS.get(index))
        .map_or_else(|| target.to_string(), |name| (*name).to_owned())
}

/// A dropdown's choices: each value, and its name.
pub type Choices = Vec<(u32, String)>;

/// The choices of the dropdown `field` (see `Kind::Choice`), and which of
/// them `effect` holds. Its value comes first, by number, when none names
/// it, so it's never lost. `None` for any other field.
pub fn choices(effect: EffectRef<'_>, field: Field) -> Option<(Choices, usize)> {
    let value = effect.number(field)?;
    let mut choices: Choices = match field {
        // By name, as EEKeeper lists them: a category's effects together.
        Field::Type => {
            let mut types: Choices = opcodes::all()
                .map(|(opcode, _)| (opcode, type_option(opcode)))
                .collect();
            types.sort_by_cached_key(|(_, name)| name.to_lowercase());
            types
        }
        Field::Target => (0..)
            .zip(TARGETS)
            .map(|(value, name)| (value, name.to_owned()))
            .collect(),
        Field::Timing => TIMINGS
            .iter()
            .map(|(mode, name)| (u32::from(*mode), (*name).to_owned()))
            // A V1 record keeps its timing in a byte, and the classic
            // games know no timing in ticks.
            .filter(|(mode, _)| {
                matches!(effect, EffectRef::V2(_))
                    || *mode < u32::from(TimingMode::InstantLimitedTicks)
            })
            .collect(),
        Field::Resistance => (0..)
            .zip(RESISTANCES)
            .map(|(value, name)| (value, name.to_owned()))
            .collect(),
        Field::Opcode
        | Field::Power
        | Field::Param1
        | Field::Param2
        | Field::Param3
        | Field::Param4
        | Field::Param5
        | Field::Special
        | Field::Name
        | Field::Projectile
        | Field::FirstApply
        | Field::Duration
        | Field::TimeApplied
        | Field::Probability1
        | Field::Probability2
        | Field::SaveType
        | Field::SaveBonus
        | Field::DiceThrown
        | Field::DiceSides
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
    };
    let value = if field == Field::Resistance {
        value & RESISTANCE_BITS
    } else {
        value
    };
    let index = match choices.iter().position(|(each, _)| *each == value) {
        Some(index) => index,
        None => {
            let name = if field == Field::Type {
                type_option(value)
            } else {
                value.to_string()
            };
            choices.insert(0, (value, name));
            0
        }
    };
    Some((choices, index))
}

/// A type's dropdown option: its name, which always ends with its opcode,
/// so the few the table shares ("Unused") stay apart.
fn type_option(opcode: u32) -> String {
    let name = type_text(opcode);
    let suffix = format!("[{opcode}]");
    if name.ends_with(&suffix) {
        name
    } else {
        format!("{name} {suffix}")
    }
}

#[cfg(test)]
mod tests {
    use infinitier_core::resource::cre::{Effect, EffectV1Body, Resistance};

    use super::*;

    #[test]
    fn the_types_are_by_name_and_each_names_its_opcode() {
        let effect = Effect::permanent(0, 0, 0);
        let (options, index) = choices(EffectRef::V2(&effect), Field::Type).expect("a dropdown");
        assert_eq!(options[index].0, 0);
        assert_eq!(options.len(), opcodes::all().count());
        assert!(
            options
                .iter()
                .all(|(opcode, name)| name.ends_with(&format!("[{opcode}]")))
        );
        let names: Vec<String> = options
            .iter()
            .map(|(_, name)| name.to_lowercase())
            .collect();
        assert!(names.is_sorted());
    }

    #[test]
    fn a_parameter_is_labelled_by_what_it_holds() {
        assert_eq!(parameter_label(0, 1), "AC Modifier");
        assert_eq!(parameter_label(0, 2), "Type");
        assert_eq!(parameter_label(2, 1), "Parameter 1 (not used)");
        assert_eq!(parameter_label(318, 2), "Parameter 2");
        assert_eq!(parameter_label(5000, 1), "Parameter 1");
        assert!(description(0).starts_with("Applies the modifier value"));
        assert_eq!(description(5000), "");
    }

    #[test]
    fn an_unnamed_value_comes_first() {
        let mut effect = Effect::permanent(400, 0, 0);
        effect.target = 12;
        let (options, index) = choices(EffectRef::V2(&effect), Field::Type).expect("a dropdown");
        assert_eq!((index, options[0].0), (0, 400));
        let (options, index) = choices(EffectRef::V2(&effect), Field::Target).expect("a dropdown");
        assert_eq!((index, options[0].1.as_str()), (0, "12"));
    }

    #[test]
    fn a_v1_effect_has_no_timing_in_ticks() {
        let effect = EffectV1Body::permanent(0, 0, 0);
        let (options, _) = choices(EffectRef::V1(&effect), Field::Timing).expect("a dropdown");
        assert!(options.iter().all(|(mode, _)| *mode < 10));
        let effect = Effect::permanent(0, 0, 0);
        let (options, _) = choices(EffectRef::V2(&effect), Field::Timing).expect("a dropdown");
        assert_eq!(options.len(), TIMINGS.len());
    }

    #[test]
    fn the_resistance_dropdown_reads_bits_0_and_1() {
        let mut effect = Effect::permanent(0, 0, 0);
        effect.resistance = Resistance::from_bits_retain(0b1001);
        let (_, index) = choices(EffectRef::V2(&effect), Field::Resistance).expect("a dropdown");
        assert_eq!(index, 1);
    }
}
