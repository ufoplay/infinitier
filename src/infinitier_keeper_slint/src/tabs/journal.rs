//! Journal Entries tab for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/journal_entries/`. The party journal
//! lives in the GAM (not per-creature); each entry points at a `dialog.tlk`
//! strref for its text and carries a section bitfield + chapter + timestamp.
//! `journal_rows`/`section_label` and the Harptos `calendar` are duplicated
//! from the egui sources. Rendered as a 4-column wrapping grid (Journal Type /
//! Journal Entry / Chapter / Time). Display-only — no `commit`.

mod calendar;

use infinitier_core::imported_resource::gam::ImportedGam;
use infinitier_core::resource::gam::GameTicks;
use slint::{ModelRc, SharedString, VecModel};

use crate::GridRow;
use crate::state::Model;
use calendar::Calendar;

pub const HEADERS: &[&str] = &["Journal Type", "Journal Entry", "Chapter", "Time"];
pub const WIDTHS: &[f32] = &[120.0, 520.0, 70.0, 280.0];

struct JournalRow {
    type_label: &'static str,
    chapter: u32,
    time: GameTicks,
    strref: u32,
}

fn journal_rows(gam: &ImportedGam) -> Vec<JournalRow> {
    gam.journal
        .iter()
        .map(|j| JournalRow {
            type_label: section_label(j.section),
            chapter: u32::from(j.chapter),
            time: j.time,
            strref: j.strref,
        })
        .collect()
}

fn section_label(section: u8) -> &'static str {
    match section {
        0 => "User",
        1 => "Quest",
        2 => "Done Quest",
        4 => "Info",
        _ => "Other",
    }
}

/// Format an in-game timestamp the way the engine's journal does: the full
/// Harptos `Day N, Hour H (DD Month, Year)` when the calendar resources are
/// present, else a plain day/clock fallback.
fn time_text(calendar: Option<&Calendar>, time: GameTicks) -> String {
    if let Some(cal) = calendar {
        return cal.format(time);
    }
    let dhm = time.dhm();
    format!("Day {}, {:02}:{:02}", dhm.day, dhm.hour, dhm.minute)
}

fn grid_row(cells: Vec<SharedString>) -> GridRow {
    GridRow {
        cells: ModelRc::new(VecModel::from(cells)),
    }
}

/// Build the Journal-tab grid rows for the save.
pub fn build(model: &Model) -> Vec<GridRow> {
    let gam = model.gam.as_ref();
    let rows = journal_rows(gam);
    // Resolve TLK text + build the calendar once per build.
    let tlk = model.game_data.dialog_tlk().ok();
    let calendar = Calendar::load(&model.game_data);

    rows.into_iter()
        .map(|r| {
            let text = tlk
                .as_ref()
                .and_then(|t| t.get(r.strref))
                .unwrap_or_default();
            grid_row(vec![
                r.type_label.into(),
                text.into(),
                r.chapter.to_string().into(),
                time_text(calendar.as_ref(), r.time).into(),
            ])
        })
        .collect()
}
