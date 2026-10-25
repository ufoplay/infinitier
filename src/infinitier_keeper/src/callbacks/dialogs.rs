//! Callbacks of the modal dialogs: Load (`LoadAdapter`), Save
//! (`SaveAdapter`) and Settings (`SettingsAdapter`).

use slint::{ComponentHandle, SharedString};

use super::{AppRc, on_app};
use crate::app::{self, App, Session};
use crate::view;
use crate::{AppWindow, LoadAdapter, SaveAdapter, SettingsAdapter};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    wire_load(ui, app);
    wire_save(ui, app);
    wire_settings(ui, app);
}

/// Load dialog: select / load / cancel / delete (with confirmation).
fn wire_load(ui: &AppWindow, app: &AppRc) {
    let load = ui.global::<LoadAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_select(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                app.load_dialog.select(usize::try_from(row).ok())
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_load(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, open_selected_save);
        });
    }
    {
        // Double-click: select the row and open it.
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_pick(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                app.load_dialog.select(usize::try_from(row).ok());
                open_selected_save(app);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.load_dialog.close());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_delete(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.load_dialog.ask_delete());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_delete_confirm(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session,
                    load_dialog,
                    ..
                } = app;
                if let Some(game_data) = session.game_data_mut() {
                    load_dialog.delete(game_data);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        load.on_delete_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.load_dialog.cancel_delete());
        });
    }
}

/// Open the Load dialog's highlighted save into a tab; on failure the
/// dialog stays open with the error.
fn open_selected_save(app: &mut App) {
    let Some(save) = app.load_dialog.selected_save().cloned() else {
        return;
    };
    match app.open_save(save) {
        Ok(()) => app.load_dialog.close(),
        Err(e) => app.load_dialog.set_error(e),
    }
}

/// Save dialog: confirm / cancel. (The header's Save button opens it.)
fn wire_save(ui: &AppWindow, app: &AppRc) {
    let save = ui.global::<SaveAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        save.on_confirm(move |name| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session,
                    save_dialog,
                    status,
                    ..
                } = app;
                if let Session::Saves(model) = session
                    && let Some(dest) = save_dialog.save(name.as_str(), model)
                {
                    let folder = dest.file_name().unwrap_or(dest.as_os_str());
                    *status = format!("Saved as {}", folder.to_string_lossy());
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        save.on_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.save_dialog.close());
        });
    }
}

/// Settings dialog: the games list and the add form.
fn wire_settings(ui: &AppWindow, app: &AppRc) {
    let settings = ui.global::<SettingsAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        settings.on_select(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                if let Ok(row) = usize::try_from(row) {
                    app.settings.select(row);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        settings.on_remove(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.settings.remove_selected());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        settings.on_add(move |game_path, save_path, game_type| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let game_type = usize::try_from(game_type).unwrap_or(0);
                if app
                    .settings
                    .add_game(game_path.as_str(), save_path.as_str(), game_type)
                {
                    let form = ui.global::<SettingsAdapter>();
                    form.set_form_game_path(SharedString::new());
                    form.set_form_save_path(SharedString::new());
                    form.set_form_game_type(0);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        settings.on_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            close_settings(&ui, &app);
        });
    }
    {
        // Double-click: select the row and close, loading it.
        let (app, weak) = (app.clone(), ui.as_weak());
        settings.on_pick(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            if let Ok(row) = usize::try_from(row) {
                app.borrow_mut().settings.select(row);
            }
            close_settings(&ui, &app);
        });
    }
}

/// Close the Settings dialog, loading the selected game if it isn't the
/// loaded one.
fn close_settings(ui: &AppWindow, app: &AppRc) {
    let switch_to = app.borrow_mut().settings.close();
    match switch_to {
        Some(config) => app::start_loading(app, ui, config),
        None => view::refresh(ui, &mut app.borrow_mut()),
    }
}
