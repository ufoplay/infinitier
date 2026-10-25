//! The Item Browser's index and its add-to-inventory glue. The item-type
//! table and `add` (the egui keeper's `assign_to_inventory`) are duplicated
//! from the egui keeper's `ui/item_browser.rs` / `game_view.rs`.

use infinitier_core::game::GameData;
use infinitier_core::resource::ResourceType;
use infinitier_core::resource::cre::{Item, ItemFlags};

use super::{Entry, first_strref};
use crate::state::{CharacterTab, Model};

/// Build the item index: parse every `.itm`, resolve its identified name and
/// description strref. Sorted by type, then name.
pub(super) fn build_index(game_data: &GameData) -> Vec<Entry> {
    let tlk = game_data.dialog_tlk().ok();
    // Collect the resrefs first so the borrow of the resource index ends
    // before each item is imported.
    let resrefs = {
        let mut resrefs: Vec<String> = game_data
            .get_all_resources_by_type(ResourceType::Itm)
            .map(|r| r.name.clone())
            .collect();
        resrefs.sort();
        resrefs.dedup();
        resrefs
    };

    let mut out: Vec<Entry> = resrefs
        .into_iter()
        .filter_map(|resref| {
            let itm = game_data.import_itm_by_name(&resref).ok()?;
            let name_ref = first_strref(
                itm.header.name_identified_strref(),
                itm.header.name_strref(),
            );
            Some(Entry {
                name: tlk
                    .as_deref()
                    .and_then(|t| t.get(name_ref))
                    .unwrap_or_default(),
                type_name: item_type_name(itm.header.item_type()),
                level: None,
                script_name: String::new(),
                icon: itm.header.inventory_icon().to_owned(),
                description_strref: first_strref(
                    itm.header.description_identified_strref(),
                    itm.header.description_strref(),
                ),
                resref,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.type_name
            .cmp(b.type_name)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

/// The inventory slot the Item Browser adds into: the Inventory tab shows an
/// embedded creature and one of its slots is selected.
pub(super) fn add_target(model: &Model) -> Option<usize> {
    (model.selected_tab == CharacterTab::Inventory && model.selected_cre().is_some())
        .then_some(model.inventory_selected)
        .flatten()
}

/// The Add button's caption, naming the selected character.
pub(super) fn add_label(model: Option<&Model>) -> String {
    match model.map(Model::character_name) {
        Some(name) if !name.is_empty() => format!("Add to {name} inventory"),
        Some(_) | None => "Add to inventory".to_owned(),
    }
}

/// Put the item `resref` into inventory `slot` of the selected character,
/// as a single identified copy whose charges/stack the ITM dictates.
pub(super) fn add(model: &mut Model, slot: usize, resref: &str) {
    let [quantity1, quantity2, quantity3] = model
        .game_data
        .import_itm_by_name(resref)
        .map(|itm| itm.max_charges())
        .unwrap_or([0, 0, 0]);
    let item = Item {
        item: resref.to_owned(),
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

/// Map an ITM category id to a display "Type".
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
