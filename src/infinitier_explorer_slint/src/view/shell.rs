//! The window chrome: the theme and the status line. (The tree's rows are a
//! live model, see [`crate::tree`].)

use slint::ComponentHandle;

use crate::app::App;
use crate::{AppWindow, ShellAdapter};

pub fn refresh(ui: &AppWindow, app: &App) {
    let shell = ui.global::<ShellAdapter>();
    shell.set_dark_theme(app.dark_theme);
    shell.set_status(app.status().into());
}
