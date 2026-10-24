use std::any::Any;
use std::fmt::Display;
use std::thread::{self, JoinHandle};

use eframe::egui;
use egui_components::theme::Theme;
use egui_components::{Dialog, Label};

use crate::game_view::GameView;
use crate::settings::GameConfig;
use crate::state::{AppState, GameLoadError};
use crate::ui::{HeaderPanel, SettingsAction};

pub struct KeeperApp {
    header_panel: HeaderPanel,
    /// The loaded game; `None` until one is loaded from the settings.
    game: Option<GameView>,
    /// Settings dialog: the configured games, and which one to load.
    settings_action: SettingsAction,
    /// Game being loaded: the settings' active game at startup, or one
    /// picked in the Settings dialog.
    pending_game: Option<PendingGame>,
}

/// A game being read on a background thread, so the UI keeps painting its
/// "Loading…" modal — spinner included, everything underneath blocked —
/// until the thread is done.
struct PendingGame {
    config: GameConfig,
    loading: JoinHandle<Result<AppState, GameLoadError>>,
}

impl KeeperApp {
    /// Start by loading the settings' active game, if they name one; the
    /// Settings dialog picks any other.
    pub fn new(settings_action: SettingsAction) -> Self {
        let mut app = Self {
            header_panel: HeaderPanel,
            game: None,
            settings_action,
            pending_game: None,
        };
        if let Some(config) = app.settings_action.startup_game() {
            app.start_loading(config);
        }
        app
    }

    /// Start reading `config`'s game data on a background thread.
    fn start_loading(&mut self, config: GameConfig) {
        let roots = config.roots();
        let game_type = config.game_type;
        let spawned = thread::Builder::new()
            .name("game-load".into())
            .spawn(move || AppState::load(&roots, game_type));
        match spawned {
            Ok(loading) => self.pending_game = Some(PendingGame { config, loading }),
            Err(e) => self.load_failed(&config, &e),
        }
    }

    /// Swap in the game a finished background load produced, in place of
    /// the current one, if any. Every open save and every per-game cache
    /// belongs to the old game, so its whole view is replaced. If the load
    /// failed the current game stays, and the Settings dialog reopens with
    /// the error.
    fn finish_loading(&mut self, ctx: &egui::Context, pending: PendingGame) {
        let PendingGame { config, loading } = pending;
        let state = match loading.join() {
            Ok(Ok(state)) => state,
            Ok(Err(e)) => return self.load_failed(&config, &e),
            Err(panic) => return self.load_failed(&config, &panic_message(panic.as_ref())),
        };
        log::info!(
            "[settings] switched to {:?} at {}",
            state.game_data.game(),
            config.game_path.display()
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(state.window_title()));
        clear_frame_store(ctx);
        self.settings_action.game_loaded(config);
        self.game = Some(GameView::new(state));
    }

    fn load_failed(&mut self, config: &GameConfig, err: &dyn Display) {
        log::error!(
            "[settings] failed to load {}: {err}",
            config.game_path.display()
        );
        self.settings_action.game_load_failed(config, err);
    }
}

impl eframe::App for KeeperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // A background load that finished: swap its game in before anything
        // reads the game state.
        if let Some(pending) = self.pending_game.take_if(|p| p.loading.is_finished()) {
            self.finish_loading(ui.ctx(), pending);
        }
        let header_action = self.header_panel.show(ui, self.game.is_some());
        if header_action.settings_clicked {
            self.settings_action.open();
        }
        if let Some(theme) = header_action.theme_changed {
            self.settings_action.set_theme(theme);
        }
        if let Some(config) = self.settings_action.show(ui.ctx()) {
            self.start_loading(config);
        }
        match &mut self.game {
            Some(game) => game.show(ui, header_action),
            None => no_game_state(ui),
        }
        if let Some(pending) = &self.pending_game {
            // The spinner keeps requesting frames, so the finished load is
            // picked up on the next one.
            loading_notice(ui.ctx(), &pending.config);
        }
    }
}

/// Drop everything memoised in egui's frame store. The character tabs
/// cache game-derived data there (TLK strings, palette, calendar, item
/// icons, …), which would otherwise leak the previous game into the new
/// one. Widget state (scroll offsets, column widths) goes with it. The
/// theme is kept in the same store, so it is reinstalled.
fn clear_frame_store(ctx: &egui::Context) {
    let theme = Theme::get(ctx);
    ctx.data_mut(|d| d.clear());
    theme.install(ctx);
}

/// Modal painted while a game loads (see [`PendingGame`]). Re-opened every
/// frame, so Esc or a backdrop click can't dismiss it; being modal, it
/// blocks the rest of the UI.
fn loading_notice(ctx: &egui::Context, config: &GameConfig) {
    let mut open = true;
    Dialog::new("Loading game")
        .no_close_button()
        .show(ctx, &mut open, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new());
                ui.add(Label::new(format!(
                    "Loading {}…",
                    config.game_path.display()
                )));
            });
        });
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

/// Placeholder shown until a game is loaded, pointing at the Settings
/// dialog where games are added and picked.
fn no_game_state(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading("No game loaded");
            ui.add_space(6.0);
            ui.label(
                "Open Settings to add a game, then select it and close the dialog to load it.",
            );
        });
    });
}
