//! Wiring of the Slint UI's callbacks to the keeper's state. Every callback
//! mutates the shared [`App`] and then repaints the whole UI from it.

mod browsers;
mod character;
mod shell;

use std::cell::RefCell;
use std::rc::Rc;

use crate::AppWindow;
use crate::app::App;
use crate::state::Model;
use crate::view;

/// The state shared by every callback.
type AppRc = Rc<RefCell<App>>;

/// Connect every UI callback to `app`.
pub fn wire(ui: &AppWindow, app: &AppRc) {
    character::wire(ui, app);
    browsers::wire(ui, app);
    shell::wire(ui, app);
}

/// Run `edit` on the open saves' model, if any, then repaint.
fn on_model(ui: &AppWindow, app: &AppRc, edit: impl FnOnce(&mut Model)) {
    let mut app = app.borrow_mut();
    if let Some(model) = app.model_mut() {
        edit(model);
    }
    view::refresh(ui, &mut app);
}

/// Run `edit` on the whole app, then repaint.
fn on_app(ui: &AppWindow, app: &AppRc, edit: impl FnOnce(&mut App)) {
    let mut app = app.borrow_mut();
    edit(&mut app);
    view::refresh(ui, &mut app);
}
