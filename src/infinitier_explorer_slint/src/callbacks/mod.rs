//! Wiring of the Slint adapters' callbacks to the explorer's state, one
//! module per part of the UI in `ui/`. Every callback mutates the shared
//! [`App`], repaints the UI from it, and lets the [`Ticker`] follow the
//! viewer's playback.

mod shell;
mod tree;
mod viewers;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, Weak};

use crate::AppWindow;
use crate::app::App;
use crate::ticker::Ticker;
use crate::view;

/// The state shared by every callback.
pub type AppRc = Rc<RefCell<App>>;

/// What a callback needs: the window, the state and the playback timer.
#[derive(Clone)]
struct Ctx {
    ui: Weak<AppWindow>,
    app: AppRc,
    ticker: Rc<Ticker>,
}

impl Ctx {
    /// Run `edit` on the app, then repaint and re-time the ticker.
    fn update(&self, edit: impl FnOnce(&mut App)) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        {
            let mut app = self.app.borrow_mut();
            edit(&mut app);
            view::refresh(&ui, &app);
        }
        self.ticker.sync(&ui, &self.app);
    }
}

/// Connect every UI callback to `app`.
pub fn wire(ui: &AppWindow, app: &AppRc, ticker: &Rc<Ticker>) {
    let ctx = Ctx {
        ui: ui.as_weak(),
        app: app.clone(),
        ticker: ticker.clone(),
    };
    shell::wire(ui, &ctx);
    tree::wire(ui, &ctx);
    viewers::wire(ui, &ctx);
}
