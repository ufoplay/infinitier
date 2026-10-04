//! Resistances tab for the Slint keeper.
//!
//! Ports the egui keeper's `ui/tabs/resistances/`. The read-only data
//! extraction (`resist_data` + its structs) is duplicated verbatim; the
//! egui view's per-type `DragValue` clamp becomes a parse-and-clamp to the
//! field's integer type on commit. Both engine families are editable:
//!
//! * **AD&D** (V1.0/V1.2/V9.0): combat resistances + AC modifiers +
//!   the five AD&D saving throws.
//! * **IWD2** (V2.2): twelve combat resistances incl. Magic Damage, and
//!   the three d20 saves. `resist_magic` is spell resistance ("Spells").
//!
//! Fields are addressed across the Slint boundary by [`RField::index`]
//! (index into [`RField::ALL`]); commit routes back via
//! [`RField::from_index`].

use infinitier_core::resource::cre::{Cre, CreHeader};

use crate::state::Model;
use crate::ui_model::{card, editable_row};
use crate::{FieldCard, FieldRow};

// ── Read-only extraction (duplicated from the egui data.rs) ───────────

struct Resistances {
    acid: i8,
    cold: i8,
    electricity: i8,
    fire: i8,
    crushing: i8,
    piercing: i8,
    slashing: i8,
    missile: i8,
    magic: i8,
    magic_fire: i8,
    magic_cold: i8,
}

struct SavingThrows {
    paralyze_poison_death: u8,
    rod_staff_wand: u8,
    petrify_polymorph: u8,
    breath: u8,
    spells: u8,
}

struct Iwd2Saves {
    fortitude: u8,
    reflex: u8,
    will: u8,
}

struct AcModifiers {
    slashing: i16,
    missile: i16,
    crushing: i16,
    piercing: i16,
}

enum ResistData {
    Adnd {
        resistances: Resistances,
        saving_throws: SavingThrows,
        ac_modifiers: AcModifiers,
    },
    Iwd2 {
        resistances: Resistances,
        magic_damage: i8,
        saves: Iwd2Saves,
    },
}

fn resist_data(cre: &Cre) -> ResistData {
    macro_rules! resistances {
        ($h:expr) => {
            Resistances {
                acid: $h.resist_acid,
                cold: $h.resist_cold,
                electricity: $h.resist_electricity,
                fire: $h.resist_fire,
                crushing: $h.resist_crushing,
                piercing: $h.resist_piercing,
                slashing: $h.resist_slashing,
                missile: $h.resist_missile,
                magic: $h.resist_magic,
                magic_fire: $h.resist_magic_fire,
                magic_cold: $h.resist_magic_cold,
            }
        };
    }
    macro_rules! ac_modifiers {
        ($h:expr) => {
            AcModifiers {
                slashing: $h.armor_class_slashing_attacks_modifier,
                missile: $h.armor_class_missile_attacks_modifier,
                crushing: $h.armor_class_crushing_attacks_modifier,
                piercing: $h.armor_class_piercing_attacks_modifier,
            }
        };
    }
    macro_rules! adnd {
        ($h:expr) => {
            ResistData::Adnd {
                resistances: resistances!($h),
                saving_throws: SavingThrows {
                    paralyze_poison_death: $h.save_versus_death,
                    rod_staff_wand: $h.save_versus_wands,
                    petrify_polymorph: $h.save_versus_polymorph,
                    breath: $h.save_versus_breath_attacks,
                    spells: $h.save_versus_spells,
                },
                ac_modifiers: ac_modifiers!($h),
            }
        };
    }

    match &cre.header {
        CreHeader::V10(h) => adnd!(h),
        CreHeader::V12(h) => adnd!(h),
        CreHeader::V90(h) => adnd!(h),
        CreHeader::V22(h) => ResistData::Iwd2 {
            resistances: resistances!(h),
            magic_damage: h.resist_magic_damage,
            saves: Iwd2Saves {
                fortitude: h.save_versus_fortitude,
                reflex: h.save_versus_reflex,
                will: h.save_versus_will,
            },
        },
    }
}

// ── Field table (id ↔ setter) ────────────────────────────────────────

/// Every editable Resistances-tab field. The index into [`Self::ALL`] is
/// the id passed across the Slint boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RField {
    // i8 combat resistances
    Acid,
    Cold,
    Electricity,
    Fire,
    Crushing,
    Piercing,
    Slashing,
    Missile,
    Magic,
    MagicFire,
    MagicCold,
    MagicDamage,
    // u8 AD&D saves
    SaveDeath,
    SaveWands,
    SavePolymorph,
    SaveBreath,
    SaveSpells,
    // u8 IWD2 d20 saves
    SaveFortitude,
    SaveReflex,
    SaveWill,
    // i16 per-damage-type AC modifiers
    AcSlashing,
    AcMissile,
    AcCrushing,
    AcPiercing,
}

