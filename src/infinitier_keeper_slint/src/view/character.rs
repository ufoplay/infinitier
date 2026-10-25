//! The selected character: the tab strip (`CharacterAdapter`) and the
//! active tab's page, each page's adapter filled by its tab module.

use infinitier_core::imported_resource::gam::NpcCre;
use slint::{ComponentHandle, SharedString};

use super::{index_or_none, rows_model, sync_nested, sync_rows};
use crate::components::edit_table::EditTable;
use crate::components::editable_fields::AttacksOption;
use crate::state::Model;
use crate::tabs::{
    CharacterTab, abilities, appearance, characteristics, effects, feats, inventory, journal,
    levels, memorization, miscellaneous, proficiencies, resistances, spells, variables,
};
use crate::{
    AppWindow, AppearanceAdapter, CardsAdapter, CharacterAdapter, CharacterContent, ColorSwatch,
    EditTableAdapter, FieldCard, GridAdapter, GridRow, InvRow, InventoryAdapter, SpellRowUi,
    SpellsAdapter, TableAdapter, VarRow,
};

/// Repaint the tab strip and the active tab's page. Takes `&mut Model`
/// because the Spells tab records its rows' spells.
pub(super) fn refresh(ui: &AppWindow, model: &mut Model) {
    let adapter = ui.global::<CharacterAdapter>();
    let visible_tabs = model.visible_tabs();
    let tab_names: Vec<SharedString> = visible_tabs.iter().map(|t| t.label().into()).collect();
    adapter.set_tab_names(rows_model(tab_names));
    let tab_index = visible_tabs.iter().position(|t| *t == model.selected_tab);
    adapter.set_selected_tab(tab_index.and_then(|i| i32::try_from(i).ok()).unwrap_or(0));
    let has_character = model.selected_cre().is_some();
    adapter.set_has_character(has_character);
    if !has_character {
        adapter.set_message(no_character_message(model).into());
        adapter.set_content(CharacterContent::None);
        return;
    }

    match model.selected_tab {
        CharacterTab::Abilities => show_cards(ui, model, abilities::build(model)),
        CharacterTab::Levels => match levels::build(model) {
            Some(cards) => show_cards(ui, model, cards),
            None => show_unavailable(ui, levels::UNAVAILABLE),
        },
        CharacterTab::Characteristics => show_cards(ui, model, characteristics::build(model)),
        CharacterTab::Feats => match feats::build(model) {
            Some(cards) => show_cards(ui, model, cards),
            None => show_unavailable(ui, feats::UNAVAILABLE),
        },
        CharacterTab::Memorization => show_edit_table(ui, memorization::build(model)),
        CharacterTab::Proficiencies => show_edit_table(ui, proficiencies::build(model)),
        CharacterTab::Resistances => show_cards(ui, model, resistances::build(model)),
        CharacterTab::Miscellaneous => show_cards(ui, model, miscellaneous::build(model)),
        CharacterTab::Appearance => {
            let (name, swatches) = appearance::build(model);
            show_appearance(ui, name, swatches);
        }
        CharacterTab::Inventory => {
            show_inventory(ui, inventory::build(model), model.inventory_selected)
        }
        CharacterTab::Spells => {
            let (books, book, rows) = spells::build(model);
            show_spells(ui, books, book, rows);
        }
        CharacterTab::LocalVariables => show_table(ui, variables::local(model)),
        CharacterTab::GlobalVariables => show_table(ui, variables::global(model)),
        CharacterTab::Effects => show_grid(
            ui,
            effects::HEADERS,
            effects::WIDTHS,
            effects::build(model),
            GridText::Elide,
            "",
        ),
        CharacterTab::JournalEntries => show_grid(
            ui,
            journal::HEADERS,
            journal::WIDTHS,
            journal::build(model),
            GridText::Wrap,
            "This save has no journal entries.",
        ),
    }
}

/// Why the selected party slot has no character to show, worded as the egui
/// keeper's.
fn no_character_message(model: &Model) -> String {
    let Some(index) = model.selected else {
        return "Select a party member on the left to view their data.".to_owned();
    };
    match model.gam.party_npcs.get(index).map(|npc| npc.cre.as_ref()) {
        None => "Stale selection — party member not found.".to_owned(),
        Some(None) => "Empty party slot — no creature record to edit.".to_owned(),
        Some(Some(NpcCre::Ref(resref))) => {
            format!("External CRE '{resref}' — embedded record not present in this GAM.")
        }
        // Has a record: `refresh` shows it.
        Some(Some(NpcCre::Cre(_))) => String::new(),
    }
}

/// No save open: no tabs and no page.
pub(super) fn clear(ui: &AppWindow) {
    let adapter = ui.global::<CharacterAdapter>();
    adapter.set_tab_names(rows_model(Vec::new()));
    adapter.set_has_character(false);
    adapter.set_content(CharacterContent::None);
}

