use slint::ComponentHandle;

use super::Ctx;
use crate::{AppWindow, ShellAdapter};

pub(super) fn wire(ui: &AppWindow, ctx: &Ctx) {
    let ctx = ctx.clone();
    ui.global::<ShellAdapter>()
        .on_theme_toggle(move || ctx.update(|app| app.dark_theme = !app.dark_theme));
}
