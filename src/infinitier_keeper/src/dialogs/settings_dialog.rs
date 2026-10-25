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
//! started with the keeper, so the list is ready when the dialog first
//! opens).
//!
//! Selecting a row picks the game to edit (and the one loaded again at the
//! next startup): when the dialog closes with a game other than the loaded
//! one selected, [`SettingsDialog::close`] hands that config back to the
//! caller, which loads it in place of the current game.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use infinitier_core::game_detect::detect_game_at;
use infinitier_core::resource::Game;
use thiserror::Error;

use crate::GameRowUi;
use crate::settings::{GameConfig, Settings, ThemeSetting};

/// Every game type the add form offers besides autodetection.
const GAME_TYPES: [Game; 14] = [
    Game::Bg,
    Game::Bgee,
    Game::BgeeSod,
    Game::Bg2,
    Game::Bg2ee,
    Game::Eet,
    Game::Tutu,
    Game::Iwd {
        heart_of_winter: false,
        totl: false,
    },
    Game::Iwd {
        heart_of_winter: true,
        totl: false,
    },
    Game::Iwd {
        heart_of_winter: true,
        totl: true,
    },
    Game::Iwdee,
    Game::Iwd2,
    Game::Pst,
    Game::Pstee,
];

const AUTODETECT: &str = "Autodetect";

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
    /// otherwise [`Self::error`] says why not. `game_type` is the dropdown
    /// index: `0` is Autodetect, `n` is `GAME_TYPES[n - 1]`.
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

    /// The games list as table rows.
    pub fn rows(&self) -> Vec<GameRowUi> {
        self.settings
            .games
            .iter()
            .map(|game| GameRowUi {
                type_label: game_type_label(game, &self.detections).into(),
                game_path: game.game_path.display().to_string().into(),
                save_path: game
                    .save_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
                    .into(),
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

/// The add form's game-type dropdown entries: Autodetect, then every type.
pub fn game_type_options() -> Vec<String> {
    std::iter::once(AUTODETECT.to_owned())
        .chain(GAME_TYPES.iter().map(|game| format!("{game:?}")))
        .collect()
}

/// The add form's input.
struct GameForm<'a> {
    game_path: &'a str,
    save_path: &'a str,
    /// Picked entry of the game-type dropdown: `0` is Autodetect, `n` is
    /// `GAME_TYPES[n - 1]`.
    game_type: usize,
}

impl GameForm<'_> {
    fn game_type(&self) -> Option<Game> {
        let index = self.game_type.checked_sub(1)?;
        GAME_TYPES.get(index).copied()
    }

    /// Validate the input into a config not already in `games`.
    fn parse(&self, games: &[GameConfig]) -> Result<GameConfig, FormError> {
        let game_path = PathBuf::from(self.game_path.trim());
        if game_path.as_os_str().is_empty() {
            return Err(FormError::MissingGamePath);
        }
        if !game_path.is_dir() {
            return Err(FormError::NotAFolder(game_path));
        }
        if !has_key_file(&game_path) {
            return Err(FormError::NoKeyFile(game_path));
        }
        let save_path = match self.save_path.trim() {
            "" => None,
            path => {
                let path = PathBuf::from(path);
                if !path.is_dir() {
                    return Err(FormError::NotAFolder(path));
                }
                Some(path)
            }
        };
        let config = GameConfig {
            game_path,
            save_path,
            game_type: self.game_type(),
        };
        if games.contains(&config) {
            return Err(FormError::Duplicate);
        }
        Ok(config)
    }
}

/// Why the add form was rejected.
#[derive(Debug, Error, PartialEq)]
enum FormError {
    #[error("Enter the game folder.")]
    MissingGamePath,
    #[error("\"{}\" is not a folder.", .0.display())]
    NotAFolder(PathBuf),
    #[error("\"{}\" has no CHITIN.KEY — pick the game's install folder.", .0.display())]
    NoKeyFile(PathBuf),
    #[error("This game is already in the list.")]
    Duplicate,
}

/// Whether `dir` directly holds a `CHITIN.KEY`, in any letter case.
fn has_key_file(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.file_name().eq_ignore_ascii_case("chitin.key"))
    })
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

    fn form<'a>(game_path: &'a str, save_path: &'a str) -> GameForm<'a> {
        GameForm {
            game_path,
            save_path,
            game_type: 0,
        }
    }

    #[test]
    fn parse_accepts_an_install_with_a_key_file_in_any_case() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Chitin.KEY"), b"").unwrap();
        let path = dir.path().display().to_string();
        let config = form(&path, "  ").parse(&[]).unwrap();
        assert_eq!(
            config,
            GameConfig {
                game_path: dir.path().to_path_buf(),
                save_path: None,
                game_type: None,
            }
        );
    }

    #[test]
    fn parse_rejects_bad_folders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().display().to_string();
        assert_eq!(form("", "").parse(&[]), Err(FormError::MissingGamePath));
        let missing = dir.path().join("missing");
        let missing_str = missing.display().to_string();
        assert_eq!(
            form(&missing_str, "").parse(&[]),
            Err(FormError::NotAFolder(missing.clone()))
        );
        assert_eq!(
            form(&path, "").parse(&[]),
            Err(FormError::NoKeyFile(dir.path().to_path_buf()))
        );
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        assert_eq!(
            form(&path, &missing_str).parse(&[]),
            Err(FormError::NotAFolder(missing))
        );
    }

    #[test]
    fn parse_rejects_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        let path = dir.path().display().to_string();
        let existing = form(&path, "").parse(&[]).unwrap();
        assert_eq!(
            form(&path, "").parse(&[existing]),
            Err(FormError::Duplicate)
        );
    }

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

    #[test]
    fn game_type_index_zero_autodetects() {
        let mut form = form("/games", "");
        assert_eq!(form.game_type(), None);
        form.game_type = 1;
        assert_eq!(form.game_type(), Some(GAME_TYPES[0]));
        form.game_type = GAME_TYPES.len();
        assert_eq!(form.game_type(), Some(Game::Pstee));
        assert_eq!(game_type_options().len(), GAME_TYPES.len() + 1);
    }
}
