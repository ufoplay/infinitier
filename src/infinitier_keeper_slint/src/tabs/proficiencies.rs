//! Proficiencies tab for the Slint keeper (editable, AD&D only).
//!
//! Ports the egui keeper's `ui/tabs/proficiencies/`: EEKeeper's
//! three-column table (Proficiency · First Class · Second Class). The
//! proficiency *list* (which `IE_PROFICIENCY*` stats + names) comes from
//! `engine_caps` (WEAPPROF.2DA); each row pairs it with the creature's
//! first/second-class points (`Cre::proficiency`) and commits via
//! `Cre::set_proficiency`. The first class is always editable; the second
//! only for a dual-classed character's weapons, read-only (blank when
//! zero) elsewhere. IWD2 has no block and hides this tab.
//!
//! Cell-id encoding across the Slint boundary: `stat` for the first-class
//! slot, `stat + 256` for the second-class slot.

use infinitier_core::resource::cre::WeaponProficiency;

use crate::components::edit_table::{EditTable, editable_cell, row, value_cell};
use crate::state::Model;

const NAME_HEADER: &str = "Proficiency";
const HEADERS: &[&str] = &["First Class", "Second Class"];

/// EEKeeper's proficiency-point cap.
const MAX_POINTS: u8 = 5;
/// Offset added to a stat to encode the second-class slot id.
const SECOND_SLOT_OFFSET: i32 = 256;

/// Build the Proficiencies table for the selected creature.
pub fn build(model: &Model) -> EditTable {
    let Some(cre) = model.selected_cre() else {
        return EditTable::empty(NAME_HEADER, HEADERS);
    };
    let rows = model
        .engine_caps
        .proficiencies()
        .iter()
        .map(|p| {
            let points = cre.proficiency(p.stat);
            let second = if cre.proficiency_has_second_class(p.stat) {
                editable_cell(
                    i32::from(p.stat) + SECOND_SLOT_OFFSET,
                    &points.second_class.to_string(),
                )
            } else {
                // Matches EEKeeper, which leaves untrained slots empty.
                value_cell(&points_text(points.second_class))
            };
            row(
                &p.name,
                vec![
                    editable_cell(i32::from(p.stat), &points.first_class.to_string()),
                    second,
                ],
            )
        })
        .collect();
    EditTable {
        name_header: NAME_HEADER,
        headers: HEADERS,
        rows,
    }
}

/// A read-only point count: blank for zero.
fn points_text(points: u8) -> String {
    if points == 0 {
        String::new()
    } else {
        points.to_string()
    }
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
