//! Inventory tab.
//!
//! One selectable row per
//! equipment slot: inventory icon (from the item's ITM → BAM), position name,
//! editable quantity/charges triple, identified item name, and resref, plus a
//! Delete action. Row selection is the Item Browser's assignment target
//! (stored on `Model::inventory_selected`); double-clicking a filled slot
//! reveals its item in the browser. Icons are decoded to Slint `Image`s.

use infinitier_core::resource::itm::{Itm, ItmHeader};
use infinitier_core::resource::tlk::Tlk;
use slint::{Image, SharedString};

use crate::icons;
use crate::state::Model;

/// One equipment slot, in slot order.
pub struct InvRow {
    pub icon: Image,
    pub position: SharedString,
    /// `None` for an empty slot.
    pub item: Option<InvItem>,
}

/// The item in a slot.
pub struct InvItem {
    /// Quantity / charges of the item's three abilities.
    pub quantities: [u16; 3],
    pub name: SharedString,
    pub resref: SharedString,
}

/// The inventory-icon BAM resref (same offset in every ITM header version).
fn icon_resref(itm: &Itm) -> &str {
    match &itm.header {
        ItmHeader::V1(h) => &h.inventory_icon,
        ItmHeader::V1_1(h) => &h.inventory_icon,
        ItmHeader::V2(h) => &h.inventory_icon,
    }
}

/// Load an item's identified name (via `dialog.tlk`) and inventory-icon image.
fn item_display(model: &Model, tlk: Option<&Tlk>, resref: &str) -> (String, Image) {
    let Ok(itm) = model.game_data.import_itm_by_name(resref) else {
        return (String::new(), Image::default());
    };
    let mut strref = itm.header.name_identified_strref();
    if strref == 0 || strref == 0xFFFF_FFFF {
        strref = itm.header.name_strref();
    }
    let name = tlk.and_then(|t| t.get(strref)).unwrap_or_default();
    let icon = icons::bam_icon(&model.game_data, icon_resref(&itm)).unwrap_or_default();
    (name, icon)
}

/// Build the Inventory rows for the selected creature.
pub fn build(model: &Model) -> Vec<InvRow> {
    let Some(imported) = model.selected_imported_cre() else {
        return Vec::new();
    };
    let rows = imported.inventory(model.game_data.game());
    let tlk = model.game_data.dialog_tlk().ok();
    rows.into_iter()
        .map(|r| match r.item {
            Some(item) => {
                let (name, icon) = item_display(model, tlk.as_deref(), &item.item);
                InvRow {
                    icon,
                    position: r.position.into(),
                    item: Some(InvItem {
                        quantities: [item.quantity1, item.quantity2, item.quantity3],
                        name: name.into(),
                        resref: item.item.as_str().into(),
                    }),
                }
            }
            None => InvRow {
                icon: Image::default(),
                position: r.position.into(),
                item: None,
            },
        })
        .collect()
}

/// Record the selected inventory slot.
pub fn select(model: &mut Model, slot: i32) {
    model.inventory_selected = usize::try_from(slot).ok();
}

/// Commit one edited quantity/charges value (`which` = 0/1/2) for `slot`.
pub fn commit_quantity(model: &mut Model, slot: i32, which: i32, value: &str) {
    let (Ok(slot), Ok(which)) = (usize::try_from(slot), usize::try_from(which)) else {
        return;
    };
    if which > 2 {
        return;
    }
    let game = model.game_data.game();
    // Read the current triple (immutable), then write it back (mutable).
    let current = model
        .selected_imported_cre()
        .and_then(|imported| {
            imported
                .inventory(game)
                .into_iter()
                .nth(slot)
                .and_then(|r| r.item)
        })
        .map(|it| [it.quantity1, it.quantity2, it.quantity3]);
    let Some(mut q) = current else {
        return;
    };
    q[which] = value
        .trim()
        .parse::<u32>()
        .unwrap_or(0)
        .min(u16::MAX as u32) as u16;
    if let Some(cre) = model.selected_cre_mut() {
        cre.set_inventory_slot_quantities(slot, q);
    }
}

/// Delete the item in `slot`.
pub fn delete(model: &mut Model, slot: i32) {
    let Ok(slot) = usize::try_from(slot) else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        cre.clear_inventory_slot(slot);
    }
}
