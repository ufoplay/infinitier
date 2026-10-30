//! Callbacks of the window chrome: the header's buttons (`ShellAdapter`)
//! and the open-saves tabs (`SaveTabsAdapter`).

use infinitier_gui_common::ThemeSetting;
use slint::ComponentHandle;

use super::{AppRc, on_app};
use crate::app::{App, Session};
use crate::{AppWindow, SaveAdapter, SaveTabsAdapter, ShellAdapter};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    wire_header(ui, app);
    wire_save_tabs(ui, app);
}

/// Header: open the Load / Save / Settings dialogs, toggle the theme.
fn wire_header(ui: &AppWindow, app: &AppRc) {
    let shell = ui.global::<ShellAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        shell.on_load_clicked(move || {
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
        shell.on_save_clicked(move || {
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
                    ui.global::<SaveAdapter>()
                        .set_folder_name(save_dialog.folder_name().into());
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        shell.on_settings_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| app.settings.open());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        shell.on_theme_toggle(move || {
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

/// Open-saves tab strip: switch / close.
fn wire_save_tabs(ui: &AppWindow, app: &AppRc) {
    let tabs = ui.global::<SaveTabsAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tabs.on_select(move |i| {
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
        tabs.on_close(move |i| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                if let Ok(i) = usize::try_from(i) {
                    app.close_save(i);
                }
            });
        });
    }
}
