//! Top-level state of the keeper: which game (if any) is loaded and
//! which saves are open, the shell's dialogs, and game switching.
//!
//! The keeper starts with no game; it loads the settings' active game, or one
//! picked in the Settings dialog, on a background thread while a modal
//! "Loading…" overlay with a spinner blocks the UI.

use std::any::Any;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use infinitier_core::fs::Importer;
use infinitier_core::game::GameData;
use infinitier_core::imported_resource::gam::ImportedGam;
use infinitier_core::resource::Game;
use infinitier_core::resource::gam::GamImporter;
use infinitier_core::save_games::SaveGame;
use infinitier_gui_common::{GameConfig, SettingsDialog};
use slint::{ComponentHandle, Timer, TimerMode};

use crate::AppWindow;
use crate::browser::Browsers;
use crate::dialogs::load_dialog::LoadDialog;
use crate::dialogs::save_dialog::SaveDialog;
use crate::state::{LoadedGame, Model};
use crate::view;

/// How often a running game load is checked for completion.
const LOAD_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// What the keeper has open.
pub enum Session {
    NoGame,
    /// A game is loaded but no save is open.
    Game(Box<LoadedGame>),
    /// A game with at least one open save.
    Saves(Box<Model>),
}

impl Session {
    /// The loaded game's data, with or without open saves.
    pub fn game_data(&self) -> Option<&GameData> {
        match self {
            Session::NoGame => None,
            Session::Game(game) => Some(&game.game_data),
            Session::Saves(model) => Some(&model.game_data),
        }
    }

    /// Mutable access to the loaded game's data.
    pub fn game_data_mut(&mut self) -> Option<&mut GameData> {
        match self {
            Session::NoGame => None,
            Session::Game(game) => Some(&mut game.game_data),
            Session::Saves(model) => Some(&mut model.game_data),
        }
    }
}

/// A game being read on a background thread.
pub struct PendingGame {
    pub config: GameConfig,
    loading: JoinHandle<Result<LoadedGame, String>>,
}

pub struct App {
    pub session: Session,
    /// The loaded game's Item and Spell Browser windows.
    pub browsers: Browsers,
    pub settings: SettingsDialog,
    pub load_dialog: LoadDialog,
    pub save_dialog: SaveDialog,
    /// The game being loaded, if any; its overlay blocks the UI meanwhile.
    pub pending: Option<PendingGame>,
    /// Polls [`Self::pending`] for completion while a load runs.
    loading_timer: Timer,
    /// One-line feedback in the header (e.g. where a save was written).
    pub status: String,
}

impl App {
    pub fn new(settings: SettingsDialog) -> Self {
        Self {
            session: Session::NoGame,
            browsers: Browsers::default(),
            settings,
            load_dialog: LoadDialog::default(),
            save_dialog: SaveDialog::default(),
            pending: None,
            loading_timer: Timer::default(),
            status: String::new(),
        }
    }

    pub fn model(&self) -> Option<&Model> {
        match &self.session {
            Session::Saves(model) => Some(model),
            Session::NoGame | Session::Game(_) => None,
        }
    }

    pub fn model_mut(&mut self) -> Option<&mut Model> {
        match &mut self.session {
            Session::Saves(model) => Some(model),
            Session::NoGame | Session::Game(_) => None,
        }
    }

    /// The loaded game's type.
    pub fn game(&self) -> Option<Game> {
        match &self.session {
            Session::NoGame => None,
            Session::Game(game) => Some(game.game_data.game()),
            Session::Saves(model) => Some(model.game_data.game()),
        }
    }

    /// Open `save` into a tab and make it active — or just focus its tab
    /// when it is already open.
    pub fn open_save(&mut self, save: SaveGame) -> Result<(), String> {
        if let Some(tab) = self.model().and_then(|m| m.tab_of(&save.folder_path)) {
            if let Some(model) = self.model_mut() {
                model.switch_save(tab);
            }
            return Ok(());
        }
        let game_data = self.session.game_data_mut().ok_or("no game is loaded")?;
        let gam = import_save(&save, game_data)?;
        log::info!("[load] opened save {}", save.folder_path.display());
        match std::mem::replace(&mut self.session, Session::NoGame) {
            Session::Saves(mut model) => {
                model.open_save(save, Box::new(gam));
                self.session = Session::Saves(model);
            }
            Session::Game(game) => {
                self.session = Session::Saves(Box::new(Model::new(*game, save, Box::new(gam))))
            }
            Session::NoGame => {}
        }
        Ok(())
    }

    /// Close save tab `i`. Closing the last one keeps the game loaded with
    /// no save open.
    pub fn close_save(&mut self, i: usize) {
        match std::mem::replace(&mut self.session, Session::NoGame) {
            Session::Saves(model) if model.parked.len() == 1 && i == 0 => {
                self.session = Session::Game(Box::new(model.into_game()));
            }
            Session::Saves(mut model) => {
                model.close_save(i);
                self.session = Session::Saves(model);
            }
            other => self.session = other,
        }
    }
}

/// Import and resolve the GAM of `save` against `game_data`.
fn import_save(save: &SaveGame, game_data: &GameData) -> Result<ImportedGam, String> {
    let gam = GamImporter {
        name: save.folder_name(),
        engine: game_data.game().engine(),
    }
    .import(&save.gam)
    .map_err(|e| format!("couldn't read \"{}\": {e}", save.folder_name()))?;
    ImportedGam::load(gam, game_data)
        .map_err(|e| format!("couldn't load \"{}\": {e}", save.folder_name()))
}

/// Start reading `config`'s game on a background thread. The loading
/// overlay stays up until the thread finishes; then the game replaces the
/// current one (closing its saves), or the Settings dialog reopens with the
/// error and the current game stays.
pub fn start_loading(app_rc: &Rc<RefCell<App>>, ui: &AppWindow, config: GameConfig) {
    let mut app = app_rc.borrow_mut();
    let roots = config.roots();
    let game_type = config.game_type;
    let spawned = thread::Builder::new()
        .name("game-load".into())
        .spawn(move || LoadedGame::load(&roots, game_type));
    match spawned {
        Ok(loading) => {
            app.pending = Some(PendingGame { config, loading });
            let (app_rc, weak) = (app_rc.clone(), ui.as_weak());
            app.loading_timer
                .start(TimerMode::Repeated, LOAD_POLL_INTERVAL, move || {
                    let Some(ui) = weak.upgrade() else { return };
                    let mut app = app_rc.borrow_mut();
                    if let Some(pending) = app.pending.take_if(|p| p.loading.is_finished()) {
                        app.loading_timer.stop();
                        finish_loading(&mut app, pending);
                        view::refresh(&ui, &mut app);
                    }
                });
        }
        Err(e) => load_failed(&mut app, &config, &e),
    }
    view::refresh(ui, &mut app);
}

/// Swap in the game a finished background load produced, or report why it
/// couldn't be loaded.
fn finish_loading(app: &mut App, pending: PendingGame) {
    let PendingGame { config, loading } = pending;
    let game = match loading.join() {
        Ok(Ok(game)) => game,
        Ok(Err(e)) => return load_failed(app, &config, &e),
        Err(panic) => return load_failed(app, &config, &panic_message(panic.as_ref())),
    };
    log::info!(
        "[settings] switched to {:?} at {}",
        game.game_data.game(),
        config.game_path.display()
    );
    app.session = Session::Game(Box::new(game));
    app.browsers = Browsers::default();
    app.load_dialog.close();
    app.save_dialog.close();
    app.status.clear();
    app.settings.game_loaded(config);
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
