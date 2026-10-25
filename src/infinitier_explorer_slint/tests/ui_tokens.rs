//! The UI is built on slint-themekit (see `PORT_PLAN.md`): its
//! design values come from the themekit's `Theme` tokens and its widgets from
//! the themekit, never from `std-widgets.slint`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use slint_themekit::lint;

/// The explorer's own layout values (`Metrics`) and data colours
/// (`SyntaxColors`, `OverlayColors`), like an app's theme files: the one
/// place literal design values belong.
const EXPLORER_THEMES: &str = "themes";

fn ui_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ui")
}

/// Every `.slint` file under `dir`.
fn slint_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            files.extend(slint_files(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "slint") {
            files.push(path);
        }
    }
    Ok(files)
}

#[test]
fn ui_reads_design_values_from_tokens() -> io::Result<()> {
    let findings = lint::scan_dir(&ui_dir(), &[EXPLORER_THEMES])?;
    assert!(findings.is_empty(), "{}", lint::report(&findings));
    Ok(())
}

#[test]
fn ui_does_not_use_std_widgets() -> io::Result<()> {
    let ui = ui_dir();
    let mut offenders = Vec::new();
    for path in slint_files(&ui)? {
        if fs::read_to_string(&path)?.contains("std-widgets.slint") {
            let relative = path.strip_prefix(&ui).unwrap_or(&path);
            offenders.push(relative.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "these files import std-widgets.slint: {offenders:?}"
    );
    Ok(())
}
