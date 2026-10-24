//! State and logic behind the Save dialog. Duplicated from the egui keeper's
//! `ui/save_action.rs` (minus the egui rendering).
//!
//! The header's Save button opens a dialog pre-filled with the next free
//! `<current> (Edited NNNN)` folder name. On confirm, every save-game file
//! (`.sav`, `.gam`, `.bmp`, `.wmp` — case-insensitive) is copied into that
//! new sibling folder and the GAM is re-exported so the in-memory edits land
//! on disk. The original save is never modified; the active tab then follows
//! the new folder, so the next Save suggests the number after it.

use std::io;
use std::path::{Path, PathBuf};

use infinitier_core::imported_resource::gam::ImportedGam;
use infinitier_core::resource::gam::GamExporter;

use crate::state::Model;

/// File extensions a save folder ships. Case-insensitive on disk.
const SAVE_FILE_EXTS: &[&str] = &["sav", "gam", "bmp", "wmp"];

/// Maximum `(Edited NNNN)` slots probed by [`next_edited_save_name`].
const MAX_EDITED_SLOT: u32 = 9999;

#[derive(Default)]
pub struct SaveDialog {
    open: bool,
    /// Pre-filled folder name, rebuilt each time the dialog opens.
    folder_name: String,
    /// Last-attempt error, surfaced inline.
    error: Option<String>,
}

impl SaveDialog {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn folder_name(&self) -> &str {
        &self.folder_name
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Open the dialog for the active save of `model`, pre-filling the next
    /// free `(Edited NNNN)` slot in its parent folder.
    pub fn open(&mut self, model: &Model) {
        let folder = &model.save_game.folder_path;
        let parent = folder.parent().map(Path::to_path_buf).unwrap_or_default();
        self.folder_name = suggest_save_name(model.save_game.folder_name(), &parent);
        self.error = None;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Write the active save of `model` into a new folder named `name`.
    /// On success the dialog closes, the active tab now points at the new
    /// folder, and the new folder's path is returned; on failure the dialog
    /// stays open with the error.
    pub fn save(&mut self, name: &str, model: &mut Model) -> Option<PathBuf> {
        let name = name.trim();
        self.folder_name = name.to_owned();
        if name.is_empty() {
            self.error = Some("Folder name can't be empty.".into());
            return None;
        }
        let Some(parent) = model.save_game.folder_path.parent().map(Path::to_path_buf) else {
            self.error = Some("Save folder has no parent directory.".into());
            return None;
        };
        match perform_save_export(name, &model.save_game.folder_path, &parent, &model.gam) {
            Ok(dest) => {
                log::info!("[save] wrote {}", dest.display());
                model.save_game.set_folder_name(name.to_string());
                model.save_game.folder_path = dest.clone();
                self.open = false;
                self.error = None;
                Some(dest)
            }
            Err(e) => {
                log::error!(
                    "[save] failed to write '{name}' into {}: {e}",
                    parent.display()
                );
                self.error = Some(e.to_string());
                None
            }
        }
    }
}

/// Suggest the next free `<base> (Edited NNNN)` folder name in `parent`.
fn suggest_save_name(current: &str, parent: &Path) -> String {
    next_edited_save_name(current, |name| parent.join(name).exists())
}

/// Create `parent/name/`, then copy every `.sav` / `.gam` / `.bmp` / `.wmp`
/// file from `src` (non-recursive) into it. Returns the new folder's path.
fn perform_save_copy(name: &str, src: &Path, parent: &Path) -> io::Result<PathBuf> {
    let dest = parent.join(name);
    if dest.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("destination already exists: {}", dest.display()),
        ));
    }
    std::fs::create_dir(&dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let lower = file_name.to_string_lossy().to_lowercase();
        let matches_save_ext = SAVE_FILE_EXTS
            .iter()
            .any(|ext| lower.ends_with(&format!(".{ext}")));
        if !matches_save_ext {
            continue;
        }
        let from = entry.path();
        let to = dest.join(&file_name);
        std::fs::copy(&from, &to).map_err(|e| {
            io::Error::other(format!("copy {} → {}: {e}", from.display(), to.display()))
        })?;
    }
    Ok(dest)
}

/// Copy the save into `parent/name/` via [`perform_save_copy`], then
/// overwrite the copied `.GAM` with a fresh serialisation of `gam` — the
/// edits live in the embedded CRE blobs inside the GAM.
fn perform_save_export(
    name: &str,
    src: &Path,
    parent: &Path,
    gam: &ImportedGam,
) -> io::Result<PathBuf> {
    let dest = perform_save_copy(name, src, parent)?;
    let gam_path = std::fs::read_dir(&dest)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("gam"))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no .GAM file in copied save folder {}", dest.display()),
            )
        })?;
    let exported = gam.clone().export()?;
    GamExporter.export_to_file(&exported, &gam_path)?;
    Ok(dest)
}

