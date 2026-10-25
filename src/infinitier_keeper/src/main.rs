//! The Infinitier Keeper: a save-game editor for Infinity Engine games.
//!
//! The data layer comes from `infinitier_core` (game-data indexing, save
//! discovery, GAM/CRE parse + export, engine caps); this crate is the Slint
//! UI on top, built on slint-themekit, with the editable-field rules in
//! [`components`]. The modules mirror the Slint UI in `ui/`: [`tabs`] and [`dialogs`] hold each page's state and
//! logic, [`view`] pushes the state into the pages' adapter globals and
//! [`callbacks`] handles their callbacks.
//!
//! The keeper starts with no game loaded: it loads the settings file's
//! active game, and the Settings dialog adds games and picks another.

mod app;
mod browser;
mod build_data;
mod callbacks;
mod components;
mod dialogs;
mod icons;
mod settings;
mod state;
mod tabs;
mod view;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::app::App;
use crate::components::editable_fields::AttacksOption;
use crate::dialogs::settings_dialog::{self, SettingsDialog};
use crate::settings::SETTINGS_FILE;

slint::include_modules!();

/// Infinitier Keeper — cross-engine Infinity Engine save-game editor.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Settings file holding the games listed in the Settings dialog.
    /// Created on the first change if it doesn't exist.
    #[arg(long, default_value = SETTINGS_FILE)]
    config: PathBuf,
    /// Log filter, e.g. "warn", "infinitier=debug,warn".
    #[arg(long, default_value = "infinitier=debug,warn")]
    log: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::Builder::new().parse_filters(&args.log).init();
    log::info!("Infinitier Keeper {}", build_data::build_version());

    let settings = SettingsDialog::load(args.config);
    let startup_game = settings.startup_game();
    let app = Rc::new(RefCell::new(App::new(settings)));

    let ui = AppWindow::new()?;
    // Static dropdown options.
    let attacks_labels: Vec<SharedString> =
        AttacksOption::ALL.iter().map(|o| o.label.into()).collect();
    ui.global::<CardsAdapter>()
        .set_attacks_options(ModelRc::new(VecModel::from(attacks_labels)));
    let game_types: Vec<SharedString> = settings_dialog::game_type_options()
        .into_iter()
        .map(Into::into)
        .collect();
    ui.global::<SettingsAdapter>()
        .set_game_types(ModelRc::new(VecModel::from(game_types)));

    callbacks::wire(&ui, &app);
    match startup_game {
        Some(config) => app::start_loading(&app, &ui, config),
        None => view::refresh(&ui, &mut app.borrow_mut()),
    }

    ui.run()?;
    Ok(())
}
