//! The Item and Spell Browser windows (`BrowsersAdapter`).

use slint::ComponentHandle;

use super::{rows_changed, rows_model};
use crate::app::{App, Session};
use crate::browser::BrowserView;
use crate::{AppWindow, BrowserKind, BrowsersAdapter};

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

/// Push one browser window's state. The list and the type checkboxes are
/// only replaced when they change (see [`rows_changed`]); the search text
/// only when it isn't what the user typed.
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
    match kind {
        BrowserKind::Items => {
            if rows_changed(&adapter.get_item_rows(), &rows) {
                adapter.set_item_rows(rows_model(rows));
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
            if rows_changed(&adapter.get_spell_rows(), &rows) {
                adapter.set_spell_rows(rows_model(rows));
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
