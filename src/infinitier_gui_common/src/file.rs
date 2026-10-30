//! The persistent settings of an app — the list of configured game installs
//! the Settings dialog offers to switch between, which of them is active,
//! and the colour theme.
//!
//! Stored as TOML at the app's `--config` path (by default its own file in
//! the current folder), read once at startup and rewritten whenever any of
//! it changes:
//!
//! ```toml
//! [settings]
//! # Index into `games` (from 0) of the game loaded at startup.
//! active = 0
//! theme = "dark"
//!
//! [[games]]
//! game_path = "/games/Baldur's Gate II - Enhanced Edition"
//! save_path = "/home/me/Documents/Baldur's Gate II - Enhanced Edition"
//!
//! [[games]]
//! game_path = "/games/Planescape Torment"
//! game_type = "Pst"
//! ```

use std::io;
use std::path::{Path, PathBuf};

use infinitier_core::resource::Game;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One configured game install.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameConfig {
    /// Game install folder (the one holding `CHITIN.KEY`).
    pub game_path: PathBuf,
    /// Folder holding the `save/` directory, when it lives outside the
    /// install (the Enhanced Editions keep it under the user's Documents).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_path: Option<PathBuf>,
    /// Game type to load the install as; `None` autodetects it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_type: Option<Game>,
}

impl GameConfig {
    /// Folders to build the game data from: the install first, then the
    /// save folder overlaid on top.
    pub fn roots(&self) -> Vec<PathBuf> {
        std::iter::once(self.game_path.clone())
            .chain(self.save_path.clone())
            .collect()
    }
}

/// The colour theme, as stored in the settings file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSetting {
    #[default]
    Light,
    Dark,
}

/// The `[settings]` table: app-wide preferences.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    /// Index into [`Settings::games`] of the active game — the last one
    /// loaded from the Settings dialog, loaded again at startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<usize>,
    #[serde(default)]
    pub theme: ThemeSetting,
}

/// Everything persisted in the settings file.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default, rename = "settings")]
    pub preferences: Preferences,
    #[serde(default)]
    pub games: Vec<GameConfig>,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("couldn't read {}: {source}", path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("couldn't parse {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("couldn't serialize the settings: {0}")]
    Serialize(#[source] toml::ser::Error),
    #[error("couldn't write {}: {source}", path.display())]
    Write { path: PathBuf, source: io::Error },
}

impl Settings {
    /// Read the settings at `path`. A missing file is not an error — it
    /// yields empty settings, as on first run.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(SettingsError::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        toml::from_str(&text).map_err(|source| SettingsError::Parse {
            path: path.to_owned(),
            source,
        })
    }

    /// Write the settings to `path`, replacing whatever is there.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        let text = toml::to_string(self).map_err(SettingsError::Serialize)?;
        std::fs::write(path, text).map_err(|source| SettingsError::Write {
            path: path.to_owned(),
            source,
        })
    }

    /// The active game, if [`Preferences::active`] names one of the
    /// listed games.
    pub fn active_game(&self) -> Option<&GameConfig> {
        self.games.get(self.preferences.active?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Name of the settings file the tests write.
    const SETTINGS_FILE: &str = "settings.toml";

    fn sample() -> Settings {
        Settings {
            preferences: Preferences {
                active: Some(1),
                theme: ThemeSetting::Dark,
            },
            games: vec![
                GameConfig {
                    game_path: PathBuf::from("/games/BG2EE"),
                    save_path: Some(PathBuf::from("/docs/Baldur's Gate II - Enhanced Edition")),
                    game_type: None,
                },
                GameConfig {
                    game_path: PathBuf::from("/games/IWD"),
                    save_path: None,
                    game_type: Some(Game::Iwd {
                        heart_of_winter: true,
                        totl: false,
                    }),
                },
                GameConfig {
                    game_path: PathBuf::from("/games/PST"),
                    save_path: None,
                    game_type: Some(Game::Pst),
                },
            ],
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        sample().save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), sample());
    }

    #[test]
    fn unset_values_are_omitted() {
        let toml = toml::to_string(&Settings {
            preferences: Preferences::default(),
            games: vec![GameConfig {
                game_path: PathBuf::from("/games/BG"),
                save_path: None,
                game_type: None,
            }],
        })
        .unwrap();
        assert_eq!(
            toml,
            "[settings]\ntheme = \"light\"\n\n[[games]]\ngame_path = \"/games/BG\"\n"
        );
    }

    #[test]
    fn missing_or_empty_file_loads_as_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
        std::fs::write(&path, "\n").unwrap();
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
    }

    #[test]
    fn malformed_file_is_a_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        std::fs::write(&path, "[[games]\ngame_path = ").unwrap();
        assert!(matches!(
            Settings::load(&path),
            Err(SettingsError::Parse { .. })
        ));
    }

    #[test]
    fn active_game_needs_an_index_in_range() {
        let mut settings = sample();
        assert_eq!(
            settings.active_game().map(|g| g.game_path.as_path()),
            Some(Path::new("/games/IWD"))
        );
        settings.preferences.active = Some(3);
        assert_eq!(settings.active_game(), None);
        settings.preferences.active = None;
        assert_eq!(settings.active_game(), None);
    }

    #[test]
    fn roots_put_the_install_before_the_save_folder() {
        let [with_saves, without_saves, _] = sample().games.try_into().unwrap();
        assert_eq!(
            with_saves.roots(),
            vec![
                PathBuf::from("/games/BG2EE"),
                PathBuf::from("/docs/Baldur's Gate II - Enhanced Edition"),
            ]
        );
        assert_eq!(without_saves.roots(), vec![PathBuf::from("/games/IWD")]);
    }
}
