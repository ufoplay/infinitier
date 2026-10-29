//! The Item and Spell Browser windows (`BrowsersAdapter`).

use slint::ComponentHandle;

use super::{rows_changed, rows_model, sync_table};
use crate::app::{App, Session};
use crate::browser::{BrowserRow, BrowserView};
use crate::components::table;
use crate::{AppWindow, BrowserKind, BrowsersAdapter, TableCell};

/// Repaint both browser windows. Rows are only built while a window is open
/// (an index can be thousands of entries).
pub fn refresh_browsers(ui: &AppWindow, app: &mut App) {
    let App {
        session, browsers, ..
    } = app;
    let model = match &*session {
        Session::Saves(model) => Some(&**model),
        Session::NoGame | Session::Game(_) => None,
    };
    for kind in BrowserKind::ALL {
        let view = match session.game_data() {
            Some(game_data) => browsers.view(kind, game_data, model),
            None => BrowserView::closed(),
        };
        push_browser(ui, kind, view);
    }
}

/// Push one browser window's state. The list is synced in place (see
/// [`sync_table`]) and the type checkboxes only replaced when they change
/// (see [`rows_changed`]); the search text only when it isn't what the user
/// typed.
fn push_browser(ui: &AppWindow, kind: BrowserKind, view: BrowserView) {
    let BrowserView {
        state,
        rows,
        types,
        text,
        all_types,
        scroll_to,
    } = view;
    let adapter = ui.global::<BrowsersAdapter>();
    let rows = rows.into_iter().map(|row| cells(kind, row)).collect();
    match kind {
        BrowserKind::Items => {
            if let Some(rows) = sync_table(&adapter.get_item_rows(), rows) {
                adapter.set_item_rows(rows);
            }
            if rows_changed(&adapter.get_item_types(), &types) {
                adapter.set_item_types(rows_model(types));
            }
            if adapter.get_item_text() != text {
                adapter.set_item_text(text);
            }
            adapter.set_item_all_types(all_types);
            adapter.set_item_state(state);
        }
        BrowserKind::Spells => {
            if let Some(rows) = sync_table(&adapter.get_spell_rows(), rows) {
                adapter.set_spell_rows(rows);
            }
            if rows_changed(&adapter.get_spell_types(), &types) {
                adapter.set_spell_types(rows_model(types));
            }
            if adapter.get_spell_text() != text {
                adapter.set_spell_text(text);
            }
            adapter.set_spell_all_types(all_types);
            adapter.set_spell_state(state);
        }
    }
    if let Some(row) = scroll_to {
        ui.invoke_browser_scroll_to(kind, row);
    }
}

/// A list row's cells, in the window's columns: the type, the name and the
/// resref, plus a spell's level after its type and its script name last.
fn cells(kind: BrowserKind, row: BrowserRow) -> Vec<TableCell> {
    let BrowserRow {
        resref,
        name,
        category,
        level,
        script,
    } = row;
    match kind {
        BrowserKind::Items => vec![
            table::text(category),
            table::text(name),
            table::text(resref),
        ],
        BrowserKind::Spells => vec![
            table::text(category),
            table::text(level),
            table::text(name),
            table::text(resref),
            table::text(script),
        ],
    }
}
