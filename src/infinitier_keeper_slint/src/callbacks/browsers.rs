//! Callbacks of the Item and Spell Browser overlays.

use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::{AppWindow, browser};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    // Item Browser: open/close, search, select, add.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_items_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::toggle_items);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_item_search(move |q| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| browser::search_items(m, q.as_str()));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_item_select(move |r| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| browser::select_item(m, r.as_str()));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_item_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::add_item);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_item_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::close_items);
        });
    }

    // Spell Browser: open/close, search, select, add.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spells_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::toggle_spells);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_browser_search(move |q| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| browser::search_spells(m, q.as_str()));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_browser_select(move |r| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| browser::select_spell(m, r.as_str()));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_browser_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::add_spell);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_browser_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, browser::close_spells);
        });
    }
}
