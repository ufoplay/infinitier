//! State and logic behind the Settings dialog — the list of configured game
//! installs.
//!
//! The dialog lists every [`GameConfig`] in the settings file, has a Remove
//! button for the selected row and a form to add a new install: game folder,
//! optional save folder, and game type (autodetected by default). Every
//! change to the list is written to the settings file straight away.
//!
//! The Game type column shows each game's short name; for autodetected ones,
//! the type detected from the install folders (by a background thread
//! started with the app, so the list is ready when the dialog first opens).
//!
//! Selecting a row picks the game to load (and the one loaded again at the
//! next startup): when the dialog closes with a game other than the loaded
//! one selected, [`SettingsDialog::close`] hands that config back to the
//! caller, which loads it in place of the current game.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use infinitier_core::game_detect::detect_game_at;
use infinitier_core::resource::Game;

use crate::file::{GameConfig, Settings, ThemeSetting};
use crate::form::GameForm;

/// Game type detected for each autodetect config; `None` when its folders
/// hold no recognised game.
type Detections = HashMap<GameConfig, Option<Game>>;

pub struct SettingsDialog {
    open: bool,
    /// Where [`Self::settings`] is persisted.
    path: PathBuf,
    settings: Settings,
    /// The config the loaded game was built from; `None` until a game is
    /// loaded.
    active: Option<GameConfig>,
    /// Index into the games list of the highlighted row.
    selected: Option<usize>,
    /// Detected game types of the autodetect configs.
    detections: Detections,
    /// Detection of the configs read at startup, still running in the
    /// background until the dialog first opens and collects it.
    startup_detections: Option<JoinHandle<Detections>>,
    /// Why the settings file couldn't be read at startup. Shown until the
    /// next successful write replaces the file.
    file_error: Option<String>,
    /// Last failed action (bad form input, failed write, failed game load),
    /// surfaced inline.
    error: Option<String>,
}

