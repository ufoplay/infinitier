//! The variable dialog (`VariableDialogAdapter`), from the open
//! [`VariableEditor`]. Its fields are filled when it opens (see
//! `callbacks::variables`), then they're the user's: a repaint doesn't
//! touch them.
//!
//! [`VariableEditor`]: crate::tabs::variables::editor::VariableEditor

use slint::ComponentHandle;

use crate::state::Model;
use crate::{AppWindow, VariableDialogAdapter};

pub(super) fn refresh(ui: &AppWindow, model: &Model) {
    let adapter = ui.global::<VariableDialogAdapter>();
    let Some(editor) = &model.variable_editor else {
        adapter.set_open(false);
        return;
    };
    adapter.set_title(editor.title().into());
    adapter.set_error(editor.error().into());
    adapter.set_open(true);
}

/// No save open: no dialog.
pub(super) fn clear(ui: &AppWindow) {
    ui.global::<VariableDialogAdapter>().set_open(false);
}
