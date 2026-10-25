//! The selected character: the tab strip (`CharacterAdapter`) and the
//! active tab's page, each page's adapter filled by its tab module.

use slint::{ComponentHandle, SharedString};

use super::{index_or_none, rows_model};
use crate::components::editable_fields::AttacksOption;
use crate::state::Model;
use crate::tabs::{
    CharacterTab, abilities, appearance, characteristics, effects, feats, inventory, journal,
    levels, memorization, miscellaneous, proficiencies, resistances, spells, variables,
};
use crate::{
    AppWindow, AppearanceAdapter, CardsAdapter, CharacterAdapter, CharacterContent, ColorSwatch,
    FieldCard, GridAdapter, GridRow, InvRow, InventoryAdapter, SpellRowUi, SpellsAdapter,
    TableAdapter, VarRow,
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
        adapter.set_content(CharacterContent::None);
        return;
    }

    match model.selected_tab {
        CharacterTab::Abilities => show_cards(ui, model, abilities::build(model)),
        CharacterTab::Levels => show_cards(ui, model, levels::build(model)),
        CharacterTab::Characteristics => show_cards(ui, model, characteristics::build(model)),
        CharacterTab::Feats => show_cards(ui, model, feats::build(model)),
        CharacterTab::Memorization => show_cards(ui, model, memorization::build(model)),
        CharacterTab::Proficiencies => show_cards(ui, model, proficiencies::build(model)),
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
        ),
        CharacterTab::JournalEntries => show_grid(
            ui,
            journal::HEADERS,
            journal::WIDTHS,
            journal::build(model),
            GridText::Wrap,
        ),
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
fn show_cards(ui: &AppWindow, model: &Model, cards: [Vec<FieldCard>; 3]) {
    let adapter = ui.global::<CardsAdapter>();
    let [column0, column1, column2] = cards;
    adapter.set_column0(rows_model(column0));
    adapter.set_column1(rows_model(column1));
    adapter.set_column2(rows_model(column2));
    let attacks_index = model
        .selected_cre()
        .and_then(|cre| AttacksOption::index_for_byte(cre.attacks_byte()))
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(-1);
    adapter.set_attacks_index(attacks_index);
    show(ui, CharacterContent::Cards);
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

/// The wide read-only grid: headers, per-column widths and rows.
fn show_grid(ui: &AppWindow, headers: &[&str], widths: &[f32], rows: Vec<GridRow>, text: GridText) {
    let adapter = ui.global::<GridAdapter>();
    let headers: Vec<SharedString> = headers.iter().map(|h| (*h).into()).collect();
    adapter.set_headers(rows_model(headers));
    adapter.set_widths(rows_model(widths.to_vec()));
    adapter.set_rows(rows_model(rows));
    adapter.set_wrap(match text {
        GridText::Elide => false,
        GridText::Wrap => true,
    });
    show(ui, CharacterContent::Grid);
}

/// The animation name and the colour swatches.
fn show_appearance(ui: &AppWindow, name: SharedString, swatches: Vec<ColorSwatch>) {
    let adapter = ui.global::<AppearanceAdapter>();
    adapter.set_animation_name(name);
    adapter.set_swatches(rows_model(swatches));
    show(ui, CharacterContent::Appearance);
}

/// The slot rows, `selected` highlighted.
fn show_inventory(ui: &AppWindow, rows: Vec<InvRow>, selected: Option<usize>) {
    let adapter = ui.global::<InventoryAdapter>();
    adapter.set_rows(rows_model(rows));
    adapter.set_selected(index_or_none(selected));
    show(ui, CharacterContent::Inventory);
}

/// The spellbook strip and the selected book's spells.
fn show_spells(ui: &AppWindow, books: Vec<SharedString>, book: i32, rows: Vec<SpellRowUi>) {
    let adapter = ui.global::<SpellsAdapter>();
    adapter.set_books(rows_model(books));
    adapter.set_book(book);
    adapter.set_rows(rows_model(rows));
    show(ui, CharacterContent::Spells);
}

fn show(ui: &AppWindow, content: CharacterContent) {
    ui.global::<CharacterAdapter>().set_content(content);
}
