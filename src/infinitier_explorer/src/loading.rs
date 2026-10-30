//! Loading a game: its data is read on a background thread while a modal
//! "Loading…" overlay with a spinner blocks the UI. Then it replaces the
//! loaded game (all of it: the tree, the selection, the viewer), or the
//! Settings dialog reopens with the error and the loaded game stays.

use std::any::Any;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use infinitier_core::game::{GameData, GameDataBuilder};
use infinitier_core::resource::Game;
use infinitier_gui_common::GameConfig;
use slint::{ComponentHandle, TimerMode};

use crate::AppWindow;
use crate::app::{App, LoadedGame};
use crate::callbacks::AppRc;
use crate::ticker::Ticker;
use crate::view;

/// How often a running game load is checked for completion.
const LOAD_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// A game being read on a background thread.
pub struct PendingGame {
    pub config: GameConfig,
    loading: JoinHandle<Result<GameData, String>>,
}

/// Start reading `config`'s game on a background thread. The loading
/// overlay stays up until the thread finishes.
pub fn start(ui: &AppWindow, app_rc: &AppRc, ticker: &Rc<Ticker>, config: GameConfig) {
    let mut app = app_rc.borrow_mut();
    let roots = config.roots();
    let game_type = config.game_type;
    let spawned = thread::Builder::new()
        .name("game-load".into())
        .spawn(move || load(roots, game_type));
    match spawned {
        Ok(loading) => {
            app.pending = Some(PendingGame { config, loading });
            let (app_rc, ticker, weak) = (app_rc.clone(), ticker.clone(), ui.as_weak());
            app.loading_timer
                .start(TimerMode::Repeated, LOAD_POLL_INTERVAL, move || {
                    let Some(ui) = weak.upgrade() else { return };
                    let loaded = {
                        let mut app = app_rc.borrow_mut();
                        let Some(pending) = app.pending.take_if(|p| p.loading.is_finished()) else {
                            return;
                        };
                        app.loading_timer.stop();
                        let loaded = finish(&mut app, pending);
                        view::refresh(&ui, &app);
                        loaded
                    };
                    // The old game's viewer, if playing, is gone.
                    ticker.sync(&ui, &app_rc);
                    if loaded {
                        ui.invoke_tree_scroll_to(0);
                    }
                });
        }
        Err(e) => load_failed(&mut app, &config, &e),
    }
    view::refresh(ui, &app);
}

/// Index the game at `roots`.
fn load(roots: Vec<PathBuf>, game_type: Option<Game>) -> Result<GameData, String> {
    GameDataBuilder::new(roots.as_slice(), game_type)
        .and_then(|builder| builder.build())
        .map_err(|e| format!("couldn't load the game data: {e}"))
}

/// Swap in the game a finished background load produced, or report why it
/// couldn't be loaded. Returns whether the game was swapped in.
fn finish(app: &mut App, pending: PendingGame) -> bool {
    let PendingGame { config, loading } = pending;
    let game_data = match loading.join() {
        Ok(Ok(game_data)) => game_data,
        Ok(Err(e)) => {
            load_failed(app, &config, &e);
            return false;
        }
        Err(panic) => {
            load_failed(app, &config, &panic_message(panic.as_ref()));
            return false;
        }
    };
    log::info!(
        "[settings] switched to {:?} at {}",
        game_data.game(),
        config.game_path.display()
    );
    app.settings.game_loaded(config.clone());
    app.set_game(LoadedGame {
        config,
        data: game_data,
    });
    true
}

fn load_failed(app: &mut App, config: &GameConfig, err: &dyn Display) {
    log::error!(
        "[settings] failed to load {}: {err}",
        config.game_path.display()
    );
    app.settings.game_load_failed(config, err);
}

/// What a panicked loader thread reported, for the error shown in Settings.
fn panic_message(payload: &(dyn Any + Send)) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned());
    match message {
        Some(message) => format!("the loader crashed: {message}"),
        None => "the loader crashed".to_owned(),
    }
}
