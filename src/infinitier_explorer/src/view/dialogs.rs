//! The modal dialogs: Settings (`SettingsAdapter`) and the loading overlay
//! (`LoadingAdapter`).

use slint::{Color, ComponentHandle, Image, Model, ModelRc};

use super::rows_model;
use crate::app::App;
use crate::{AppWindow, LoadingAdapter, SettingsAdapter, TableCell, TableCellKind};

pub(super) fn refresh(ui: &AppWindow, app: &App) {
    let settings = &app.settings;
    let adapter = ui.global::<SettingsAdapter>();
    adapter.set_open(settings.is_open());
    if settings.is_open() {
        let rows = settings.rows();
        if rows_changed(&adapter.get_rows(), &rows) {
            adapter.set_rows(rows_model(
                rows.into_iter()
                    .map(|cells| rows_model(cells.into_iter().map(text_cell).collect()))
                    .collect(),
            ));
        }
        let selected = settings.selected().and_then(|i| i32::try_from(i).ok());
        adapter.set_selected(selected.unwrap_or(-1));
        adapter.set_file_error(settings.file_error().unwrap_or_default().into());
        adapter.set_error(settings.error().unwrap_or_default().into());
    }

    let adapter = ui.global::<LoadingAdapter>();
    adapter.set_active(app.pending.is_some());
    if let Some(pending) = &app.pending {
        adapter.set_text(format!("Loading {}…", pending.config.game_path.display()).into());
    }
}

/// Whether the table showing `current` must be given `rows` instead. The
/// rows are only replaced when they change: a new model rebuilds the row
/// elements, which drops a click in progress — the second click of a
/// double-click lands on a fresh row and never registers as one.
fn rows_changed(current: &ModelRc<ModelRc<TableCell>>, rows: &[[String; 3]]) -> bool {
    current.row_count() != rows.len()
        || current.iter().zip(rows).any(|(old, new)| {
            !old.iter()
                .map(|cell| cell.text)
                .eq(new.iter().map(|text| text.as_str()))
        })
}

/// A table cell showing `text`.
fn text_cell(text: String) -> TableCell {
    TableCell {
        kind: TableCellKind::Text,
        text: text.into(),
        image: Image::default(),
        colorize: Color::default(),
        value: 0.0,
        minimum: 0.0,
        maximum: 0.0,
        step: 0.0,
    }
}
