//! "Settings" dialog for the egui keeper — the list of configured game
//! installs.
//!
//! The header's Settings button pops a modal listing every
//! [`GameConfig`] in the settings file, a Remove button for the selected
//! row, and a form to add a new install: game folder, optional save
//! folder, and game type (autodetected by default). Every change to the
//! list is written to the settings file straight away.
//!
//! The Game type column shows each game's short name; for autodetected
//! ones, the type detected from the install folders (by a background
//! thread started with the keeper, so the list is ready when the dialog
//! first opens).
//!
//! Selecting a row picks the game to edit (and the one loaded again at the
//! next startup): when the dialog closes (Close,
//! ×, Esc, the backdrop, or double-clicking a row) with a game other than
//! the loaded one selected, [`SettingsAction::show`] hands that config
//! back to the host, which loads it in place of the current game.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use eframe::egui;
use egui_components::{Button, Dialog, Input, Label, LabelTone, Select, Table, TableColumn};
use infinitier_core::game_detect::detect_game_at;
use infinitier_core::resource::Game;
use thiserror::Error;

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

/// Width of the add form's inputs.
const INPUT_WIDTH: f32 = 470.0;

/// Owned state for the Settings dialog. Lives on `KeeperApp` so it
/// survives across frames — and across game switches, which rebuild the
/// rest of the app.
pub struct SettingsAction {
    open: bool,
    /// Where [`Self::settings`] is persisted.
    path: PathBuf,
    settings: Settings,
    /// The config the loaded game was built from; `None` until a game is
    /// loaded.
    active: Option<GameConfig>,
    /// Index into the games list of the highlighted row.
    selected: Option<usize>,
    form: GameForm,
    /// Detected game types of the autodetect configs.
    detections: Detections,
    /// Detection of the configs read at startup, still running in the
    /// background until the dialog first opens and collects it.
    startup_detections: Option<JoinHandle<Detections>>,
    /// Why the settings file couldn't be read at startup. Shown until the
    /// next successful write replaces the file.
    file_error: Option<String>,
    /// Last failed action (bad form input, failed write, failed game
    /// load), surfaced inline.
    error: Option<String>,
}

