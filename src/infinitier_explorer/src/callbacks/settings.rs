//! The Settings dialog: the games list and the add form. Closing it with
//! another game selected loads that game.

use std::path::Path;

use rfd::AsyncFileDialog;
use slint::{ComponentHandle, SharedString};

use super::Ctx;
use crate::{AppWindow, FormFolder, SettingsAdapter, loading};

pub(super) fn wire(ui: &AppWindow, ctx: &Ctx) {
    let settings = ui.global::<SettingsAdapter>();
    {
        let ctx = ctx.clone();
        settings.on_select(move |row| {
            ctx.update(|app| {
                if let Ok(row) = usize::try_from(row) {
                    app.settings.select(row);
                }
            })
        });
    }
    {
        let ctx = ctx.clone();
        settings.on_remove(move || ctx.update(|app| app.settings.remove_selected()));
    }
    {
        let ctx = ctx.clone();
        settings.on_add(move |game_path, save_path, game_type| {
            let game_type = usize::try_from(game_type).unwrap_or(0);
            let mut added = false;
            ctx.update(|app| {
                added = app
                    .settings
                    .add_game(game_path.as_str(), save_path.as_str(), game_type);
            });
            if added && let Some(ui) = ctx.ui.upgrade() {
                let form = ui.global::<SettingsAdapter>();
                form.set_form_game_path(SharedString::new());
                form.set_form_save_path(SharedString::new());
                form.set_form_game_type(0);
            }
        });
    }
    {
        let weak = ui.as_weak();
        settings.on_browse(move |folder| {
            let Some(ui) = weak.upgrade() else { return };
            browse_folder(&ui, folder);
        });
    }
    {
        let ctx = ctx.clone();
        settings.on_close(move || close(&ctx));
    }
    {
        // Double-click: select the row and close, loading it.
        let ctx = ctx.clone();
        settings.on_pick(move |row| {
            if let Ok(row) = usize::try_from(row) {
                ctx.app.borrow_mut().settings.select(row);
            }
            close(&ctx);
        });
    }
}

/// Close the dialog, loading the selected game if it isn't the loaded one.
fn close(ctx: &Ctx) {
    let mut switch_to = None;
    ctx.update(|app| switch_to = app.settings.close());
    if let (Some(config), Some(ui)) = (switch_to, ctx.ui.upgrade()) {
        loading::start(&ui, &ctx.app, &ctx.ticker, config);
    }
}

/// Open the OS's folder picker for one of the add form's folders, starting
/// at the field's folder, and put the chosen folder into the field. The
/// picker is awaited on the event loop, so the window keeps repainting; the
/// Browse buttons wait for it.
fn browse_folder(ui: &AppWindow, folder: FormFolder) {
    let form = ui.global::<SettingsAdapter>();
    let (title, current) = match folder {
        FormFolder::Game => (
            "Select the game folder (containing CHITIN.KEY)",
            form.get_form_game_path(),
        ),
        FormFolder::Save => (
            "Select the savegame folder (containing save/), or save/ itself",
            form.get_form_save_path(),
        ),
    };
    let dialog = AsyncFileDialog::new()
        .set_title(title)
        .set_parent(&ui.window().window_handle());
    let start = Path::new(current.trim());
    let dialog = if start.is_dir() {
        dialog.set_directory(start)
    } else {
        dialog
    };
    let picked = dialog.pick_folder();
    form.set_browsing(true);
    let weak = ui.as_weak();
    let spawned = slint::spawn_local(async move {
        let picked = picked.await;
        let Some(ui) = weak.upgrade() else { return };
        let form = ui.global::<SettingsAdapter>();
        form.set_browsing(false);
        let Some(picked) = picked else { return };
        let path = SharedString::from(picked.path().to_string_lossy().as_ref());
        match folder {
            FormFolder::Game => form.set_form_game_path(path),
            FormFolder::Save => form.set_form_save_path(path),
        }
    });
    if let Err(e) = spawned {
        log::error!("[settings] can't open the folder picker: {e}");
        form.set_browsing(false);
    }
}
