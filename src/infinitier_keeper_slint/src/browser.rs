//! Item & Spell Browser overlays for the Slint keeper.
//!
//! Ports the "add" side of the egui keeper's `ui/item_browser.rs` /
//! `ui/spell_browser.rs`: a searchable list of every ITM / SPL in the game,
//! from which the selected resource is added to the current creature —
//! items into the Inventory tab's selected slot (`Model::inventory_selected`),
//! spells into the character's spellbook. The item-type table + the add glue
//! (`assign_to_inventory` / `add_spell_to_character`) are duplicated from the
//! egui `item_browser.rs` / `app.rs`. NOTE: the AD&D spell add resolves the
//! book/level from the SPL header; the IWD2 (V2.2) add path is not ported.

use infinitier_core::game::GameData;
use infinitier_core::resource::ResourceType;
use infinitier_core::resource::cre::{Item, ItemFlags, SpellType};
use slint::SharedString;

use crate::BrowserRow;
use crate::state::{BrowserEntry, Model};

/// Prefer the identified strref, falling back to the unidentified one.
fn first_strref(identified: u32, unidentified: u32) -> u32 {
    if identified == 0 || identified == 0xFFFF_FFFF {
        unidentified
    } else {
        identified
    }
}

// ── Item index ────────────────────────────────────────────────────────

fn build_item_index(gd: &GameData) -> Vec<BrowserEntry> {
    let tlk = gd.dialog_tlk().ok();
    let mut resrefs: Vec<String> = gd
        .get_all_resources_by_type(ResourceType::Itm)
        .map(|r| r.name.clone())
        .collect();
    resrefs.sort();
    resrefs.dedup();

    let mut out = Vec::with_capacity(resrefs.len());
    for resref in resrefs {
        let Ok(itm) = gd.import_itm_by_name(&resref) else {
            continue;
        };
        let name_ref = first_strref(
            itm.header.name_identified_strref(),
            itm.header.name_strref(),
        );
        let name = tlk
            .as_deref()
            .and_then(|t| t.get(name_ref))
            .unwrap_or_default();
        out.push(BrowserEntry {
            resref,
            name,
            category: item_type_name(itm.header.item_type()).to_string(),
        });
    }
    out.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// Map an ITM category id to a display "Type" (duplicated from the egui
/// item browser).
fn item_type_name(item_type: u16) -> &'static str {
    match item_type {
        1 => "Amulet",
        2 => "Armor",
        3 => "Belt",
        4 => "Boots",
        5 => "Arrow",
        6 => "Bracers",
        7 => "Helmet",
        8 => "Key",
        9 => "Potion",
        10 => "Ring",
        11 => "Scroll",
        12 => "Shield",
        13 => "Food",
        14 => "Bullet",
        15 => "Bow",
        16 => "Dagger",
        17 => "Mace",
        18 => "Sling",
        19 | 20 | 60 | 72 => "Sword",
        21 => "Hammer",
        22 => "Morning Star",
        23 => "Flail",
        24 => "Dart",
        25 => "Axe",
        26 => "Staff",
        27 => "Crossbow",
        28 => "Fist",
        29 => "Spear",
        30 => "Halberd",
        31 => "Bolt",
        32 => "Cloak",
        33 => "Gold",
        34 => "Gem",
        35 => "Wand",
        36 | 61 => "Container",
        37 => "Book",
        38 => "Familiar",
        39 => "Tattoo",
        40 => "Lens",
        41 | 47 => "Buckler",
        42 => "Candle",
        44 => "Club",
        48 => "Large Shield",
        49 => "Medium Shield",
        50 => "Notes",
        57 => "Small Shield",
        58 => "Telescope",
        59 => "Drink",
        63 => "Leather Armor",
        73 => "Scarf",
        0 => "Miscellaneous",
        _ => "Other",
    }
}

// ── Spell index ───────────────────────────────────────────────────────

fn build_spell_index(gd: &GameData) -> Vec<BrowserEntry> {
    let tlk = gd.dialog_tlk().ok();
    let mut resrefs: Vec<String> = gd
        .get_all_resources_by_type(ResourceType::Spl)
        .map(|r| r.name.clone())
        .collect();
    resrefs.sort();
    resrefs.dedup();

    let mut out = Vec::with_capacity(resrefs.len());
    for resref in resrefs {
        let Ok(spl) = gd.import_spl_by_name(&resref) else {
            continue;
        };
        let name = tlk
            .as_deref()
            .and_then(|t| t.get(spl.header.name_strref()))
            .unwrap_or_default();
        out.push(BrowserEntry {
            resref,
            name,
            category: spell_type_label(spl.header.spell_type()).to_string(),
        });
    }
    out.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

fn spell_type_label(spell_type: u16) -> &'static str {
    match spell_type {
        1 => "Wizard",
        2 => "Priest",
        _ => "Innate",
    }
}

// ── Load dialog ───────────────────────────────────────────────────────

