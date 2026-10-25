//! The modal dialogs: Settings (`SettingsAdapter`), Load (`LoadAdapter`),
//! Save (`SaveAdapter`) and the loading overlay (`LoadingAdapter`).

use slint::ComponentHandle;

use super::{index_or_none, rows_changed, rows_model};
use crate::app::App;
use crate::{AppWindow, LoadAdapter, LoadingAdapter, SaveAdapter, SettingsAdapter};

pub(super) fn refresh(ui: &AppWindow, app: &App) {
    let settings = &app.settings;
    let adapter = ui.global::<SettingsAdapter>();
    adapter.set_open(settings.is_open());
    if settings.is_open() {
        let rows = settings.rows();
        if rows_changed(&adapter.get_rows(), &rows) {
            adapter.set_rows(rows_model(rows));
        }
        adapter.set_selected(index_or_none(settings.selected()));
        adapter.set_file_error(settings.file_error().unwrap_or_default().into());
        adapter.set_error(settings.error().unwrap_or_default().into());
    }

    let load = &app.load_dialog;
    let adapter = ui.global::<LoadAdapter>();
    adapter.set_open(load.is_open());
    if load.is_open() {
        let rows = load.rows();
        if rows_changed(&adapter.get_rows(), &rows) {
            adapter.set_rows(rows_model(rows));
        }
        adapter.set_selected(index_or_none(load.selected()));
        adapter.set_screenshot(load.screenshot());
        adapter.set_portraits(rows_model(load.portraits()));
        adapter.set_error(load.error().unwrap_or_default().into());
        adapter.set_confirm_delete(load.confirming_delete());
        adapter.set_delete_name(load.selected_name().into());
    }

    let adapter = ui.global::<SaveAdapter>();
    adapter.set_open(app.save_dialog.is_open());
    adapter.set_error(app.save_dialog.error().unwrap_or_default().into());

    let adapter = ui.global::<LoadingAdapter>();
    adapter.set_active(app.pending.is_some());
    if let Some(pending) = &app.pending {
        adapter.set_text(format!("Loading {}…", pending.config.game_path.display()).into());
    }
}
