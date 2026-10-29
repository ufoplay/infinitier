//! Pushing the keeper's state into the Slint pages' adapter globals, one
//! module per part of the UI in `ui/`: [`shell`] the window chrome and the
//! party, [`dialogs`] the modal dialogs, [`character`] the selected
//! character's tab, [`browsers`] the browser windows.

mod browsers;
mod character;
mod dialogs;
mod shell;

use slint::{Model as _, ModelRc, VecModel};

use crate::app::{App, Session};
use crate::{AppWindow, TableCell};

pub use browsers::refresh_browsers;

/// Repaint the whole UI from `app`.
pub fn refresh(ui: &AppWindow, app: &mut App) {
    shell::refresh(ui, app);
    dialogs::refresh(ui, app);
    match &mut app.session {
        Session::Saves(model) => character::refresh(ui, model),
        Session::NoGame | Session::Game(_) => character::clear(ui),
    }
    refresh_browsers(ui, app);
}

/// A Slint list model holding `rows`.
fn rows_model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

/// Whether a list property's rows differ from `rows`. Lists are only
/// replaced when they change: a new model rebuilds the row elements, which
/// drops a click in progress — the second click of a double-click lands on
/// a fresh row and never registers as one.
fn rows_changed<T: Clone + PartialEq + 'static>(current: &ModelRc<T>, rows: &[T]) -> bool {
    current.row_count() != rows.len() || current.iter().zip(rows).any(|(old, new)| old != *new)
}

/// Show `rows` in a list property whose model is `current`, keeping its row
/// elements where it can: with as many rows as before, the current model is
/// updated row by row, so a row's input being typed in or dragged survives
/// the refresh its own commit triggers (a new model would rebuild every row
/// mid-edit). Returns the new model to set when the row count changed.
#[must_use]
fn sync_rows<T: Clone + PartialEq + 'static>(
    current: &ModelRc<T>,
    rows: Vec<T>,
) -> Option<ModelRc<T>> {
    let model = current
        .as_any()
        .downcast_ref::<VecModel<T>>()
        .filter(|model| model.row_count() == rows.len());
    let Some(model) = model else {
        return Some(rows_model(rows));
    };
    for (i, row) in rows.into_iter().enumerate() {
        if model.row_data(i).as_ref() != Some(&row) {
            model.set_row_data(i, row);
        }
    }
    None
}

/// A row holding a list of its own (a card's fields), for [`sync_nested`].
trait NestedRows: Clone + 'static {
    type Item: Clone + PartialEq + 'static;
    /// Whether `other` is the same row with other contents: then its nested
    /// list is synced into this one's.
    fn same_row(&self, other: &Self) -> bool;
    fn nested(&self) -> &ModelRc<Self::Item>;
}

impl NestedRows for crate::FieldCard {
    type Item = crate::FieldRow;
    fn same_row(&self, other: &Self) -> bool {
        self.title == other.title && self.column == other.column
    }
    fn nested(&self) -> &ModelRc<crate::FieldRow> {
        &self.rows
    }
}

/// [`sync_rows`] for rows holding lists: the same row keeps its nested
/// model too, synced in place, so the inputs inside it survive as well.
#[must_use]
fn sync_nested<T: NestedRows>(current: &ModelRc<T>, rows: Vec<T>) -> Option<ModelRc<T>> {
    let model = current
        .as_any()
        .downcast_ref::<VecModel<T>>()
        .filter(|model| model.row_count() == rows.len());
    let Some(model) = model else {
        return Some(rows_model(rows));
    };
    for (i, row) in rows.into_iter().enumerate() {
        let kept = model
            .row_data(i)
            .filter(|old| old.same_row(&row))
            .is_some_and(|old| sync_rows(old.nested(), row.nested().iter().collect()).is_none());
        if !kept {
            model.set_row_data(i, row);
        }
    }
    None
}

/// A Slint row index for `index`, `-1` for none.
fn index_or_none(index: Option<usize>) -> i32 {
    index.and_then(|i| i32::try_from(i).ok()).unwrap_or(-1)
}

/// A themekit `Table`'s rows: each row's cells, one per column.
type TableRows = ModelRc<ModelRc<TableCell>>;

/// Show `rows` in a `Table` whose rows model is `current`: when it has as
/// many rows, each row's cells are synced in place (see [`sync_rows`]), so a
/// number being edited survives and a double-click registers.
#[must_use]
fn sync_table(current: &TableRows, rows: Vec<Vec<TableCell>>) -> Option<TableRows> {
    let model = current
        .as_any()
        .downcast_ref::<VecModel<ModelRc<TableCell>>>()
        .filter(|model| model.row_count() == rows.len());
    let Some(model) = model else {
        return Some(rows_model(rows.into_iter().map(rows_model).collect()));
    };
    for (i, cells) in rows.into_iter().enumerate() {
        if let Some(cells) = model.row_data(i).and_then(|old| sync_rows(&old, cells)) {
            model.set_row_data(i, cells);
        }
    }
    None
}
