//! The code shared by the Infinitier GUIs (the keeper and the explorer),
//! independent of their UI: their settings — the configured game installs,
//! which of them is active, and the colour theme.
//!
//! [`Settings`] is the settings file itself; [`SettingsDialog`] is the state
//! and logic behind each app's Settings dialog, which lists the games, adds
//! and removes them, and picks the one to load. Each app keeps its own file
//! (`infinitier_keeper.toml`, `infinitier_explorer.toml`) and draws the
//! dialog with its own UI.

mod dialog;
mod file;
mod form;

pub use dialog::SettingsDialog;
pub use file::{GameConfig, Preferences, Settings, SettingsError, ThemeSetting};
pub use form::game_type_options;
