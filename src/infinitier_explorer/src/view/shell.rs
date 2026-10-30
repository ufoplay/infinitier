//! The window chrome: the title, the header bar, the theme and the status
//! line. (The tree's rows are a live model, see [`crate::tree`].)

use infinitier_gui_common::ThemeSetting;
use slint::ComponentHandle;

use crate::app::App;
use crate::build_data::build_version;
use crate::{AppWindow, ShellAdapter};

pub fn refresh(ui: &AppWindow, app: &App) {
    let shell = ui.global::<ShellAdapter>();
    // The build, then the loaded game's type.
    let app_title = format!("Infinitier Explorer {}", build_version());
    let title = match app.game_type() {
        Some(game) => format!("{app_title} - {game:?}"),
        None => app_title,
    };
    shell.set_title(title.into());
    let game = match &app.game {
        Some(game) => format!(
            "{} — {}",
            game.data.game().short_name(),
            game.config.game_path.display()
        ),
        None => "No game loaded".to_owned(),
    };
    shell.set_game(game.into());
    shell.set_dark_theme(app.settings.theme() == ThemeSetting::Dark);
    shell.set_status(app.status().into());
}
