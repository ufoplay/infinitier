//! Callbacks of the shell: the open-saves tab strip, the Load / Save /
//! Settings dialogs and the theme toggle.

use slint::{ComponentHandle, SharedString};

use super::{AppRc, on_app};
use crate::AppWindow;
use crate::app::{self, App, Session};
use crate::settings::ThemeSetting;
use crate::view;

pub fn wire(ui: &AppWindow, app: &AppRc) {
    wire_save_tabs(ui, app);
    wire_load_dialog(ui, app);
    wire_save_dialog(ui, app);
    wire_settings(ui, app);
}

/// Open-saves tab strip: switch / close.
fn wire_save_tabs(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_select_save_tab(move |i| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                if let (Some(model), Ok(i)) = (app.model_mut(), usize::try_from(i)) {
                    model.switch_save(i);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_close_save_tab(move |i| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                if let Ok(i) = usize::try_from(i) {
                    app.close_save(i);
                }
            });
        });
    }
}

/// Load dialog: open / select / load / cancel / delete (with confirmation).
fn wire_load_dialog(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session,
                    load_dialog,
                    ..
                } = app;
                if let Some(game_data) = session.game_data_mut() {
                    load_dialog.open(game_data);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_select(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                app.load_dialog.select(usize::try_from(row).ok())
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_open_save(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, open_selected_save);
        });
    }
    {
        // Double-click: select the row and open it.
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_pick(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                app.load_dialog.select(usize::try_from(row).ok());
                open_selected_save(app);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.load_dialog.close());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_delete(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.load_dialog.ask_delete());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_load_delete_confirm(move || {
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
        ui.on_load_delete_cancel(move || {
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

/// Save dialog: open (pre-filled name) / confirm / cancel.
fn wire_save_dialog(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_save_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session,
                    save_dialog,
                    ..
                } = app;
                if let Session::Saves(model) = session {
                    save_dialog.open(model);
                    // Set once on open, so later repaints don't overwrite
                    // what the user types.
                    ui.set_save_folder_name(save_dialog.folder_name().into());
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_save_confirm(move |name| {
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
        ui.on_save_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.save_dialog.close());
        });
    }
}

/// Settings dialog (games list + add form) and the theme toggle.
fn wire_settings(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_settings_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.settings.open());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_settings_select(move |row| {
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
        ui.on_settings_remove(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.settings.remove_selected());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_settings_add(move |game_path, save_path, game_type| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let game_type = usize::try_from(game_type).unwrap_or(0);
                if app
                    .settings
                    .add_game(game_path.as_str(), save_path.as_str(), game_type)
                {
                    ui.set_settings_form_game_path(SharedString::new());
                    ui.set_settings_form_save_path(SharedString::new());
                    ui.set_settings_form_game_type(0);
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_settings_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            close_settings(&ui, &app);
        });
    }
    {
        // Double-click: select the row and close, loading it.
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_settings_pick(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            if let Ok(row) = usize::try_from(row) {
                app.borrow_mut().settings.select(row);
            }
            close_settings(&ui, &app);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_theme_toggle(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let next = match app.settings.theme() {
                    ThemeSetting::Light => ThemeSetting::Dark,
                    ThemeSetting::Dark => ThemeSetting::Light,
                };
                app.settings.set_theme(next);
            });
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
