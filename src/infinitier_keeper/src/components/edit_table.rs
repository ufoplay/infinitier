//! The Memorization / Proficiencies tables (`ui/tabs/edit_table_tab.slint`):
//! the [`EditTable`] a tab builds, and small constructors for its rows and
//! cells.

use slint::SharedString;

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

    /// The id of the editable cell at `row`, `column` of the table as shown
    /// (column 0 holds the rows' names).
    pub fn cell_id(&self, row: usize, column: usize) -> Option<i32> {
        let cell = self.rows.get(row)?.cells.get(column.checked_sub(1)?)?;
        match cell {
            EditCell::Editable { id, .. } => Some(*id),
            EditCell::Value(_) => None,
        }
    }
}

/// A named row of value cells.
pub struct EditRow {
    pub name: SharedString,
    pub cells: Vec<EditCell>,
}

/// One value of a row.
pub enum EditCell {
    /// A number the user edits, from `minimum` to `maximum`. `id` is the
    /// tab-local cell id its commits carry (dispatched to the active tab by
    /// `callbacks::character`).
    Editable {
        id: i32,
        value: i64,
        minimum: i32,
        maximum: i32,
    },
    /// A read-only text.
    Value(SharedString),
}

/// A row named `name`.
pub fn row(name: &str, cells: Vec<EditCell>) -> EditRow {
    EditRow {
        name: name.into(),
        cells,
    }
}

/// An editable number cell, from `minimum` to `maximum`.
pub fn editable_cell(id: i32, value: i64, minimum: i32, maximum: i32) -> EditCell {
    EditCell::Editable {
        id,
        value,
        minimum,
        maximum,
    }
}

/// A read-only cell.
pub fn value_cell(text: &str) -> EditCell {
    EditCell::Value(text.into())
}
