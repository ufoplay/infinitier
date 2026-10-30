//! The Settings dialog's add form: its game-type choices, and the validation
//! of its input into a [`GameConfig`].

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use infinitier_core::resource::Game;
use infinitier_core::save_games::SAVE_DIRS;
use thiserror::Error;

use crate::file::GameConfig;

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

/// The add form's game-type dropdown entries: Autodetect, then every type.
pub fn game_type_options() -> Vec<String> {
    std::iter::once(AUTODETECT.to_owned())
        .chain(GAME_TYPES.iter().map(|game| format!("{game:?}")))
        .collect()
}

/// The add form's input.
pub(crate) struct GameForm<'a> {
    pub game_path: &'a str,
    pub save_path: &'a str,
    /// Picked entry of the game-type dropdown: `0` is Autodetect, `n` is
    /// `GAME_TYPES[n - 1]`.
    pub game_type: usize,
}

impl GameForm<'_> {
    fn game_type(&self) -> Option<Game> {
        let index = self.game_type.checked_sub(1)?;
        GAME_TYPES.get(index).copied()
    }

    /// Validate the input into a config not already in `games`.
    pub fn parse(&self, games: &[GameConfig]) -> Result<GameConfig, FormError> {
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
            path => Some(savegame_root(PathBuf::from(path))?),
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
pub(crate) enum FormError {
    #[error("Enter the game folder.")]
    MissingGamePath,
    #[error("\"{}\" is not a folder.", .0.display())]
    NotAFolder(PathBuf),
    #[error("\"{}\" has no CHITIN.KEY — pick the game's install folder.", .0.display())]
    NoKeyFile(PathBuf),
    #[error("\"{}\" has no save folder — pick the folder containing save/, or save/ itself.", .0.display())]
    NoSaveFolder(PathBuf),
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

/// The savegame path to keep for the picked folder `path`: the folder
/// holding the save folders ([`SAVE_DIRS`]: `save/`, `mpsave/`, …), as the
/// game reads its other files from it too. That is the parent when `path`
/// is a save folder itself, else `path`, if it holds one.
fn savegame_root(path: PathBuf) -> Result<PathBuf, FormError> {
    if !path.is_dir() {
        return Err(FormError::NotAFolder(path));
    }
    if path.file_name().is_some_and(is_save_dir_name) {
        return Ok(match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            // A bare relative `save`: its parent is the current folder.
            _ => PathBuf::from("."),
        });
    }
    let holds_save_dir = std::fs::read_dir(&path).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| is_save_dir_name(&e.file_name()) && e.path().is_dir())
    });
    if holds_save_dir {
        Ok(path)
    } else {
        Err(FormError::NoSaveFolder(path))
    }
}

/// Whether `name` is one of the [`SAVE_DIRS`], in any letter case.
fn is_save_dir_name(name: &OsStr) -> bool {
    SAVE_DIRS.iter().any(|dir| name.eq_ignore_ascii_case(dir))
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
    fn parse_keeps_the_folder_holding_the_save_folders() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        let user = dir.path().join("user");
        std::fs::create_dir_all(user.join("MPSave")).unwrap();
        std::fs::create_dir_all(user.join("SODSAVE")).unwrap();
        let game = dir.path().display().to_string();
        let save_path = |picked: &Path| {
            form(&game, &picked.display().to_string())
                .parse(&[])
                .map(|config| config.save_path)
        };
        // The folder holding a save folder, or a save folder itself, in
        // any letter case.
        assert_eq!(save_path(&user), Ok(Some(user.clone())));
        assert_eq!(save_path(&user.join("MPSave")), Ok(Some(user.clone())));
        assert_eq!(save_path(&user.join("SODSAVE")), Ok(Some(user.clone())));
    }

    #[test]
    fn parse_rejects_a_savegame_folder_without_save_folders() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        let user = dir.path().join("user");
        std::fs::create_dir_all(user.join("portraits")).unwrap();
        // A file named like a save folder doesn't count.
        std::fs::write(user.join("save"), b"").unwrap();
        let game = dir.path().display().to_string();
        assert_eq!(
            form(&game, &user.display().to_string()).parse(&[]),
            Err(FormError::NoSaveFolder(user))
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
