//! Callbacks of the variables tabs (`VariablesAdapter`) and the variable
//! dialog (`VariableDialogAdapter`).

use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::tabs::variables::{self, editor};
use crate::{AppWindow, VariableDialogAdapter, VariablesAdapter};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    let tab = ui.global::<VariablesAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, editor::add);
            fill_dialog(&ui, &app);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_edit(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| editor::edit(m, row));
            fill_dialog(&ui, &app);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_delete(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| variables::delete(m, row));
        });
    }
    let dialog = ui.global::<VariableDialogAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        dialog.on_save(move |name, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| editor::save(m, &name, &value));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        dialog.on_cancel(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, editor::cancel);
        });
    }
}

/// Fill the just-opened dialog's fields with the variable it edits (empty
/// for a new one). Only here: from then on they're the user's to type in.
fn fill_dialog(ui: &AppWindow, app: &AppRc) {
    let app = app.borrow();
    let Some(editor) = app.model().and_then(|model| model.variable_editor.as_ref()) else {
        return;
    };
    let (name, value) = editor.fields();
    let dialog = ui.global::<VariableDialogAdapter>();
    dialog.set_name(name.into());
    dialog.set_value(value.into());
}