/// Rows for the Load dialog: every discovered save game (filtered by the
/// query), with `resref` carrying the save's discovery index as a string.
pub fn load_rows(model: &Model) -> Vec<BrowserRow> {
    let saves = model.game_data.save_games();
    let q = model.load_query.trim().to_lowercase();
    (0..saves.len())
        .filter_map(|i| saves.by_index(i).map(|s| (i, s.folder_name().to_string())))
        .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
        .map(|(i, name)| BrowserRow {
            resref: i.to_string().into(),
            name: name.into(),
            category: String::new().into(),
            selected: model.load_selected == Some(i),
        })
        .collect()
}

// ── Shared filtering ──────────────────────────────────────────────────

fn filter_rows(index: &[BrowserEntry], query: &str, selected: &Option<String>) -> Vec<BrowserRow> {
    let q = query.trim().to_lowercase();
    index
        .iter()
        .filter(|e| {
            q.is_empty()
                || e.name.to_lowercase().contains(&q)
                || e.resref.to_lowercase().contains(&q)
                || e.category.to_lowercase().contains(&q)
        })
        .map(|e| BrowserRow {
            resref: e.resref.as_str().into(),
            name: e.name.as_str().into(),
            category: e.category.as_str().into(),
            selected: selected.as_deref() == Some(e.resref.as_str()),
        })
        .collect()
}

// ── Item browser ──────────────────────────────────────────────────────

/// Toggle the Item Browser open/closed (building its index on first open).
pub fn toggle_items(model: &mut Model) {
    model.item_browser_open = !model.item_browser_open;
    if model.item_browser_open && model.item_index.is_empty() {
        model.item_index = build_item_index(&model.game_data);
    }
}

pub fn close_items(model: &mut Model) {
    model.item_browser_open = false;
}

pub fn search_items(model: &mut Model, query: &str) {
    model.item_query = query.to_string();
}

pub fn select_item(model: &mut Model, resref: &str) {
    model.item_selected = Some(resref.to_string());
}

pub fn item_rows(model: &Model) -> Vec<BrowserRow> {
    filter_rows(&model.item_index, &model.item_query, &model.item_selected)
}

/// Whether the "Add to inventory" button is enabled: a slot is selected on
/// the Inventory tab and an item is selected in the browser.
pub fn item_add_enabled(model: &Model) -> bool {
    model.inventory_selected.is_some() && model.item_selected.is_some()
}

pub fn item_add_label(model: &Model) -> SharedString {
    match model.character_name() {
        name if !name.is_empty() => format!("Add to {name}'s inventory").into(),
        _ => "Add to inventory".into(),
    }
}

/// Add the selected item into the selected inventory slot (a single
/// identified copy, charges/stack from the ITM).
pub fn add_item(model: &mut Model) {
    let Some(slot) = model.inventory_selected else {
        return;
    };
    let Some(resref) = model.item_selected.clone() else {
        return;
    };
    let [quantity1, quantity2, quantity3] = model
        .game_data
        .import_itm_by_name(&resref)
        .map(|itm| itm.max_charges())
        .unwrap_or([0, 0, 0]);
    let item = Item {
        item: resref,
        duration: 0,
        quantity1,
        quantity2,
        quantity3,
        flags: ItemFlags::Identified,
    };
    if let Some(cre) = model.selected_cre_mut() {
        cre.set_inventory_slot_item(slot, item);
    }
}

// ── Spell browser ─────────────────────────────────────────────────────

pub fn toggle_spells(model: &mut Model) {
    model.spell_browser_open = !model.spell_browser_open;
    if model.spell_browser_open && model.spell_index.is_empty() {
        model.spell_index = build_spell_index(&model.game_data);
    }
}

pub fn close_spells(model: &mut Model) {
    model.spell_browser_open = false;
}

pub fn search_spells(model: &mut Model, query: &str) {
    model.spell_query = query.to_string();
}

pub fn select_spell(model: &mut Model, resref: &str) {
    model.spell_selected = Some(resref.to_string());
}

pub fn spell_rows(model: &Model) -> Vec<BrowserRow> {
    filter_rows(
        &model.spell_index,
        &model.spell_query,
        &model.spell_selected,
    )
}

pub fn spell_add_enabled(model: &Model) -> bool {
    model.selected_cre().is_some() && model.spell_selected.is_some()
}

pub fn spell_add_label(model: &Model) -> SharedString {
    match model.character_name() {
        name if !name.is_empty() => format!("Add to {name}").into(),
        _ => "Add to character".into(),
    }
}

/// Add the selected spell to the character as a fresh, un-memorised entry.
/// AD&D only: the spellbook/level come from the SPL header. (IWD2's per-class
/// list-2DA add path is not ported.)
pub fn add_spell(model: &mut Model) {
    let Some(resref) = model.spell_selected.clone() else {
        return;
    };
    let Some((spell_type, level)) = model.game_data.import_spl_by_name(&resref).ok().map(|spl| {
        let spell_type = match spl.header.spell_type() {
            1 => SpellType::Wizard,
            2 => SpellType::Priest,
            _ => SpellType::Innate,
        };
        // SPL levels are 1-based; known spells store them 0-based.
        let level = (spl.header.spell_level() as u16).saturating_sub(1);
        (spell_type, level)
    }) else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        cre.add_known_spell(spell_type, level, &resref);
    }
}
