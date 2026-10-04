//! Proficiencies tab for the Slint keeper (editable, AD&D only).
//!
//! Ports the egui keeper's `ui/tabs/proficiencies/`. The proficiency *list*
//! (which `IE_PROFICIENCY*` stats + names) comes from `engine_caps`
//! (WEAPPROF.2DA); each row pairs it with the creature's first/second-class
//! points (`Cre::proficiency`) and commits via `Cre::set_proficiency`.
//! Rendered as editable cards: one row per proficiency's first-class points,
//! plus a "(2nd)" row for dual-classed characters. IWD2 has no block and
//! hides this tab.
//!
//! Field-id encoding across the Slint boundary: `stat` for the first-class
//! slot, `stat + 256` for the second-class slot.

use infinitier_core::resource::cre::WeaponProficiency;

use crate::FieldCard;
use crate::state::Model;
use crate::ui_model::{card, editable_row};

/// EEKeeper's proficiency-point cap.
const MAX_POINTS: u8 = 5;
/// Offset added to a stat to encode the second-class slot id.
const SECOND_SLOT_OFFSET: i32 = 256;

/// Build the Proficiencies-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(cre) = model.selected_cre() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let mut rows = Vec::new();
    for p in model.engine_caps.proficiencies() {
        let points = cre.proficiency(p.stat);
        rows.push(editable_row(
            i32::from(p.stat),
            &p.name,
            &points.first_class.to_string(),
        ));
        if cre.proficiency_has_second_class(p.stat) {
            rows.push(editable_row(
                i32::from(p.stat) + SECOND_SLOT_OFFSET,
                &format!("{} (2nd)", p.name),
                &points.second_class.to_string(),
            ));
        }
    }
    [
        vec![card("Weapon Proficiencies", 0, rows)],
        Vec::new(),
        Vec::new(),
    ]
}

/// Commit an edited proficiency-point cell. Parses + clamps to 0..=5,
/// rewrites the whole packed byte for the stat preserving the other slot.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    let (stat, is_second) = if id >= SECOND_SLOT_OFFSET {
        ((id - SECOND_SLOT_OFFSET) as u8, true)
    } else if id >= 0 {
        (id as u8, false)
    } else {
        return;
    };
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    let Ok(parsed) = value.trim().parse::<u32>() else {
        return;
    };
    let clamped = parsed.min(u32::from(MAX_POINTS)) as u8;
    let current = cre.proficiency(stat);
    let points = if is_second {
        WeaponProficiency {
            first_class: current.first_class,
            second_class: clamped,
        }
    } else {
        WeaponProficiency {
            first_class: clamped,
            second_class: current.second_class,
        }
    };
    cre.set_proficiency(stat, points);
}
