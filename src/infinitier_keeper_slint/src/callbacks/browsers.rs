//! Callbacks of the Item and Spell Browser windows. Each carries the
//! browser's kind (`0` items, `1` spells; see [`BrowserKind`]).

use infinitier_core::game::GameData;
use slint::ComponentHandle;

use super::{AppRc, on_app};
use crate::AppWindow;
use crate::app::{App, Session};
use crate::browser::{Browser, BrowserKind};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_toggle(move |kind| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.toggle(game_data)
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_close(move |kind| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.close());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_search(move |kind, text| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.search(&text));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_select(move |kind, resref| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.select(game_data, &resref);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_step(move |kind, delta| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.step(game_data, delta);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_all_types(move |kind, shown| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.set_all_types(shown));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_type(move |kind, name, shown| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| {
                browser.set_type_shown(&name, shown);
            });
        });
    }
    {
        // Double-clicking an inventory slot / a Spells-tab row.
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_reveal(move |kind, resref| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.reveal(game_data, &resref);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_browser_add(move |kind, placement| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session, browsers, ..
                } = app;
                if let (Session::Saves(model), Ok(kind)) = (session, BrowserKind::try_from(kind)) {
                    browsers.add(kind, model, placement);
                }
            });
        });
    }
}

/// Run `edit` on browser `kind` with the loaded game's data, then repaint.
fn on_browser(ui: &AppWindow, app: &AppRc, kind: i32, edit: impl FnOnce(&mut Browser, &GameData)) {
    on_app(ui, app, |app| {
        let App {
            session, browsers, ..
        } = app;
        if let (Some(game_data), Ok(kind)) = (session.game_data(), BrowserKind::try_from(kind)) {
            edit(browsers.get_mut(kind), game_data);
        }
    });
}
