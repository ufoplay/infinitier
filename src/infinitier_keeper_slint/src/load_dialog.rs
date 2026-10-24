//! State and logic behind the Load dialog — the *Open Single Player Saved
//! Game* picker. Duplicated from the egui keeper's `ui/load_action.rs`
//! (minus the egui rendering).
//!
//! The dialog lists every discovered save game (folder name + the time it
//! was last written), newest first. Selecting a row previews its screenshot
//! and party portraits. Load opens the selected save into a tab; Delete
//! removes the save folder from disk after a confirmation prompt. The list
//! is read fresh from disk each time the dialog opens (and after a delete).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use infinitier_core::fs::{DataSource, Importer};
use infinitier_core::game::GameData;
use infinitier_core::imported_resource::image::ImportedImage;
use infinitier_core::resource::bmp::BmpImporter;
use infinitier_core::save_games::SaveGame;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

use crate::LoadRowUi;

/// One discovered save game, with its display timestamp precomputed.
struct SaveEntry {
    save: SaveGame,
    /// Newest file mtime in the save folder, used for sorting.
    modified: SystemTime,
    /// `modified` formatted like EEKeeper's "Time Saved" column.
    time_label: String,
}

#[derive(Default)]
pub struct LoadDialog {
    open: bool,
    /// Discovered saves, newest first. Rebuilt on open and after delete.
    entries: Vec<SaveEntry>,
    /// Index into [`Self::entries`] of the highlighted row.
    selected: Option<usize>,
    /// Preview of the selected save.
    screenshot: Image,
    portraits: Vec<Image>,
    /// Whether the delete confirmation is showing.
    confirm_delete: bool,
    /// Last error (failed load / delete), surfaced inline.
    error: Option<String>,
}

impl LoadDialog {
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the picker, scanning the save directory fresh.
    pub fn open(&mut self, game_data: &mut GameData) {
        self.error = None;
        self.confirm_delete = false;
        self.rescan(game_data);
        self.select((!self.entries.is_empty()).then_some(0));
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.confirm_delete = false;
    }

    /// Highlight `row` and decode its preview.
    pub fn select(&mut self, row: Option<usize>) {
        self.selected = row.filter(|&i| i < self.entries.len());
        let entry = self.selected.and_then(|i| self.entries.get(i));
        self.screenshot = entry
            .and_then(|e| e.save.screenshot.as_ref())
            .and_then(|ds| bmp_image(ds, "save-screenshot"))
            .unwrap_or_default();
        self.portraits = entry
            .map(|e| {
                e.save
                    .portraits
                    .iter()
                    .filter_map(|ds| bmp_image(ds, "save-portrait"))
                    .collect()
            })
            .unwrap_or_default();
    }

    /// The highlighted save, to open.
    pub fn selected_save(&self) -> Option<&SaveGame> {
        self.selected
            .and_then(|i| self.entries.get(i))
            .map(|e| &e.save)
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Ask for confirmation before deleting the highlighted save.
    pub fn ask_delete(&mut self) {
        self.error = None;
        self.confirm_delete = self.selected.is_some();
    }

    pub fn cancel_delete(&mut self) {
        self.confirm_delete = false;
    }

    pub fn confirming_delete(&self) -> bool {
        self.confirm_delete
    }

    /// Delete the highlighted save's folder from disk, then rescan and keep
    /// the selection on the same row. A tab that has the save open keeps
    /// its in-memory copy.
    pub fn delete(&mut self, game_data: &mut GameData) {
        self.confirm_delete = false;
        let Some((row, folder)) = self
            .selected
            .and_then(|i| self.entries.get(i).map(|e| (i, e.save.folder_path.clone())))
        else {
            return;
        };
        if let Err(e) = std::fs::remove_dir_all(&folder) {
            self.error = Some(format!("Couldn't delete save: {e}"));
            return;
        }
        log::info!("[load] deleted save folder {}", folder.display());
        self.rescan(game_data);
        let row = (!self.entries.is_empty()).then(|| row.min(self.entries.len() - 1));
        self.select(row);
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Display name of the highlighted save (for the delete prompt).
    pub fn selected_name(&self) -> String {
        self.selected_save()
            .map(|s| s.display_name().to_string())
            .unwrap_or_default()
    }

    pub fn rows(&self) -> Vec<LoadRowUi> {
        self.entries
            .iter()
            .map(|e| LoadRowUi {
                name: e.save.display_name().into(),
                time: e.time_label.as_str().into(),
            })
            .collect()
    }

    pub fn screenshot(&self) -> Image {
        self.screenshot.clone()
    }

    pub fn portraits(&self) -> Vec<Image> {
        self.portraits.clone()
    }

    /// Rescan the save directory and rebuild [`Self::entries`], newest
    /// first. Refreshes the save folders on the game's file system first, so
    /// saves written or deleted since the game was loaded are reflected.
    fn rescan(&mut self, game_data: &mut GameData) {
        let save_games = match game_data.refresh_save_games() {
            Ok(save_games) => save_games,
            Err(e) => {
                self.error = Some(format!("Couldn't read saves: {e}"));
                return;
            }
        };
        let mut entries: Vec<SaveEntry> = save_games
            .saves()
            .iter()
            .map(|save| {
                let modified = folder_mtime(&save.folder_path);
                SaveEntry {
                    save: save.clone(),
                    modified,
                    time_label: format_time(modified),
                }
            })
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.modified));
        self.entries = entries;
    }
}

/// Decode a save-folder BMP (screenshot / portrait) into a Slint image.
fn bmp_image(ds: &DataSource, name: &str) -> Option<Image> {
    let bmp = BmpImporter { name }.import(ds).ok()?;
    let buffer = ImportedImage::from_bmp(bmp).image;
    let (width, height) = (buffer.width(), buffer.height());
    if width == 0 || height == 0 {
        return None;
    }
    let mut pixels = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    pixels.make_mut_bytes().copy_from_slice(buffer.as_raw());
    Some(Image::from_rgba8(pixels))
}

/// Most recent file mtime within a save folder (the actual save time),
/// falling back to the folder's own mtime.
fn folder_mtime(folder: &Path) -> SystemTime {
    let folder_time = std::fs::metadata(folder)
        .and_then(|m| m.modified())
        .unwrap_or(UNIX_EPOCH);
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.metadata().and_then(|m| m.modified()).ok())
        .fold(folder_time, SystemTime::max)
}

/// Format a timestamp like EEKeeper's column, in the local timezone:
/// "Thursday, March 26, 2026 8:53:32 PM".
fn format_time(time: SystemTime) -> String {
    match jiff::Timestamp::try_from(time) {
        Ok(ts) => ts
            .to_zoned(jiff::tz::TimeZone::system())
            .strftime("%A, %B %-d, %Y %-I:%M:%S %p")
            .to_string(),
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_mtime_is_the_newest_file_time() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("old.sav");
        let new = dir.path().join("new.gam");
        std::fs::write(&old, b"x").unwrap();
        std::fs::write(&new, b"x").unwrap();
        let newest = std::fs::metadata(&new).unwrap().modified().unwrap();
        assert!(folder_mtime(dir.path()) >= newest);
    }

    #[test]
    fn format_time_is_empty_only_for_unrepresentable_times() {
        assert!(!format_time(UNIX_EPOCH).is_empty());
    }
}
