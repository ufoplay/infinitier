//! The header bar: the theme toggle and Settings.

use infinitier_gui_common::ThemeSetting;
use slint::ComponentHandle;

use super::Ctx;
use crate::{AppWindow, ShellAdapter};

pub(super) fn wire(ui: &AppWindow, ctx: &Ctx) {
    let shell = ui.global::<ShellAdapter>();
    {
        let ctx = ctx.clone();
        shell.on_theme_toggle(move || {
            ctx.update(|app| {
                let next = match app.settings.theme() {
                    ThemeSetting::Light => ThemeSetting::Dark,
                    ThemeSetting::Dark => ThemeSetting::Light,
                };
                app.settings.set_theme(next);
            })
        });
    }
    {
        let ctx = ctx.clone();
        shell.on_settings_clicked(move || ctx.update(|app| app.settings.open()));
    }
}
