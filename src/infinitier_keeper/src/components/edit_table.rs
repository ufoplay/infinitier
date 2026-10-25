//! The Memorization / Proficiencies tables (`ui/tabs/edit_table_tab.slint`):
//! the [`EditTable`] a tab builds, and small constructors for its Slint
//! `EditRow` / `EditCell` structs.

use slint::{ModelRc, VecModel};

use crate::{EditCell, EditRow};

/// A table page: the first column's header, the value columns' headers and
/// the rows (one cell per value column).
pub struct EditTable {
    pub name_header: &'static str,
    pub headers: &'static [&'static str],
    pub rows: Vec<EditRow>,
}

impl EditTable {
    /// No character: the headers over no rows.
    pub fn empty(name_header: &'static str, headers: &'static [&'static str]) -> Self {
        Self {
            name_header,
            headers,
            rows: Vec::new(),
        }
    }
}

/// A row named `name`.
pub fn row(name: &str, cells: Vec<EditCell>) -> EditRow {
    EditRow {
        name: name.into(),
        cells: ModelRc::new(VecModel::from(cells)),
    }
}

/// An editable number cell, from `minimum` to `maximum`. `id` is the
/// tab-local cell id routed back with each change (dispatched to the active
/// tab by `callbacks::character`).
pub fn editable_cell(id: i32, value: i64, minimum: i32, maximum: i32) -> EditCell {
    EditCell {
        text: value.to_string().into(),
        editable: true,
        id,
        minimum,
        maximum,
    }
}

/// A read-only cell (id `-1`, never committed).
pub fn value_cell(text: &str) -> EditCell {
    EditCell {
        text: text.into(),
        editable: false,
        id: -1,
        minimum: 0,
        maximum: 0,
    }
}
