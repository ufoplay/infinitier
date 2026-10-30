//! The Infinitier Explorer: browse the resources of Infinity Engine games.
//!
//! The data layer comes from `infinitier_core` (game-data indexing, every
//! resource importer, the sound and movie decoders); this crate is the Slint
//! UI on top, built on slint-themekit. The modules mirror the Slint UI in
//! `ui/`: [`viewers`] holds each viewer's state and logic,
//! [`view`] pushes the state into the adapter globals and [`callbacks`]
//! handles their callbacks.
//!
//! The explorer starts with no game loaded: it loads the settings file's
//! active game, and the Settings dialog adds games and picks another (the
//! settings and the dialog's state are shared with the keeper, in
//! `infinitier_gui_common`).

mod app;
mod build_data;
mod callbacks;
mod fonts;
mod loading;
mod ticker;
mod tree;
mod view;
mod viewers;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use infinitier_gui_common::{SettingsDialog, game_type_options};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::app::App;
use crate::build_data::build_version;
use crate::ticker::Ticker;

slint::include_modules!();

/// Default settings file, resolved against the current folder.
const SETTINGS_FILE: &str = "infinitier_explorer.toml";

/// Infinitier Explorer — browse resources from Infinity Engine games.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Settings file holding the games listed in the Settings dialog.
    /// Created on the first change if it doesn't exist.
    #[arg(long, default_value = SETTINGS_FILE)]
    config: PathBuf,
    /// Log filter, e.g. "warn", "debug", "infinitier=debug,warn".
    #[arg(long, default_value = "infinitier=debug,warn")]
    log: String,
}

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse();
    env_logger::Builder::new().parse_filters(&args.log).init();

    log::info!("Infinitier Explorer {}", build_version());

    let settings = SettingsDialog::load(args.config);
    let startup_game = settings.startup_game();
    let app = Rc::new(RefCell::new(App::new(settings)));

    let ui = AppWindow::new()?;
    match fonts::monospace_family() {
        Some(family) => ui.global::<Fonts>().set_mono(family.into()),
        None => log::warn!("no monospace font found: code is shown in the default font"),
    }
    let game_types: Vec<SharedString> = game_type_options().into_iter().map(Into::into).collect();
    ui.global::<SettingsAdapter>()
        .set_game_types(ModelRc::new(VecModel::from(game_types)));

    ui.global::<TreeAdapter>()
        .set_rows(ModelRc::from(app.borrow().tree.clone()));
    let ticker = Rc::new(Ticker::default());
    callbacks::wire(&ui, &app, &ticker);
    match startup_game {
        Some(config) => loading::start(&ui, &app, &ticker, config),
        None => view::refresh(&ui, &app.borrow()),
    }

    ui.run()
}
