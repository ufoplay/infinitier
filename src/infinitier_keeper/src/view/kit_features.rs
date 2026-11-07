//! The Kit Features dialog (`KitFeaturesAdapter`), from the selected
//! character's open [`KitFeatures`].

use infinitier_core::class_abilities::Grant;
use slint::{ComponentHandle, SharedString};

use super::{rows_changed, rows_model, sync_table};
use crate::components::table;
use crate::state::Model;
use crate::tabs::characteristics::KitFeatures;
use crate::tabs::characteristics::kit_features::{MAX_LEVEL, Part};
use crate::{AppWindow, KitFeaturesAdapter};

pub(super) fn refresh(ui: &AppWindow, model: &Model) {
    let adapter = ui.global::<KitFeaturesAdapter>();
    let Some(features) = &model.kit_features else {
        adapter.set_open(false);
        return;
    };
    show(&adapter, features);
    adapter.set_open(true);
}

/// No save open: no dialog.
pub(super) fn clear(ui: &AppWindow) {
    ui.global::<KitFeaturesAdapter>().set_open(false);
}

fn show(adapter: &KitFeaturesAdapter, features: &KitFeatures) {
    let names: Vec<SharedString> = features.names().map(Into::into).collect();
    // A long list, kept unless it changes (see `rows_changed`).
    if rows_changed(&adapter.get_tables(), &names) {
        adapter.set_tables(rows_model(names));
    }
    adapter.set_table_index(index(features.table()));
    let parts: Vec<SharedString> = Part::ALL.iter().map(|part| part.label().into()).collect();
    if rows_changed(&adapter.get_parts(), &parts) {
        adapter.set_parts(rows_model(parts));
    }
    let part = Part::ALL.iter().position(|&part| part == features.part());
    adapter.set_part_index(part.map_or(0, index));
    let (from, to) = features.levels();
    adapter.set_from_level(level(from));
    adapter.set_to_level(level(to));
    adapter.set_max_level(level(MAX_LEVEL));
    let rows = features
        .preview()
        .iter()
        .map(|line| {
            let grant = match line.grant {
                Grant::Effects => "Effects",
                Grant::Innate => "Innate",
            };
            vec![
                table::text(line.level.to_string()),
                table::text(grant),
                table::text(line.spell.as_str()),
            ]
        })
        .collect();
    if let Some(rows) = sync_table(&adapter.get_rows(), rows) {
        adapter.set_rows(rows);
    }
}

fn index(index: usize) -> i32 {
    i32::try_from(index).unwrap_or(-1)
}

fn level(level: u32) -> i32 {
    i32::try_from(level).unwrap_or(i32::MAX)
}