impl RField {
    const ALL: &'static [RField] = &[
        RField::Acid,
        RField::Cold,
        RField::Electricity,
        RField::Fire,
        RField::Crushing,
        RField::Piercing,
        RField::Slashing,
        RField::Missile,
        RField::Magic,
        RField::MagicFire,
        RField::MagicCold,
        RField::MagicDamage,
        RField::SaveDeath,
        RField::SaveWands,
        RField::SavePolymorph,
        RField::SaveBreath,
        RField::SaveSpells,
        RField::SaveFortitude,
        RField::SaveReflex,
        RField::SaveWill,
        RField::AcSlashing,
        RField::AcMissile,
        RField::AcCrushing,
        RField::AcPiercing,
    ];

    fn index(self) -> i32 {
        Self::ALL
            .iter()
            .position(|&f| f == self)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    fn from_index(id: i32) -> Option<Self> {
        usize::try_from(id)
            .ok()
            .and_then(|i| Self::ALL.get(i).copied())
    }

    /// Parse + clamp to the field's integer type and write to the CRE.
    fn write(self, cre: &mut Cre, raw: &str) {
        match self {
            Self::Acid => write_i8(raw, cre.resist_acid(), |v| cre.set_resist_acid(v)),
            Self::Cold => write_i8(raw, cre.resist_cold(), |v| cre.set_resist_cold(v)),
            Self::Electricity => write_i8(raw, cre.resist_electricity(), |v| {
                cre.set_resist_electricity(v)
            }),
            Self::Fire => write_i8(raw, cre.resist_fire(), |v| cre.set_resist_fire(v)),
            Self::Crushing => write_i8(raw, cre.resist_crushing(), |v| cre.set_resist_crushing(v)),
            Self::Piercing => write_i8(raw, cre.resist_piercing(), |v| cre.set_resist_piercing(v)),
            Self::Slashing => write_i8(raw, cre.resist_slashing(), |v| cre.set_resist_slashing(v)),
            Self::Missile => write_i8(raw, cre.resist_missile(), |v| cre.set_resist_missile(v)),
            Self::Magic => write_i8(raw, cre.resist_magic(), |v| cre.set_resist_magic(v)),
            Self::MagicFire => write_i8(raw, cre.resist_magic_fire(), |v| {
                cre.set_resist_magic_fire(v)
            }),
            Self::MagicCold => write_i8(raw, cre.resist_magic_cold(), |v| {
                cre.set_resist_magic_cold(v)
            }),
            Self::MagicDamage => write_i8(raw, cre.resist_magic_damage().unwrap_or(0), |v| {
                cre.set_resist_magic_damage(v)
            }),
            Self::SaveDeath => write_u8(raw, cre.save_vs_death().unwrap_or(0), |v| {
                cre.set_save_vs_death(v)
            }),
            Self::SaveWands => write_u8(raw, cre.save_vs_wands().unwrap_or(0), |v| {
                cre.set_save_vs_wands(v)
            }),
            Self::SavePolymorph => write_u8(raw, cre.save_vs_polymorph().unwrap_or(0), |v| {
                cre.set_save_vs_polymorph(v)
            }),
            Self::SaveBreath => write_u8(raw, cre.save_vs_breath().unwrap_or(0), |v| {
                cre.set_save_vs_breath(v)
            }),
            Self::SaveSpells => write_u8(raw, cre.save_vs_spells().unwrap_or(0), |v| {
                cre.set_save_vs_spells(v)
            }),
            Self::SaveFortitude => write_u8(raw, cre.save_vs_fortitude().unwrap_or(0), |v| {
                cre.set_save_vs_fortitude(v)
            }),
            Self::SaveReflex => write_u8(raw, cre.save_vs_reflex().unwrap_or(0), |v| {
                cre.set_save_vs_reflex(v)
            }),
            Self::SaveWill => write_u8(raw, cre.save_vs_will().unwrap_or(0), |v| {
                cre.set_save_vs_will(v)
            }),
            Self::AcSlashing => write_i16(raw, cre.ac_slashing_modifier(), |v| {
                cre.set_ac_slashing_modifier(v)
            }),
            Self::AcMissile => write_i16(raw, cre.ac_missile_modifier(), |v| {
                cre.set_ac_missile_modifier(v)
            }),
            Self::AcCrushing => write_i16(raw, cre.ac_crushing_modifier(), |v| {
                cre.set_ac_crushing_modifier(v)
            }),
            Self::AcPiercing => write_i16(raw, cre.ac_piercing_modifier(), |v| {
                cre.set_ac_piercing_modifier(v)
            }),
        }
    }
}

