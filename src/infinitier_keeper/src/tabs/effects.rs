//! Effects tab.
//!
//! Lists the effects on the selected creature, in either record format
//! ([`fields`]): one row each, ending with Edit, Clone and Delete. Edit (or
//! a double-click), Clone and the Add button under the table open the
//! effect dialog ([`editor`]) on a draft. Delete removes the effect at once, like a
//! deleted item or spell. The opcode → "Type" name table, with what each
//! type's parameters hold and what it does, is `opcodes` (EEKeeper's); resource resrefs are resolved to `SPELL.IDS` symbols,
//! and timing/target ints to EEKeeper's labels ([`labels`]). Rendered as
//! a wide table that scrolls sideways.

pub mod editor;
mod fields;
mod labels;
mod opcodes;

use infinitier_core::resource::cre::{EffectList, TimingMode};
use infinitier_core::resource::ids::Ids;
use slint::SharedString;

use crate::state::Model;
use editor::{EffectEditor, Mode};
use fields::{Draft, EffectRef, Field, listed};
pub use labels::type_text;
use labels::{target_text, timing_text};

/// Column headers, in display order (parallel to [`WIDTHS`]). The Edit,
/// Clone and Delete buttons' follow.
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

/// Column widths in logical px.
pub const WIDTHS: &[f32] = &[
    250.0, 70.0, 78.0, 78.0, 160.0, 110.0, 110.0, 160.0, 80.0, 150.0, 140.0,
];

// ── Resource names ────────────────────────────────────────────────────

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

/// The row of `effect`, under [`HEADERS`].
fn row(effect: EffectRef<'_>, spell_ids: Option<&Ids>) -> Vec<SharedString> {
    let number = |field| effect.number(field).unwrap_or(0);
    let shown = |field| effect.shown(field).unwrap_or(0).to_string();
    let text = |field| effect.text(field).unwrap_or_default();
    let mut cells: Vec<SharedString> = Vec::with_capacity(HEADERS.len());
    cells.push(type_text(number(Field::Opcode)).into());
    cells.push(text(Field::Name).into());
    cells.push(shown(Field::Param1).into());
    cells.push(shown(Field::Param2).into());
    for field in [
        Field::Resource0,
        Field::Resource1,
        Field::Resource2,
        Field::Resource3,
    ] {
        cells.push(resource_display(spell_ids, text(field)).into());
    }
    cells.push(shown(Field::Duration).into());
    cells.push(timing_text(TimingMode::from(number(Field::Timing))).into());
    cells.push(target_text(number(Field::Target)).into());
    cells
}

/// Build the Effects-tab rows for the selected creature.
pub fn build(model: &Model) -> Vec<Vec<SharedString>> {
    let Some(cre) = model.selected_cre() else {
        return Vec::new();
    };
    // `SPELL.IDS` is loaded once per build; effect lists are short.
    let spell_ids = model.game_data.import_ids_by_name("SPELL").ok();
    let spell_ids = spell_ids.as_deref();
    listed(cre.effect_list())
        .into_iter()
        .map(|(_, effect)| row(effect, spell_ids))
        .collect()
}

// ── Editing ───────────────────────────────────────────────────────────

/// The effect of table row `row`: its index in the creature's effect list,
/// and a copy of it.
fn effect_at(model: &Model, row: i32) -> Option<(usize, Draft)> {
    let cre = model.selected_cre()?;
    let row = usize::try_from(row).ok()?;
    listed(cre.effect_list())
        .get(row)
        .map(|&(index, effect)| (index, Draft::of(effect)))
}

/// Edit: open the dialog on a copy of row `row`'s effect.
pub fn edit(model: &mut Model, row: i32) {
    if let Some((index, draft)) = effect_at(model, row) {
        model.effect_editor = Some(EffectEditor::new(Mode::Edit(index), draft));
    }
}

/// Clone: open the dialog on a copy of row `row`'s effect, saved as a new
/// one.
pub fn duplicate(model: &mut Model, row: i32) {
    if let Some((_, draft)) = effect_at(model, row) {
        model.effect_editor = Some(EffectEditor::new(Mode::Clone, draft));
    }
}

/// Add: open the dialog on a new effect, in the creature's format.
pub fn add(model: &mut Model) {
    if let Some(cre) = model.selected_cre() {
        let draft = Draft::new_for(cre.effect_list());
        model.effect_editor = Some(EffectEditor::new(Mode::Add, draft));
    }
}

/// Delete row `row`'s effect.
pub fn delete(model: &mut Model, row: i32) {
    let Some((index, _)) = effect_at(model, row) else {
        return;
    };
    let Some(cre) = model.selected_cre_mut() else {
        return;
    };
    match cre.effect_list_mut() {
        EffectList::V1(list) => {
            list.remove(index);
        }
        EffectList::V2(list) => {
            list.remove(index);
        }
    }
}
