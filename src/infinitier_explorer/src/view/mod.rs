//! Pushing the explorer's state into the Slint adapter globals, one module
//! per part of the UI in `ui/`: [`shell`] the window chrome, [`dialogs`] the
//! modal dialogs, [`viewers`] the selected resource's viewer. Every push is
//! cheap: images and the BAF rows are built once by the viewers and shared,
//! so a refresh after any change re-sends the whole state.

mod dialogs;
mod shell;
mod viewers;

use slint::{ModelRc, SharedString, VecModel};

use crate::AppWindow;
use crate::app::App;

pub use viewers::refresh as refresh_viewer;

/// Repaint the whole UI from `app`.
pub fn refresh(ui: &AppWindow, app: &App) {
    shell::refresh(ui, app);
    dialogs::refresh(ui, app);
    refresh_viewer(ui, app);
}

/// A Slint list model holding `rows`.
fn rows_model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

/// A Slint list of strings.
fn strings_model(strings: Vec<String>) -> ModelRc<SharedString> {
    rows_model(strings.into_iter().map(SharedString::from).collect())
}
