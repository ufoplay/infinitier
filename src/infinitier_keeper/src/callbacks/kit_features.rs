//! Callbacks of the Kit Features dialog (`KitFeaturesAdapter`).

use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::tabs::characteristics::kit_features;
use crate::{AppWindow, KitFeaturesAdapter};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    let adapter = ui.global::<KitFeaturesAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_select_table(move |index| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| kit_features::select_table(m, index));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_select_part(move |index| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| kit_features::select_part(m, index));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_set_from(move |value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| kit_features::set_from(m, value));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_set_to(move |value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| kit_features::set_to(m, value));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, kit_features::add);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_remove(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, kit_features::remove);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, kit_features::close);
        });
    }
}
