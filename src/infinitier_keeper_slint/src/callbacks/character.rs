//! Callbacks of the party list, the character tab strip and the tab pages'
//! editable content.

use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::state;
use crate::tabs::{
    CharacterTab, abilities, feats, inventory, levels, memorization, proficiencies, resistances,
    spells,
};
use crate::view;
use crate::{
    AppWindow, CardsAdapter, CharacterAdapter, InventoryAdapter, PartyAdapter, SpellsAdapter,
};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    wire_party(ui, app);
    wire_cards(ui, app);
    wire_inventory(ui, app);
    wire_spells(ui, app);
}

/// Select a party member / a character tab → re-read the tab's content.
fn wire_party(ui: &AppWindow, app: &AppRc) {
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.global::<PartyAdapter>().on_select(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                m.selected = usize::try_from(idx).ok();
                m.inventory_selected = None; // selection is per-creature
                m.spell_tab = state::SPELL_TAB_AUTO; // re-default the spellbook
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        ui.global::<CharacterAdapter>().on_select_tab(move |idx| {
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
}

/// The card tabs: commit an edited field (clamp + write + re-read) or the
/// Attacks dropdown.
fn wire_cards(ui: &AppWindow, app: &AppRc) {
    let cards = ui.global::<CardsAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        cards.on_commit_field(move |id, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                // Field ids are tab-local: the active tab commits.
                let value = value.as_str();
                match m.selected_tab {
                    CharacterTab::Abilities => abilities::commit(m, id, value),
                    CharacterTab::Resistances => resistances::commit(m, id, value),
                    CharacterTab::Proficiencies => proficiencies::commit(m, id, value),
                    CharacterTab::Memorization => memorization::commit(m, id, value),
                    CharacterTab::Levels => levels::commit(m, id, value),
                    CharacterTab::Feats => feats::commit(m, id, value),
                    // Read-only cards, or not a card tab.
                    CharacterTab::Characteristics
                    | CharacterTab::Miscellaneous
                    | CharacterTab::Appearance
                    | CharacterTab::Inventory
                    | CharacterTab::Spells
                    | CharacterTab::Effects
                    | CharacterTab::LocalVariables
                    | CharacterTab::GlobalVariables
                    | CharacterTab::JournalEntries => {}
                }
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        cards.on_commit_attacks(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| abilities::commit_attacks(m, idx));
        });
    }
}

/// Inventory: select a slot / edit a quantity / delete an item.
fn wire_inventory(ui: &AppWindow, app: &AppRc) {
    let inventory = ui.global::<InventoryAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        inventory.on_select(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            // The page already highlights the slot; only the Item Browser's
            // add target changes. Not rebuilding the rows keeps the slot's
            // double-click (reveal in the browser) registering.
            let mut app = app.borrow_mut();
            if let Some(model) = app.model_mut() {
                inventory::select(model, slot);
            }
            view::refresh_browsers(&ui, &mut app);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        inventory.on_quantity(move |slot, which, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                inventory::commit_quantity(m, slot, which, value.as_str());
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        inventory.on_delete(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| inventory::delete(m, slot));
        });
    }
}

/// Spells: switch spellbook / edit a memorised count / delete a spell.
fn wire_spells(ui: &AppWindow, app: &AppRc) {
    let spells = ui.global::<SpellsAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        spells.on_select_book(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| spells::select_tab(m, idx));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        spells.on_memorize(move |ref_id, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                spells::commit_memorized(m, ref_id, value.as_str());
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        spells.on_delete(move |ref_id| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| spells::delete(m, ref_id));
        });
    }
}
