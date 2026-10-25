//! Memorization tab for the Slint keeper (editable, AD&D only).
//!
//! Ports the egui keeper's `ui/tabs/memorization/`: EEKeeper's three-column
//! table (Type · Level · Max Can Memorize), one row per spell-memorisation
//! slot (`spell_memorization_info`). The type and level are fixed; the
//! "Max Can Memorize" count (`num_memorizable_total`) is editable, committed
//! by slot index via `Cre::set_spell_memorization_total`. IWD2 (V2.2)
//! stores per-class spell tables instead and hides this tab.
//!
//! Cell id across the Slint boundary = the slot index.

use infinitier_core::resource::cre::{Cre, SpellType, SubSections};

use crate::components::edit_table::{EditTable, editable_cell, row, value_cell};
use crate::state::Model;

const NAME_HEADER: &str = "Type";
const HEADERS: &[&str] = &["Level", "Max Can Memorize"];

struct MemRow {
    type_name: &'static str,
    level: u16,
    max: u16,
}

fn memorization_rows(cre: &Cre) -> Vec<MemRow> {
    let SubSections::V1(sub) = &cre.sub_sections else {
        return Vec::new();
    };
    sub.spell_memorization_info
        .iter()
        .map(|info| MemRow {
            type_name: spell_type_name(info.spell_type),
            level: info.level.saturating_add(1),
            max: info.num_memorizable_total,
        })
        .collect()
}

fn spell_type_name(spell_type: SpellType) -> &'static str {
    match spell_type {
        SpellType::Priest => "Cleric",
        SpellType::Wizard => "Wizard",
        SpellType::Innate => "Innate",
        SpellType::Unknown(_) => "Unknown",
    }
}

/// Build the Memorization table for the selected creature.
pub fn build(model: &Model) -> EditTable {
    let Some(cre) = model.selected_cre() else {
        return EditTable::empty(NAME_HEADER, HEADERS);
    };
    let rows = memorization_rows(cre)
        .into_iter()
        .enumerate()
        .map(|(slot, r)| {
            row(
                r.type_name,
                vec![
                    value_cell(&r.level.to_string()),
                    editable_cell(
                        i32::try_from(slot).unwrap_or(i32::MAX),
                        r.max.into(),
                        0,
                        u16::MAX.into(),
                    ),
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

/// Commit an edited "Max Can Memorize" cell (parsed to u16) by slot index.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    let Ok(index) = usize::try_from(id) else {
        return;
    };
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    let parsed = value
        .trim()
        .parse::<u32>()
        .unwrap_or(0)
        .min(u16::MAX as u32) as u16;
    cre.set_spell_memorization_total(index, parsed);
}
