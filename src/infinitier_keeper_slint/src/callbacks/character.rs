//! Callbacks of the party list, the character tab strip and the tabs'
//! editable content.

use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::state::{self, CharacterTab};
use crate::{
    AppWindow, abilities, feats, inventory, levels, memorization, proficiencies, resistances,
    spells,
};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    // Select a party member → re-read its tab content.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_select_party(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                m.selected = usize::try_from(idx).ok();
                m.inventory_selected = None; // selection is per-creature
                m.spell_tab = state::SPELL_TAB_AUTO; // re-default inner spell tab
            });
        });
    }

    // Switch the active character sub-tab.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_select_tab(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                let tab = usize::try_from(idx)
                    .ok()
                    .and_then(|i| m.visible_tabs().get(i).copied());
                if let Some(tab) = tab {
                    m.selected_tab = tab;
                }
            });
        });
    }

    // Commit an edited field → clamp + write + re-read.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_commit_field(move |id, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                // Field ids are tab-local; dispatch to the active tab's committer.
                let value = value.as_str();
                match m.selected_tab {
                    CharacterTab::Abilities => abilities::commit(m, id, value),
                    CharacterTab::Resistances => resistances::commit(m, id, value),
                    CharacterTab::Proficiencies => proficiencies::commit(m, id, value),
                    CharacterTab::Memorization => memorization::commit(m, id, value),
                    CharacterTab::Levels => levels::commit(m, id, value),
                    CharacterTab::Feats => feats::commit(m, id, value),
                    _ => {}
                }
            });
        });
    }

    // Commit the Attacks dropdown selection.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_commit_attacks(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| abilities::commit_attacks(m, idx));
        });
    }

    // Inventory: select a slot / edit a quantity / delete an item.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_inv_select(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| inventory::select(m, slot));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_inv_quantity(move |slot, which, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                inventory::commit_quantity(m, slot, which, value.as_str());
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_inv_delete(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| inventory::delete(m, slot));
        });
    }

    // Spells: switch inner tab / edit memorised count / delete a spell.
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_select_tab(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| spells::select_tab(m, idx));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_memorize(move |ref_id, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                spells::commit_memorized(m, ref_id, value.as_str());
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.on_spell_delete(move |ref_id| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| spells::delete(m, ref_id));
        });
    }
}
