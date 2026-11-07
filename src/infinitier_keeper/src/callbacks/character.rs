//! Callbacks of the party list, the character tab strip and the tab pages'
//! editable content.

use infinitier_core::imported_resource::plt::Equipment;
use infinitier_core::portraits::PortraitSlot;
use slint::ComponentHandle;

use super::{AppRc, on_model};
use crate::components::edit_table::EditTable;
use crate::state::{self, Model};
use crate::tabs::effects::{self, editor};
use crate::tabs::{
    CharacterTab, abilities, appearance, characteristics, feats, inventory, levels, memorization,
    proficiencies, resistances, spells,
};
use crate::view;
use crate::{
    AppWindow, AppearanceAdapter, CardsAdapter, CharacterAdapter, EditTableAdapter,
    EffectDialogAdapter, EffectsAdapter, InventoryAdapter, PartyAdapter, SpellsAdapter,
};

pub fn wire(ui: &AppWindow, app: &AppRc) {
    wire_party(ui, app);
    wire_cards(ui, app);
    wire_edit_table(ui, app);
    wire_appearance(ui, app);
    wire_inventory(ui, app);
    wire_spells(ui, app);
    wire_effects(ui, app);
}

/// Appearance: pick the animation, a colour or a portrait, show or hide the
/// paperdoll's equipment.
fn wire_appearance(ui: &AppWindow, app: &AppRc) {
    let adapter = ui.global::<AppearanceAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_select_animation(move |index| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| appearance::select_animation(m, index));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_pick_color(move |slot, gradient| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| appearance::pick_color(m, slot, gradient));
        });
    }
    let select_portrait = |slot| {
        let (app, weak) = (app.clone(), ui.as_weak());
        move |index| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| appearance::select_portrait(m, slot, index));
        }
    };
    adapter.on_select_small_portrait(select_portrait(PortraitSlot::Small));
    adapter.on_select_large_portrait(select_portrait(PortraitSlot::Large));
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        adapter.on_toggle_equipment(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                m.doll_equipment = match m.doll_equipment {
                    Equipment::Shown => Equipment::Hidden,
                    Equipment::Hidden => Equipment::Shown,
                };
            });
        });
    }
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
                m.effect_editor = None; // edits the previous one's effects
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
                    CharacterTab::Levels => levels::commit(m, id, value),
                    CharacterTab::Feats => feats::commit(m, id, value),
                    CharacterTab::Characteristics => characteristics::commit(m, id, value),
                    // Read-only cards, or not a card tab.
                    CharacterTab::Miscellaneous
                    | CharacterTab::Memorization
                    | CharacterTab::Proficiencies
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

/// The Memorization / Proficiencies table: commit an edited cell (clamp +
/// write + re-read).
fn wire_edit_table(ui: &AppWindow, app: &AppRc) {
    let (app, weak) = (app.clone(), ui.as_weak());
    ui.global::<EditTableAdapter>()
        .on_commit(move |row, column, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                // The active tab's table, as shown, says which cell was edited.
                let (table, commit): (EditTable, fn(&mut Model, i32, &str)) = match m.selected_tab {
                    CharacterTab::Memorization => (memorization::build(m), memorization::commit),
                    CharacterTab::Proficiencies => (proficiencies::build(m), proficiencies::commit),
                    CharacterTab::Abilities
                    | CharacterTab::Levels
                    | CharacterTab::Characteristics
                    | CharacterTab::Appearance
                    | CharacterTab::Inventory
                    | CharacterTab::Spells
                    | CharacterTab::Feats
                    | CharacterTab::Resistances
                    | CharacterTab::Effects
                    | CharacterTab::LocalVariables
                    | CharacterTab::GlobalVariables
                    | CharacterTab::JournalEntries
                    | CharacterTab::Miscellaneous => return,
                };
                let (Ok(row), Ok(column)) = (usize::try_from(row), usize::try_from(column)) else {
                    return;
                };
                if let Some(id) = table.cell_id(row, column) {
                    commit(m, id, value.as_str());
                }
            });
        });
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
        spells.on_memorize(move |row, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| {
                spells::commit_memorized(m, row, value.as_str());
            });
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        spells.on_delete(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| spells::delete(m, row));
        });
    }
}

/// Effects: add, edit, clone or delete an effect; the effect dialog's
/// fields, Save and Cancel.
fn wire_effects(ui: &AppWindow, app: &AppRc) {
    let tab = ui.global::<EffectsAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, effects::add);
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_edit(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| effects::edit(m, row));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_duplicate(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| effects::duplicate(m, row));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        tab.on_delete(move |row| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| effects::delete(m, row));
        });
    }
    let dialog = ui.global::<EffectDialogAdapter>();
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        dialog.on_commit_field(move |id, value| {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, |m| editor::commit(m, id, value.as_str()));
        });
    }
    {
        let (app, weak) = (app.clone(), ui.as_weak());
        dialog.on_save(move || {
            let Some(ui) = weak.upgrade() else { return };
            on_model(&ui, &app, editor::save);
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
