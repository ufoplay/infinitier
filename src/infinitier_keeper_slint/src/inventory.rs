//! Inventory tab for the Slint keeper.
//!
//! Ports the egui keeper's `ui/tabs/inventory/`. One selectable row per
//! equipment slot: inventory icon (from the item's ITM → BAM), position name,
//! editable quantity/charges triple, identified item name, and resref, plus a
//! Delete action. Row selection is the Item Browser's assignment target
//! (stored on `Model::inventory_selected`, ready for the browser). The
//! item/icon resolution is duplicated from the egui view; icons are decoded
//! to Slint `Image`s. (The Item Browser "add item" flow is not yet ported.)

use infinitier_core::imported_resource::ImportedResource;
use infinitier_core::resource::ResourceType;
use infinitier_core::resource::itm::{Itm, ItmHeader};
use infinitier_core::resource::tlk::Tlk;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

use crate::InvRow;
use crate::state::Model;

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
    let icon = load_icon(model, icon_resref(&itm)).unwrap_or_default();
    (name, icon)
}

/// Decode an inventory-icon BAM's first frame into a Slint `Image`.
fn load_icon(model: &Model, icon: &str) -> Option<Image> {
    if icon.is_empty() {
        return None;
    }
    let imported = model
        .game_data
        .import_by_name_and_type(icon, ResourceType::Bam)
        .ok()?;
    let ImportedResource::Bam(bam) = imported.as_ref() else {
        return None;
    };
    let frame = bam.render_frame_centered(0, 0)?;
    let (w, h) = (frame.width(), frame.height());
    if w == 0 || h == 0 {
        return None;
    }
    let raw = frame.into_raw();
    let mut pb = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    pb.make_mut_bytes().copy_from_slice(&raw);
    Some(Image::from_rgba8(pb))
}

/// Build the Inventory rows for the selected creature.
pub fn build(model: &Model) -> Vec<InvRow> {
    let Some(imported) = model.selected_imported_cre() else {
        return Vec::new();
    };
    let rows = imported.inventory(model.game_data.game());
    let tlk = model.game_data.dialog_tlk().ok();
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| match r.item {
            Some(item) => {
                let (name, icon) = item_display(model, tlk.as_deref(), &item.item);
                InvRow {
                    slot: i as i32,
                    icon,
                    position: r.position.into(),
                    filled: true,
                    q1: item.quantity1.to_string().into(),
                    q2: item.quantity2.to_string().into(),
                    q3: item.quantity3.to_string().into(),
                    name: name.into(),
                    resref: item.item.as_str().into(),
                    selected: model.inventory_selected == Some(i),
                }
            }
            None => InvRow {
                slot: i as i32,
                icon: Image::default(),
                position: r.position.into(),
                filled: false,
                q1: Default::default(),
                q2: Default::default(),
                q3: Default::default(),
                name: Default::default(),
                resref: Default::default(),
                selected: model.inventory_selected == Some(i),
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
