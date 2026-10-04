//! Small constructors for the Slint `FieldRow` / `FieldCard` structs,
//! shared by the per-tab card builders.

use slint::{ModelRc, SharedString, VecModel};

use crate::{FieldCard, FieldRow};

/// An editable text row. `id` is the tab-local field id routed back on
/// commit (dispatched by the active tab in `main::refresh`).
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
    }
}

pub fn card(title: &str, column: i32, rows: Vec<FieldRow>) -> FieldCard {
    FieldCard {
        title: title.into(),
        column,
        rows: ModelRc::new(VecModel::from(rows)),
    }
}
