//! Callbacks of the Item and Spell Browser windows (`BrowsersAdapter`).
//! Each carries the browser's kind.

use infinitier_core::game::GameData;
use slint::ComponentHandle;

use super::{AppRc, on_app};
use crate::app::{App, Session};
use crate::browser::Browser;
use crate::{AppWindow, BrowserKind, BrowsersAdapter};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    let browsers = ui.global::<BrowsersAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_toggle(move |kind| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.toggle(game_data)
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_close(move |kind| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.close());
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_search(move |kind, text| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.search(&text));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_select(move |kind, resref| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.select(game_data, &resref);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_step(move |kind, delta| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.step(game_data, delta);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_all_types(move |kind, shown| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| browser.set_all_types(shown));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_type_shown(move |kind, name, shown| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, _| {
                browser.set_type_shown(&name, shown);
            });
        });
    }
    {
        // Double-clicking an inventory slot / a Spells-tab row.
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_reveal(move |kind, resref| {
            let Some(ui) = weak.upgrade() else { return };
            on_browser(&ui, &app, kind, |browser, game_data| {
                browser.reveal(game_data, &resref);
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        browsers.on_add(move |kind, placement| {
            let Some(ui) = weak.upgrade() else { return };
            on_app(&ui, &app, |app| {
                let App {
                    session, browsers, ..
                } = app;
                if let Session::Saves(model) = session {
                    browsers.add(kind, model, placement);
                }
            });
        });
    }
}

/// Run `edit` on browser `kind` with the loaded game's data, then repaint.
fn on_browser(
    ui: &AppWindow,
    app: &AppRc,
    kind: BrowserKind,
    edit: impl FnOnce(&mut Browser, &GameData),
) {
    on_app(ui, app, |app| {
        let App {
            session, browsers, ..
        } = app;
        if let Some(game_data) = session.game_data() {
            edit(browsers.get_mut(kind), game_data);
        }
    });
}
