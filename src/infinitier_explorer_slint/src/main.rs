//! Slint edition of the Infinitier Explorer: browse the resources of
//! Infinity Engine games.
//!
//! Reuses the data layer from `infinitier_core` (game-data indexing, every
//! resource importer, the sound and movie decoders) and reimplements the
//! *view* in Slint, on slint-themekit, instead of egui. The egui explorer's
//! pure logic is duplicated here rather than shared (the tree grouping, the
//! TIS column limits, the BAF tokenizer, the streaming players), per the
//! port decision to keep the two explorers independent. The modules mirror
//! the Slint UI in `ui/`: [`viewers`] holds each viewer's state and logic,
//! [`view`] pushes the state into the adapter globals and [`callbacks`]
//! handles their callbacks.

mod app;
mod build_data;
mod callbacks;
mod fonts;
mod ticker;
mod tree;
mod view;
mod viewers;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use infinitier_core::game::GameDataBuilder;
use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::build_data::build_version;
use crate::ticker::Ticker;

slint::include_modules!();

/// Infinitier Explorer (Slint) — browse resources from Infinity Engine games.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Comma-separated list of game folders (bg, bg2, bgee, bg2ee, idw, idwee,
    /// idw2, pst, pstee). At least the first folder must contain a CHITIN.KEY
    /// file. When multiple folders are given they are merged into a single
    /// case-insensitive view in input order — later folders override earlier
    /// ones on path conflicts (mod-overlay style).
    #[arg(value_delimiter = ',', required = true, num_args = 1..)]
    game_path: Vec<PathBuf>,
    /// Log filter, e.g. "warn", "debug", "infinitier=debug,warn".
    #[arg(long, default_value = "infinitier=debug,warn")]
    log: String,
}

/// `paths` as a comma-separated string, for messages.
fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse();
    env_logger::Builder::new().parse_filters(&args.log).init();

    log::info!("Infinitier Explorer {}", build_version());

    let game_data = match GameDataBuilder::new(args.game_path.as_slice(), None)
        .and_then(|builder| builder.build())
    {
        Ok(game_data) => game_data,
        Err(e) => {
            eprintln!(
                "Failed to load key file from [{}]: {e}",
                display_paths(&args.game_path),
            );
            std::process::exit(1);
        }
    };
    let title = format!(
        "Infinitier Explorer {} - {:?}",
        build_version(),
        game_data.game()
    );

    let ui = AppWindow::new()?;
    ui.global::<ShellAdapter>().set_title(title.into());
    match fonts::monospace_family() {
        Some(family) => ui.global::<Fonts>().set_mono(family.into()),
        None => log::warn!("no monospace font found: code is shown in the default font"),
    }

    let app = Rc::new(RefCell::new(App::new(game_data)));
    ui.global::<TreeAdapter>()
        .set_rows(ModelRc::from(app.borrow().tree.clone()));
    let ticker = Rc::new(Ticker::default());
    callbacks::wire(&ui, &app, &ticker);
    view::refresh(&ui, &app.borrow());

    ui.run()
}