/// Suggest the next free `<base> (Edited NNNN)` save name derived from
/// `current`, using `exists(name)` to skip names already taken.
///
/// If `current` already ends in a ` (Edited NNNN)` suffix, that suffix is
/// stripped to recover the base name and the search starts just past its
/// number — so re-saving `X (Edited 0002)` yields `X (Edited 0003)`, not
/// `X (Edited 0002) (Edited 0001)`. With no such suffix the search starts
/// at `0001`.
fn next_edited_save_name(current: &str, exists: impl Fn(&str) -> bool) -> String {
    let (base, existing) = split_edited_suffix(current);
    let start = existing.map(|n| n.saturating_add(1)).unwrap_or(1);
    for n in start..=MAX_EDITED_SLOT {
        let candidate = format!("{base} (Edited {n:04})");
        if !exists(&candidate) {
            return candidate;
        }
    }
    format!("{base} (Edited)")
}

/// Split a trailing ` (Edited <N>)` suffix off `name`, returning the base
/// name and the parsed number. `(name, None)` when there's no such suffix
/// (or its number doesn't parse).
fn split_edited_suffix(name: &str) -> (&str, Option<u32>) {
    const PREFIX: &str = " (Edited ";
    let Some(inner) = name.strip_suffix(')') else {
        return (name, None);
    };
    // `rfind` so the *last* suffix wins if the base also contains one.
    let Some(idx) = inner.rfind(PREFIX) else {
        return (name, None);
    };
    let digits = &inner[idx + PREFIX.len()..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return (name, None);
    }
    match digits.parse::<u32>() {
        Ok(n) => (&inner[..idx], Some(n)),
        Err(_) => (name, None),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn suggest_save_name_picks_first_free_slot() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("Quick (Edited 0001)")).unwrap();
        std::fs::create_dir(dir.path().join("Quick (Edited 0002)")).unwrap();
        assert_eq!(
            suggest_save_name("Quick", dir.path()),
            "Quick (Edited 0003)"
        );
    }

    #[test]
    fn perform_save_copy_writes_known_extensions_only() {
        let src = tempfile::tempdir().unwrap();
        let parent = tempfile::tempdir().unwrap();
        for name in [
            "BALDUR.GAM",
            "AR0072.sav",
            "PORTRT0.bmp",
            "WORLDMAP.WMP",
            "note.txt",
        ] {
            std::fs::write(src.path().join(name), b"x").unwrap();
        }
        let dest = perform_save_copy("My Save (Edited 0001)", src.path(), parent.path()).unwrap();
        let mut got: Vec<String> = std::fs::read_dir(&dest)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        got.sort();
        assert_eq!(
            got,
            ["AR0072.sav", "BALDUR.GAM", "PORTRT0.bmp", "WORLDMAP.WMP"]
        );
    }

    #[test]
    fn perform_save_copy_refuses_to_overwrite() {
        let src = tempfile::tempdir().unwrap();
        let parent = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("X.SAV"), b"sav").unwrap();
        std::fs::create_dir(parent.path().join("Existing")).unwrap();
        let err = perform_save_copy("Existing", src.path(), parent.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn split_edited_suffix_recognises_and_rejects() {
        assert_eq!(split_edited_suffix("Foo (Edited 0002)"), ("Foo", Some(2)));
        assert_eq!(split_edited_suffix("Quick-Save"), ("Quick-Save", None));
        assert_eq!(
            split_edited_suffix("Foo (Edited abc)"),
            ("Foo (Edited abc)", None)
        );
        assert_eq!(
            split_edited_suffix("A (Edited 0001) (Edited 0002)"),
            ("A (Edited 0001)", Some(2))
        );
    }

    #[test]
    fn next_edited_skips_taken_slots_and_never_nests() {
        let taken: HashSet<&str> = ["Quick-Save (Edited 0001)", "Quick-Save (Edited 0002)"]
            .into_iter()
            .collect();
        assert_eq!(
            next_edited_save_name("Quick-Save", |n| taken.contains(n)),
            "Quick-Save (Edited 0003)"
        );
        assert_eq!(
            next_edited_save_name("Quick-Save (Edited 0005)", |_| false),
            "Quick-Save (Edited 0006)"
        );
    }
}