/// The card grid, with the Attacks dropdown's current option (the Abilities
/// tab's Combat card).
/// Synced in place (see [`sync_rows`]), so a field being edited survives.
fn show_cards(ui: &AppWindow, model: &Model, cards: [Vec<FieldCard>; 3]) {
    let adapter = ui.global::<CardsAdapter>();
    let [column0, column1, column2] = cards;
    if let Some(rows) = sync_nested(&adapter.get_column0(), column0) {
        adapter.set_column0(rows);
    }
    if let Some(rows) = sync_nested(&adapter.get_column1(), column1) {
        adapter.set_column1(rows);
    }
    if let Some(rows) = sync_nested(&adapter.get_column2(), column2) {
        adapter.set_column2(rows);
    }
    adapter.set_message(SharedString::new());
    let attacks_index = model
        .selected_cre()
        .and_then(|cre| AttacksOption::index_for_byte(cre.attacks_byte()))
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(-1);
    adapter.set_attacks_index(attacks_index);
    show(ui, CharacterContent::Cards);
}

/// A card tab that has no cards for this creature: why, instead.
fn show_unavailable(ui: &AppWindow, message: &str) {
    let adapter = ui.global::<CardsAdapter>();
    adapter.set_column0(rows_model(Vec::new()));
    adapter.set_column1(rows_model(Vec::new()));
    adapter.set_column2(rows_model(Vec::new()));
    adapter.set_message(message.into());
    show(ui, CharacterContent::Cards);
}

/// The Memorization / Proficiencies table, synced in place (see
/// [`sync_rows`]).
fn show_edit_table(ui: &AppWindow, table: EditTable) {
    let EditTable {
        name_header,
        headers,
        rows,
    } = table;
    let adapter = ui.global::<EditTableAdapter>();
    adapter.set_name_header(name_header.into());
    let headers: Vec<SharedString> = headers.iter().map(|h| (*h).into()).collect();
    adapter.set_headers(rows_model(headers));
    if let Some(rows) = sync_nested(&adapter.get_rows(), rows) {
        adapter.set_rows(rows);
    }
    show(ui, CharacterContent::EditTable);
}

/// The read-only Name · Value table.
fn show_table(ui: &AppWindow, rows: Vec<VarRow>) {
    ui.global::<TableAdapter>().set_rows(rows_model(rows));
    show(ui, CharacterContent::Table);
}

/// How the grid fits a cell's text in its column.
enum GridText {
    Elide,
    /// Onto more lines, the row growing to fit (full journal entries).
    Wrap,
}

/// The wide read-only grid: headers, per-column widths and rows, or
/// `empty_text` when there are none.
fn show_grid(
    ui: &AppWindow,
    headers: &[&str],
    widths: &[f32],
    rows: Vec<GridRow>,
    text: GridText,
    empty_text: &str,
) {
    let adapter = ui.global::<GridAdapter>();
    let headers: Vec<SharedString> = headers.iter().map(|h| (*h).into()).collect();
    adapter.set_headers(rows_model(headers));
    adapter.set_widths(rows_model(widths.to_vec()));
    adapter.set_columns_width(widths.iter().sum());
    adapter.set_rows(rows_model(rows));
    adapter.set_wrap(match text {
        GridText::Elide => false,
        GridText::Wrap => true,
    });
    adapter.set_empty_text(empty_text.into());
    show(ui, CharacterContent::Grid);
}

/// The animation name and the colour swatches.
fn show_appearance(ui: &AppWindow, name: SharedString, swatches: Vec<ColorSwatch>) {
    let adapter = ui.global::<AppearanceAdapter>();
    adapter.set_animation_name(name);
    adapter.set_swatches(rows_model(swatches));
    show(ui, CharacterContent::Appearance);
}

/// The slot rows, `selected` highlighted; synced in place (see
/// [`sync_rows`]).
fn show_inventory(ui: &AppWindow, rows: Vec<InvRow>, selected: Option<usize>) {
    let adapter = ui.global::<InventoryAdapter>();
    if let Some(rows) = sync_rows(&adapter.get_rows(), rows) {
        adapter.set_rows(rows);
    }
    adapter.set_selected(index_or_none(selected));
    show(ui, CharacterContent::Inventory);
}

/// The spellbook strip and the selected book's spells; synced in place (see
/// [`sync_rows`]).
fn show_spells(ui: &AppWindow, books: Vec<SharedString>, book: i32, rows: Vec<SpellRowUi>) {
    let adapter = ui.global::<SpellsAdapter>();
    if let Some(books) = sync_rows(&adapter.get_books(), books) {
        adapter.set_books(books);
    }
    adapter.set_book(book);
    if let Some(rows) = sync_rows(&adapter.get_rows(), rows) {
        adapter.set_rows(rows);
    }
    show(ui, CharacterContent::Spells);
}

fn show(ui: &AppWindow, content: CharacterContent) {
    ui.global::<CharacterAdapter>().set_content(content);
}
