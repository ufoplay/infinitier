//! Effects tab for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/effects/`. Lists the EE (V2) effect
//! records on the selected creature, excluding the proficiency (op233) and
//! local-variable (op187) opcodes that have their own tabs. The opcode →
//! "Type" name table (`opcode_names`) is copied verbatim; resource resrefs
//! are resolved to `SPELL.IDS` symbols, and timing/target ints to EEKeeper's
//! labels — all duplicated from the egui `data.rs`/`view.rs`. Rendered as a
//! wide, horizontally-scrollable grid (effect lists are short, so no
//! virtualisation). Display-only — no `commit`.

mod opcode_names;

use infinitier_core::resource::cre::{Cre, EffectList, EffectV2, SubSections};
use infinitier_core::resource::ids::Ids;
use slint::{ModelRc, SharedString, VecModel};

use crate::GridRow;
use crate::state::Model;
use opcode_names::effect_name;

/// Column headers, in display order (parallel to [`WIDTHS`]).
pub const HEADERS: &[&str] = &[
    "Type",
    "Name",
    "Parameter 1",
    "Parameter 2",
    "Resource 0",
    "Resource 1",
    "Resource 2",
    "Resource 3",
    "Time",
    "Flags",
    "Target",
];

/// Column widths in logical px, matching the egui keeper's layout.
pub const WIDTHS: &[f32] = &[
    250.0, 70.0, 78.0, 78.0, 160.0, 110.0, 110.0, 160.0, 80.0, 150.0, 140.0,
];

// ── Read-only extraction (duplicated from the egui data.rs) ───────────

struct EffectRow {
    opcode: u32,
    name: String,
    param1: u32,
    param2: u32,
    resources: [String; 4],
    time: u32,
    timing_mode: u32,
    target: u32,
}

fn effect_rows(cre: &Cre) -> Vec<EffectRow> {
    let SubSections::V1(sub) = &cre.sub_sections else {
        return Vec::new();
    };
    let EffectList::V2(effects) = &sub.effects else {
        return Vec::new();
    };
    effects
        .iter()
        .filter_map(|e| {
            let EffectV2::Effect(e) = e else { return None };
            Some(EffectRow {
                opcode: e.opcode,
                name: e.variable.clone(),
                param1: e.param1,
                param2: e.param2,
                resources: [
                    e.resource.clone(),
                    e.resource2.clone(),
                    e.resource3.clone(),
                    e.parent_resource.clone(),
                ],
                time: e.duration,
                timing_mode: e.timing_mode,
                target: e.target,
            })
        })
        .collect()
}

// ── Name / label resolution (duplicated from the egui view.rs) ────────

fn type_text(opcode: u32) -> String {
    effect_name(opcode).map_or_else(|| format!("Unknown [{opcode}]"), str::to_owned)
}

fn timing_text(mode: u32) -> String {
    let label = match mode {
        0 => "Duration",
        1 => "Permanent",
        2 => "While Equipped",
        3 => "Delayed Duration",
        4 => "Delayed",
        5 => "Delayed, Equipped",
        6 => "Delayed, Absolute Duration",
        7 => "Delayed, Permanent",
        8 => "Equipped, Permanent",
        9 => "Instant, Permanent",
        4096 => "Absolute Duration",
        other => return other.to_string(),
    };
    label.to_owned()
}

fn target_text(target: u32) -> String {
    let label = match target {
        0 => "None",
        1 => "Self",
        2 => "Preset Target",
        3 => "Party",
        4 => "Everyone",
        5 => "Everyone Except Party",
        6 => "Caster Group",
        7 => "Target Group",
        8 => "Everyone Except Self",
        9 => "Original Caster",
        other => return other.to_string(),
    };
    label.to_owned()
}

/// Render a resource resref the way EEKeeper does: `"Spell Name [RESREF]"`
/// when the resref maps to a `SPELL.IDS` symbol, otherwise the bare resref
/// (empty stays empty).
fn resource_display(spell_ids: Option<&Ids>, resref: &str) -> String {
    if resref.is_empty() {
        return String::new();
    }
    match spell_ids
        .zip(spell_ids_value(resref))
        .and_then(|(ids, value)| ids.of_value(value))
    {
        Some(symbol) => format!("{} [{resref}]", title_case(symbol)),
        None => resref.to_owned(),
    }
}

/// Map a spell resref to its `SPELL.IDS` numeric value (school-code prefix →
/// thousands digit + trailing number). `None` for non-spell resrefs.
fn spell_ids_value(resref: &str) -> Option<i32> {
    let body = resref
        .get(..2)
        .filter(|p| p.eq_ignore_ascii_case("SP"))
        .map(|_| &resref[2..])?;
    let (code, number) = body.split_at_checked(2)?;
    let thousands = match code.to_ascii_uppercase().as_str() {
        "PR" => 1,
        "WI" => 2,
        "IN" => 3,
        "CL" => 4,
        _ => return None,
    };
    let n: i32 = number.parse().ok()?;
    Some(thousands * 1000 + n)
}

fn title_case(symbol: &str) -> String {
    symbol
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_ascii_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Build ────────────────────────────────────────────────────────────

fn grid_row(cells: Vec<SharedString>) -> GridRow {
    GridRow {
        cells: ModelRc::new(VecModel::from(cells)),
    }
}

/// Build the Effects-tab grid rows for the selected creature.
pub fn build(model: &Model) -> Vec<GridRow> {
    let Some(cre) = model.selected_cre() else {
        return Vec::new();
    };
    let rows = effect_rows(cre);
    // `SPELL.IDS` is loaded once per build; effect lists are short.
    let spell_ids = model.game_data.import_ids_by_name("SPELL").ok();
    let spell_ids = spell_ids.as_deref();

    rows.into_iter()
        .map(|r| {
            let mut cells: Vec<SharedString> = Vec::with_capacity(HEADERS.len());
            cells.push(type_text(r.opcode).into());
            cells.push(r.name.as_str().into());
            cells.push(r.param1.to_string().into());
            cells.push(r.param2.to_string().into());
            for resref in &r.resources {
                cells.push(resource_display(spell_ids, resref).into());
            }
            cells.push(r.time.to_string().into());
            cells.push(timing_text(r.timing_mode).into());
            cells.push(target_text(r.target).into());
            grid_row(cells)
        })
        .collect()
}
