//! Levels & Kits tab (editable, IWD2 only).
//!
//! Left card: the eleven IWD2
//! per-class levels (reusing the shared `EditableField` class-level fields,
//! whose `write_clamped_cre` keeps the eleven summing to ≤255) plus a
//! read-only Total. Right card: a toggle per IWD2 kit (`Iwd2Kits`), flipping
//! its bit in the CRE `kit_bitfield`.
//!
//! Field-id encoding: `EditableField::index()` for class levels;
//! `KIT_ID_OFFSET + kit_index` for kit toggles.

use infinitier_core::resource::cre::{CreHeader, Iwd2Kits};

use crate::FieldCard;
use crate::components::editable_fields::EditableField;
use crate::components::field_card::{card, editable_check_row, editable_row, readonly_row};
use crate::state::Model;

/// Kit toggle ids start here, clear of the `EditableField` index space.
const KIT_ID_OFFSET: i32 = 1000;

/// The eleven IWD2 class-level fields, sorted by name (as EEKeeper shows).
const LEVEL_FIELDS: &[(EditableField, &str)] = &[
    (EditableField::BarbarianLevel, "Barbarian"),
    (EditableField::BardLevel, "Bard"),
    (EditableField::ClericLevel, "Cleric"),
    (EditableField::DruidLevel, "Druid"),
    (EditableField::FighterLevel, "Fighter"),
    (EditableField::MonkLevel, "Monk"),
    (EditableField::PaladinLevel, "Paladin"),
    (EditableField::RangerLevel, "Ranger"),
    (EditableField::RogueLevel, "Rogue"),
    (EditableField::SorcererLevel, "Sorcerer"),
    (EditableField::WizardLevel, "Wizard"),
];

/// Shown instead of the cards for a creature that isn't IWD2's.
pub const UNAVAILABLE: &str = "Levels are only available for IWD2 (CRE V2.2) creatures.";

/// Build the Levels & Kits cards for the selected IWD2 creature; `None` for
/// any other.
pub fn build(model: &Model) -> Option<[Vec<FieldCard>; 3]> {
    let cre = model.selected_cre()?;
    let CreHeader::V22(h) = &cre.header else {
        return None;
    };
    let gam = model.gam.as_ref();

    let mut level_rows: Vec<_> = LEVEL_FIELDS
        .iter()
        .map(|(field, label)| editable_row(field.index(), label, &field.read_text(cre, gam)))
        .collect();
    level_rows.push(readonly_row("Total", &h.total_levels.to_string()));

    let mut kit_rows = Vec::new();
    if let Some(kits) = cre.iwd2_kits() {
        for (i, (label, kit)) in Iwd2Kits::ALL.iter().enumerate() {
            kit_rows.push(editable_check_row(
                KIT_ID_OFFSET + i as i32,
                label,
                kits.contains(*kit),
            ));
        }
    }

    Some([
        vec![card("Class Levels", 0, level_rows)],
        vec![card("Kits", 1, kit_rows)],
        Vec::new(),
    ])
}

/// Commit an edited Levels & Kits field: a kit toggle (id ≥ offset) or a
/// class-level write (a shared `EditableField`, via `abilities::commit`).
pub fn commit(model: &mut Model, id: i32, value: &str) {
    if id >= KIT_ID_OFFSET {
        let i = (id - KIT_ID_OFFSET) as usize;
        let Some((_, kit)) = Iwd2Kits::ALL.get(i) else {
            return;
        };
        let on = value == "1";
        if let Some(cre) = model.selected_cre_mut()
            && let Some(mut kits) = cre.iwd2_kits()
        {
            kits.set(*kit, on);
            cre.set_iwd2_kits(kits);
        }
    } else {
        super::abilities::commit(model, id, value);
    }
}
