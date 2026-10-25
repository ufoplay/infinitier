//! Small constructors for the Slint `FieldRow` / `FieldCard` structs,
//! shared by the per-tab card builders.

use slint::{ModelRc, SharedString, VecModel};

use crate::{FieldCard, FieldRow};

/// An editable text row. `id` is the tab-local field id routed back on
/// commit (dispatched to the active tab by `callbacks::character`).
pub fn editable_row(id: i32, label: &str, value: &str) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        value: value.into(),
        bonus: SharedString::new(),
        editable: true,
        is_attacks: false,
        is_check: false,
        checked: false,
        numeric: false,
        minimum: 0,
        maximum: 0,
    }
}

/// An editable number row, from `minimum` to `maximum`: a number field
/// (drag or type) whose every change commits `id` with the new value.
pub fn number_row(id: i32, label: &str, value: i64, minimum: i32, maximum: i32) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        value: value.to_string().into(),
        bonus: SharedString::new(),
        editable: true,
        is_attacks: false,
        is_check: false,
        checked: false,
        numeric: true,
        minimum,
        maximum,
    }
}

/// A derived, read-only value row (id `-1`, never committed).
pub fn readonly_row(label: &str, value: &str) -> FieldRow {
    FieldRow {
        id: -1,
        label: label.into(),
        value: value.into(),
        bonus: SharedString::new(),
        editable: false,
        is_attacks: false,
        is_check: false,
        checked: false,
        numeric: false,
        minimum: 0,
        maximum: 0,
    }
}

/// A read-only (disabled) checkbox row.
pub fn check_row(label: &str, checked: bool) -> FieldRow {
    FieldRow {
        id: -1,
        label: label.into(),
        value: SharedString::new(),
        bonus: SharedString::new(),
        editable: false,
        is_attacks: false,
        is_check: true,
        checked,
        numeric: false,
        minimum: 0,
        maximum: 0,
    }
}

/// An editable checkbox row. Toggling commits `commit-field(id, "1"|"0")`.
pub fn editable_check_row(id: i32, label: &str, checked: bool) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        value: SharedString::new(),
        bonus: SharedString::new(),
        editable: false,
        is_attacks: false,
        is_check: true,
        checked,
        numeric: false,
        minimum: 0,
        maximum: 0,
    }
}

pub fn card(title: &str, column: i32, rows: Vec<FieldRow>) -> FieldCard {
    FieldCard {
        title: title.into(),
        column,
        rows: ModelRc::new(VecModel::from(rows)),
    }
}
