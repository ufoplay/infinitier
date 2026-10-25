//! Pushing the keeper's state into the Slint pages' adapter globals, one
//! module per part of the UI in `ui/`: [`shell`] the window chrome and the
//! party, [`dialogs`] the modal dialogs, [`character`] the selected
//! character's tab, [`browsers`] the browser windows.

mod browsers;
mod character;
mod dialogs;
mod shell;

use slint::{Model as _, ModelRc, VecModel};

use crate::AppWindow;
use crate::app::{App, Session};

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

/// A Slint row index for `index`, `-1` for none.
fn index_or_none(index: Option<usize>) -> i32 {
    index.and_then(|i| i32::try_from(i).ok()).unwrap_or(-1)
}
