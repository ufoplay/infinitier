//! Memorization tab for the Slint keeper (editable, AD&D only).
//!
//! Ports the egui keeper's `ui/tabs/memorization/`. One row per spell-
//! memorisation slot (`spell_memorization_info`): a type + level label and
//! the editable "Max Can Memorise" count (`num_memorizable_total`), committed
//! by slot index via `Cre::set_spell_memorization_total`. IWD2 (V2.2) stores
//! per-class spell tables instead and hides this tab.
//!
//! Field-id across the Slint boundary = the slot index.

use infinitier_core::resource::cre::{Cre, SpellType, SubSections};

use crate::FieldCard;
use crate::state::Model;
use crate::ui_model::{card, editable_row};

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

/// Build the Memorization-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(cre) = model.selected_cre() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let rows = memorization_rows(cre)
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            editable_row(
                i as i32,
                &format!("{} L{}", r.type_name, r.level),
                &r.max.to_string(),
            )
        })
        .collect();
    [
        vec![card("Spell Memorization (Max Can Memorise)", 0, rows)],
        Vec::new(),
        Vec::new(),
    ]
}

/// Commit an edited "Max Can Memorise" cell (parsed to u16) by slot index.
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