impl SettingsAction {
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
            form: GameForm::default(),
            detections: Detections::new(),
            startup_detections,
            file_error,
            error: None,
        }
    }

    /// Open the dialog with the loaded game's row selected.
    pub fn open(&mut self) {
        self.error = None;
        self.selected = self.active_index();
        self.update_detections();
        self.open = true;
    }

    /// The game the settings file marks as active, to load at startup.
    pub fn startup_game(&self) -> Option<GameConfig> {
        self.settings.active_game().cloned()
    }

    /// The host loaded `config` (at startup, or picked on close): it becomes
    /// the active game, recorded in the settings file if that changes it.
    pub fn game_loaded(&mut self, config: GameConfig) {
        self.active = Some(config);
        let index = self.active_index();
        if self.settings.preferences.active != index {
            self.settings.preferences.active = index;
            self.persist();
        }
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

    /// The host couldn't load `config`: reopen with the error, the
    /// selection back on the still-loaded game so closing again doesn't
    /// retry the load.
    pub fn game_load_failed(&mut self, config: &GameConfig, err: impl Display) {
        self.open();
        self.error = Some(format!(
            "Couldn't load \"{}\": {err}",
            config.game_path.display()
        ));
    }

    /// Paint the dialog if open. Returns the game to switch to on the
    /// frame the dialog closes with a game other than the loaded one
    /// selected.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<GameConfig> {
        if !self.open {
            return None;
        }

        let mut action = None;
        {
            // Split-borrow so the body can edit the selection / form while
            // `Dialog::show` holds `&mut open`.
            let SettingsAction {
                open,
                settings,
                selected,
                form,
                detections,
                file_error,
                error,
                ..
            } = self;
            Dialog::new("Settings").width(640.0).show(ctx, open, |ui| {
                action = render_body(
                    ui,
                    &settings.games,
                    detections,
                    selected,
                    form,
                    file_error.as_deref(),
                    error.as_deref(),
                );
            });
        }

        match action {
            Some(DialogAction::Add) => self.add_game(),
            Some(DialogAction::Remove) => self.remove_selected(),
            Some(DialogAction::Close) => self.open = false,
            None => {}
        }
        if self.open {
            return None;
        }

        // Closed this frame.
        let config = self.selected.and_then(|i| self.settings.games.get(i))?;
        (self.active.as_ref() != Some(config)).then(|| config.clone())
    }

    /// Append the form's game to the list and persist it.
    fn add_game(&mut self) {
        self.error = None;
        match self.form.parse(&self.settings.games) {
            Ok(config) => {
                log::info!("[settings] added game {}", config.game_path.display());
                self.settings.games.push(config);
                self.form = GameForm::default();
                self.update_detections();
                self.persist();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    /// Drop the selected game from the list and persist it. The game
    /// stays loaded if it was the active one.
    fn remove_selected(&mut self) {
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

/// The add form's input buffers.
#[derive(Default)]
struct GameForm {
    game_path: String,
    save_path: String,
    /// Picked entry of the game-type dropdown: `0` is Autodetect, `n` is
    /// `GAME_TYPES[n - 1]`. `None` (nothing picked yet) autodetects too.
    game_type: Option<usize>,
}

impl GameForm {
    fn game_type(&self) -> Option<Game> {
        let index = self.game_type?.checked_sub(1)?;
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

/// Action chosen in the dialog body, resolved after `Dialog::show`
/// releases its borrows.
enum DialogAction {
    Add,
    Remove,
    Close,
}

/// Paint the games list + add form + buttons; return the action clicked,
/// if any.
fn render_body(
    ui: &mut egui::Ui,
    games: &[GameConfig],
    detections: &Detections,
    selected: &mut Option<usize>,
    form: &mut GameForm,
    file_error: Option<&str>,
    error: Option<&str>,
) -> Option<DialogAction> {
    let mut action = None;

    if let Some(err) = file_error {
        error_label(ui, err);
        ui.add_space(6.0);
    }

    ui.add(Label::new("Games").strong());
    ui.add_space(4.0);
    // Double-clicking a row selects it and closes, loading it.
    if games_table(ui, games, detections, selected) {
        action = Some(DialogAction::Close);
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui
            .add(
                Button::danger("Remove")
                    .small()
                    .disabled(selected.is_none()),
            )
            .clicked()
        {
            action = Some(DialogAction::Remove);
        }
        ui.add(
            Label::new("Select a game and close this dialog to load it.").tone(LabelTone::Muted),
        );
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);
    ui.add(Label::new("Add a game").strong());
    ui.add_space(4.0);
    if add_form(ui, form) {
        action = Some(DialogAction::Add);
    }

    if let Some(err) = error {
        ui.add_space(6.0);
        error_label(ui, err);
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(Button::primary("Close")).clicked() {
                action = Some(DialogAction::Close);
            }
        });
    });

    action
}

/// The selectable games list (type / game folder / save folder). Returns
/// `true` when a row was double-clicked.
fn games_table(
    ui: &mut egui::Ui,
    games: &[GameConfig],
    detections: &Detections,
    selected: &mut Option<usize>,
) -> bool {
    if games.is_empty() {
        ui.add(Label::new("No games configured yet — add one below.").tone(LabelTone::Muted));
        return false;
    }
    let mut picked = false;
    ui.scope(|ui| {
        // Selectable label text would take the click (as a text-selection
        // drag) before the row sees it, so clicking a cell's text wouldn't
        // select its row.
        ui.style_mut().interaction.selectable_labels = false;
        Table::new("settings-games")
            .selectable(true)
            .max_height(220.0)
            .column(
                TableColumn::initial(190.0)
                    .at_least(120.0)
                    .clip(true)
                    .header("Game type"),
            )
            .column(
                TableColumn::initial(220.0)
                    .at_least(120.0)
                    .clip(true)
                    .header("Game path"),
            )
            .column(TableColumn::remainder().clip(true).header("Savegame path"))
            .show(ui, |body| {
                body.rows(games.len(), |i, mut row| {
                    let Some(game) = games.get(i) else {
                        return;
                    };
                    row.selected(*selected == Some(i));
                    row.col(|ui| {
                        ui.add(Label::new(game_type_label(game, detections)));
                    });
                    row.col(|ui| path_label(ui, &game.game_path));
                    row.col(|ui| {
                        if let Some(save_path) = &game.save_path {
                            path_label(ui, save_path);
                        }
                    });
                    let resp = row.response();
                    if resp.clicked() {
                        *selected = Some(i);
                    }
                    if resp.double_clicked() {
                        *selected = Some(i);
                        picked = true;
                    }
                });
            });
    });
    picked
}

/// A (clipped) path cell, with the full path on hover.
fn path_label(ui: &mut egui::Ui, path: &Path) {
    let text = path.display().to_string();
    ui.add(Label::new(text.clone())).on_hover_text(text);
}

/// Paint the add-game inputs; returns `true` when Add was clicked.
fn add_form(ui: &mut egui::Ui, form: &mut GameForm) -> bool {
    let mut add = false;
    egui::Grid::new("settings-add-game")
        .num_columns(2)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            ui.add(Label::new("Game path"));
            ui.add(
                Input::new(&mut form.game_path)
                    .placeholder("Folder containing CHITIN.KEY")
                    .width(INPUT_WIDTH),
            );
            ui.end_row();

            ui.add(Label::new("Savegame path"));
            ui.add(
                Input::new(&mut form.save_path)
                    .placeholder("Optional — folder containing save/, if outside the game folder")
                    .width(INPUT_WIDTH),
            );
            ui.end_row();

            ui.add(Label::new("Game type"));
            let labels = std::iter::once(AUTODETECT.to_owned())
                .chain(GAME_TYPES.iter().map(|game| format!("{game:?}")));
            Select::new("settings-game-type", &mut form.game_type)
                .options(labels)
                .placeholder(AUTODETECT)
                .width(INPUT_WIDTH)
                .show(ui);
            ui.end_row();

            ui.label("");
            if ui.add(Button::primary("Add").small()).clicked() {
                add = true;
            }
            ui.end_row();
        });
    add
}

fn error_label(ui: &mut egui::Ui, err: &str) {
    ui.colored_label(egui::Color32::from_rgb(0xE5, 0x4A, 0x4A), err);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(game_path: &Path, save_path: &str) -> GameForm {
        GameForm {
            game_path: game_path.display().to_string(),
            save_path: save_path.to_owned(),
            game_type: None,
        }
    }

    #[test]
    fn parse_accepts_an_install_with_a_key_file_in_any_case() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Chitin.KEY"), b"").unwrap();
        let config = form(dir.path(), "  ").parse(&[]).unwrap();
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
        assert_eq!(
            form(Path::new(""), "").parse(&[]),
            Err(FormError::MissingGamePath)
        );
        let missing = dir.path().join("missing");
        assert_eq!(
            form(&missing, "").parse(&[]),
            Err(FormError::NotAFolder(missing.clone()))
        );
        assert_eq!(
            form(dir.path(), "").parse(&[]),
            Err(FormError::NoKeyFile(dir.path().to_path_buf()))
        );
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        assert_eq!(
            form(dir.path(), &missing.display().to_string()).parse(&[]),
            Err(FormError::NotAFolder(missing))
        );
    }

    #[test]
    fn parse_rejects_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("chitin.key"), b"").unwrap();
        let existing = form(dir.path(), "").parse(&[]).unwrap();
        assert_eq!(
            form(dir.path(), "").parse(&[existing]),
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
    fn game_type_index_zero_and_none_autodetect() {
        let mut form = form(Path::new("/games"), "");
        assert_eq!(form.game_type(), None);
        form.game_type = Some(0);
        assert_eq!(form.game_type(), None);
        form.game_type = Some(1);
        assert_eq!(form.game_type(), Some(GAME_TYPES[0]));
        form.game_type = Some(GAME_TYPES.len());
        assert_eq!(form.game_type(), Some(Game::Pstee));
    }
}