impl SettingsDialog {
    /// Read the settings file at `path`. A file that can't be read leaves
    /// the list empty and says why in the dialog.
    pub fn load(path: PathBuf) -> Self {
        // Anchor a relative path to the current folder now, so logs and
        // errors name the real file.
        let path = std::path::absolute(&path).unwrap_or(path);
        log::info!("[settings] using {}", path.display());
        let (settings, file_error) = match Settings::load(&path) {
            Ok(settings) => (settings, None),
            Err(e) => {
                log::error!("[settings] {e}");
                (Settings::default(), Some(e.to_string()))
            }
        };
        if let Some(index) = settings.preferences.active
            && settings.active_game().is_none()
        {
            log::warn!("[settings] active game {index} is not in the list, ignored");
        }
        let startup_detections = spawn_detections(&settings.games);
        Self {
            open: false,
            path,
            settings,
            active: None,
            selected: None,
            detections: Detections::new(),
            startup_detections,
            file_error,
            error: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the dialog with the loaded game's row selected.
    pub fn open(&mut self) {
        self.error = None;
        self.selected = self.active_index();
        self.update_detections();
        self.open = true;
    }

    /// Close the dialog. Returns the game to switch to, when the selected
    /// row isn't the loaded game.
    pub fn close(&mut self) -> Option<GameConfig> {
        self.open = false;
        let config = self.selected.and_then(|i| self.settings.games.get(i))?;
        (self.active.as_ref() != Some(config)).then(|| config.clone())
    }

    /// Highlight row `row` of the games list.
    pub fn select(&mut self, row: usize) {
        if row < self.settings.games.len() {
            self.selected = Some(row);
        }
    }

    /// The game the settings file marks as active, to load at startup.
    pub fn startup_game(&self) -> Option<GameConfig> {
        self.settings.active_game().cloned()
    }

    /// `config` was loaded (at startup, or picked on close): it becomes the
    /// active game, recorded in the settings file if that changes it.
    pub fn game_loaded(&mut self, config: GameConfig) {
        self.active = Some(config);
        let index = self.active_index();
        if self.settings.preferences.active != index {
            self.settings.preferences.active = index;
            self.persist();
        }
    }

    /// `config` couldn't be loaded: reopen with the error, the selection
    /// back on the still-loaded game so closing again doesn't retry the load.
    pub fn game_load_failed(&mut self, config: &GameConfig, err: impl Display) {
        self.open();
        self.error = Some(format!(
            "Couldn't load \"{}\": {err}",
            config.game_path.display()
        ));
    }

    /// The colour theme stored in the settings file.
    pub fn theme(&self) -> ThemeSetting {
        self.settings.preferences.theme
    }

    /// Record the theme the user switched to in the settings file.
    pub fn set_theme(&mut self, theme: ThemeSetting) {
        if self.settings.preferences.theme != theme {
            self.settings.preferences.theme = theme;
            self.persist();
        }
    }

    /// Validate the add form's input and append the game to the list.
    /// Returns whether it was added (so the caller clears the form);
    /// otherwise [`Self::error`] says why not. `game_type` is the index into
    /// [`game_type_options`](crate::game_type_options): `0` is Autodetect.
    pub fn add_game(&mut self, game_path: &str, save_path: &str, game_type: usize) -> bool {
        self.error = None;
        let form = GameForm {
            game_path,
            save_path,
            game_type,
        };
        match form.parse(&self.settings.games) {
            Ok(config) => {
                log::info!("[settings] added game {}", config.game_path.display());
                self.settings.games.push(config);
                self.update_detections();
                self.persist();
                true
            }
            Err(e) => {
                self.error = Some(e.to_string());
                false
            }
        }
    }

    /// Drop the selected game from the list and persist it. The game stays
    /// loaded if it was the active one.
    pub fn remove_selected(&mut self) {
        self.error = None;
        let Some(i) = self.selected.take() else {
            return;
        };
        if i < self.settings.games.len() {
            let removed = self.settings.games.remove(i);
            log::info!("[settings] removed game {}", removed.game_path.display());
            let active = &mut self.settings.preferences.active;
            *active = active_after_removal(*active, i);
            self.persist();
        }
    }

    /// The games list as table rows: each game's type, game path and
    /// savegame path.
    pub fn rows(&self) -> Vec<[String; 3]> {
        self.settings
            .games
            .iter()
            .map(|game| {
                [
                    game_type_label(game, &self.detections),
                    game.game_path.display().to_string(),
                    game.save_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                ]
            })
            .collect()
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn file_error(&self) -> Option<&str> {
        self.file_error.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Write the settings to the settings file.
    fn persist(&mut self) {
        match self.settings.save(&self.path) {
            Ok(()) => self.file_error = None,
            Err(e) => {
                log::error!("[settings] {e}");
                self.error = Some(e.to_string());
            }
        }
    }

    /// Give every autodetect config a detected type: collect the startup
    /// detection (waiting for it if still running), then detect any config
    /// it didn't cover — one added since — on the spot.
    fn update_detections(&mut self) {
        if let Some(startup) = self.startup_detections.take() {
            match startup.join() {
                Ok(detections) => self.detections.extend(detections),
                Err(_) => log::error!("[settings] game type detection thread panicked"),
            }
        }
        let missing: Vec<GameConfig> = self
            .settings
            .games
            .iter()
            .filter(|g| g.game_type.is_none() && !self.detections.contains_key(*g))
            .cloned()
            .collect();
        self.detections.extend(detect_all(missing));
    }

    /// Row of the active game, if it is (still) listed.
    fn active_index(&self) -> Option<usize> {
        let active = self.active.as_ref()?;
        self.settings.games.iter().position(|g| g == active)
    }
}

/// The stored active index once row `removed` is gone: still on the same
/// game, or `None` if that game was the one removed (nothing to load at
/// startup any more).
fn active_after_removal(active: Option<usize>, removed: usize) -> Option<usize> {
    match active {
        Some(a) if a == removed => None,
        Some(a) if a > removed => Some(a - 1),
        unchanged => unchanged,
    }
}

/// Detect the game type of each of `configs`.
fn detect_all(configs: Vec<GameConfig>) -> Detections {
    configs
        .into_iter()
        .map(|config| {
            let game = detect_game_at(config.roots().as_slice());
            (config, game)
        })
        .collect()
}

/// Detect the autodetect configs among `games` on a background thread
/// (indexing every install folder takes a noticeable fraction of a second
/// in all). `None` if the thread couldn't be started — the dialog then
/// detects when it opens.
fn spawn_detections(games: &[GameConfig]) -> Option<JoinHandle<Detections>> {
    let autodetect: Vec<GameConfig> = games
        .iter()
        .filter(|g| g.game_type.is_none())
        .cloned()
        .collect();
    thread::Builder::new()
        .name("settings-detect".into())
        .spawn(move || detect_all(autodetect))
        .inspect_err(|e| log::error!("[settings] can't start game type detection: {e}"))
        .ok()
}

/// The Game type cell: the short name of the chosen type, or of the
/// detected one for an autodetect config.
fn game_type_label(config: &GameConfig, detections: &Detections) -> String {
    match config.game_type {
        Some(game) => game.short_name().to_owned(),
        None => match detections.get(config).copied().flatten() {
            Some(game) => format!("{} (autodetected)", game.short_name()),
            None => "Not detected".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_type_label_uses_short_names() {
        let config = |path: &str, game_type| GameConfig {
            game_path: PathBuf::from(path),
            save_path: None,
            game_type,
        };
        let chosen = config("/games/bg2ee", Some(Game::Bg2ee));
        let detected = config("/games/bgee", None);
        let unknown = config("/games/empty", None);
        let detections = Detections::from([
            (detected.clone(), Some(Game::Bgee)),
            (unknown.clone(), None),
        ]);

        assert_eq!(game_type_label(&chosen, &detections), "BG2EE");
        assert_eq!(
            game_type_label(&detected, &detections),
            "BGEE (autodetected)"
        );
        assert_eq!(game_type_label(&unknown, &detections), "Not detected");
    }

    #[test]
    fn active_index_follows_its_game_on_removal() {
        assert_eq!(active_after_removal(Some(3), 1), Some(2));
        assert_eq!(active_after_removal(Some(3), 3), None);
        assert_eq!(active_after_removal(Some(3), 5), Some(3));
        assert_eq!(active_after_removal(None, 0), None);
    }
}
