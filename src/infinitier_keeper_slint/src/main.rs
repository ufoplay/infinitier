//! Slint edition of the Infinitier Keeper.
//!
//! Reuses the whole data layer from `infinitier_core` (game-data indexing,
//! save discovery, GAM/CRE parse + export, engine caps) and reimplements the
//! *view* in vanilla Slint instead of egui. The editable-field logic is
//! duplicated from the egui keeper (see [`fields`] / [`cre_fields`]) per the
//! port decision to keep the two keepers independent.
//!
//! The keeper starts with no game loaded: it loads the settings file's
//! active game, and the Settings dialog adds games and picks another.

mod abilities;
mod app;
mod appearance;
mod browser;
mod callbacks;
mod characteristics;
mod cre_fields;
mod effects;
mod feats;
mod fields;
mod inventory;
mod journal;
mod levels;
mod load_dialog;
mod memorization;
mod miscellaneous;
mod proficiencies;
mod resistances;
mod save_dialog;
mod settings;
mod settings_dialog;
mod spells;
mod state;
mod ui_model;
mod variables;
mod view;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use slint::{ModelRc, SharedString, VecModel};

use crate::app::App;
use crate::fields::AttacksOption;
use crate::settings::SETTINGS_FILE;
use crate::settings_dialog::SettingsDialog;

slint::include_modules!();

/// Infinitier Keeper (Slint) — cross-engine Infinity Engine save-game editor.
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

    let settings = SettingsDialog::load(args.config);
    let startup_game = settings.startup_game();
    let app = Rc::new(RefCell::new(App::new(settings)));

    let ui = AppWindow::new()?;
    // Static dropdown options.
    let attacks_labels: Vec<SharedString> =
        AttacksOption::ALL.iter().map(|o| o.label.into()).collect();
    ui.set_attacks_options(ModelRc::new(VecModel::from(attacks_labels)));
    let game_types: Vec<SharedString> = settings_dialog::game_type_options()
        .into_iter()
        .map(Into::into)
        .collect();
    ui.set_settings_game_types(ModelRc::new(VecModel::from(game_types)));

    callbacks::wire(&ui, &app);
    match startup_game {
        Some(config) => app::start_loading(&app, &ui, config),
        None => view::refresh(&ui, &mut app.borrow_mut()),
    }

    ui.run()?;
    Ok(())
}