fn write_i8<F: FnOnce(i8)>(raw: &str, current: i8, write: F) {
    let v = match raw.trim().parse::<i32>() {
        Ok(n) => n.clamp(i8::MIN as i32, i8::MAX as i32) as i8,
        Err(_) => current,
    };
    write(v);
}

fn write_u8<F: FnOnce(u8)>(raw: &str, current: u8, write: F) {
    let v = match raw.trim().parse::<i32>() {
        Ok(n) => n.clamp(0, u8::MAX as i32) as u8,
        Err(_) => current,
    };
    write(v);
}

fn write_i16<F: FnOnce(i16)>(raw: &str, current: i16, write: F) {
    let v = match raw.trim().parse::<i32>() {
        Ok(n) => n.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        Err(_) => current,
    };
    write(v);
}

// ── Build + commit ───────────────────────────────────────────────────

fn erow(field: RField, label: &str, value: impl ToString) -> FieldRow {
    editable_row(field.index(), label, &value.to_string())
}

/// Build the Resistances-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(cre) = model.selected_cre() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    match resist_data(cre) {
        ResistData::Adnd {
            resistances: r,
            saving_throws: s,
            ac_modifiers: ac,
        } => {
            let resist_rows = vec![
                erow(RField::Acid, "Acid", r.acid),
                erow(RField::Cold, "Cold", r.cold),
                erow(RField::Electricity, "Electricity", r.electricity),
                erow(RField::Fire, "Fire", r.fire),
                erow(RField::Crushing, "Crushing", r.crushing),
                erow(RField::Piercing, "Piercing", r.piercing),
                erow(RField::Slashing, "Slashing", r.slashing),
                erow(RField::Missile, "Missile", r.missile),
                erow(RField::Magic, "Magic", r.magic),
                erow(RField::MagicFire, "Magic Fire", r.magic_fire),
                erow(RField::MagicCold, "Magic Cold", r.magic_cold),
            ];
            let ac_rows = vec![
                erow(RField::AcSlashing, "Slashing", ac.slashing),
                erow(RField::AcMissile, "Missile", ac.missile),
                erow(RField::AcCrushing, "Crushing", ac.crushing),
                erow(RField::AcPiercing, "Piercing", ac.piercing),
            ];
            let save_rows = vec![
                erow(
                    RField::SaveDeath,
                    "Paralyzation, Poison, Death",
                    s.paralyze_poison_death,
                ),
                erow(RField::SaveWands, "Rod, Staff, Wand", s.rod_staff_wand),
                erow(
                    RField::SavePolymorph,
                    "Petrification, Polymorph",
                    s.petrify_polymorph,
                ),
                erow(RField::SaveBreath, "Breath Weapons", s.breath),
                erow(RField::SaveSpells, "Spells", s.spells),
            ];
            [
                vec![
                    card("Resistances", 0, resist_rows),
                    card("Armor Class Modifiers", 0, ac_rows),
                ],
                vec![card("Saving Throws", 1, save_rows)],
                Vec::new(),
            ]
        }
        ResistData::Iwd2 {
            resistances: r,
            magic_damage,
            saves,
        } => {
            let resist_rows = vec![
                erow(RField::Acid, "Acid", r.acid),
                erow(RField::Cold, "Cold", r.cold),
                erow(RField::Electricity, "Electricity", r.electricity),
                erow(RField::Fire, "Fire", r.fire),
                erow(RField::Crushing, "Crushing", r.crushing),
                erow(RField::Piercing, "Piercing", r.piercing),
                erow(RField::Slashing, "Slashing", r.slashing),
                erow(RField::Missile, "Missile", r.missile),
                erow(RField::MagicFire, "Magic Fire", r.magic_fire),
                erow(RField::MagicCold, "Magic Cold", r.magic_cold),
                // On IWD2 `resist_magic` is spell resistance.
                erow(RField::Magic, "Spells", r.magic),
                erow(RField::MagicDamage, "Magic Damage", magic_damage),
            ];
            let save_rows = vec![
                erow(RField::SaveFortitude, "Fortitude", saves.fortitude),
                erow(RField::SaveReflex, "Reflex", saves.reflex),
                erow(RField::SaveWill, "Will", saves.will),
            ];
            [
                vec![card("Resistances", 0, resist_rows)],
                vec![card("Saving Throws", 1, save_rows)],
                Vec::new(),
            ]
        }
    }
}

/// Commit an edited Resistances field back to the selected CRE.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    let Some(field) = RField::from_index(id) else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        field.write(cre, value);
    }
}
