//! The window chrome: title, theme, status and the header (`ShellAdapter`),
//! the open-saves tabs (`SaveTabsAdapter`) and the party list
//! (`PartyAdapter`).

use infinitier_core::imported_resource::gam::{ImportedGamNpc, NpcCre};
use slint::{ComponentHandle, Image, SharedString};

use super::rows_model;
use crate::app::App;
use crate::build_data::build_version;
use crate::settings::ThemeSetting;
use crate::state::Model;
use crate::{AppWindow, PartyAdapter, PartyMember, SaveTabsAdapter, ShellAdapter, icons};

pub(super) fn refresh(ui: &AppWindow, app: &App) {
    let shell = ui.global::<ShellAdapter>();
    // The build, then the loaded game's type.
    let app_title = format!("Infinitier Keeper {}", build_version());
    let title = match app.game() {
        Some(game) => format!("{app_title} - {game:?}"),
        None => app_title,
    };
    shell.set_title(title.into());
    shell.set_has_game(app.game().is_some());
    shell.set_has_save(app.model().is_some());
    shell.set_dark_theme(app.settings.theme() == ThemeSetting::Dark);
    shell.set_status(app.status.as_str().into());

    match app.model() {
        Some(model) => refresh_saves(ui, model),
        None => clear_saves(ui),
    }
}

/// The open saves' tabs, and the active save's party and selected member.
/// Set on every refresh: they change when the active save is switched.
fn refresh_saves(ui: &AppWindow, model: &Model) {
    let tabs = ui.global::<SaveTabsAdapter>();
    let names: Vec<SharedString> = model.save_tab_names().into_iter().map(Into::into).collect();
    tabs.set_names(rows_model(names));
    tabs.set_active(i32::try_from(model.active_tab).unwrap_or(0));

    let party = ui.global::<PartyAdapter>();
    let members: Vec<PartyMember> = model
        .gam
        .party_npcs
        .iter()
        .map(|npc| PartyMember {
            name: npc.display_name.as_str().into(),
            portrait: party_portrait(model, npc),
        })
        .collect();
    party.set_members(rows_model(members));
    party.set_selected(i32::try_from(model.selected.unwrap_or(0)).unwrap_or(0));
}

/// No save open: no tabs or party.
fn clear_saves(ui: &AppWindow) {
    ui.global::<SaveTabsAdapter>()
        .set_names(rows_model(Vec::new()));
    ui.global::<PartyAdapter>()
        .set_members(rows_model(Vec::new()));
}

/// The party-list portrait for a member: the CRE's small (then large)
/// portrait BMP. Empty when unavailable.
fn party_portrait(model: &Model, npc: &ImportedGamNpc) -> Image {
    let Some(NpcCre::Cre(imported)) = npc.cre.as_ref() else {
        return Image::default();
    };
    let cre = imported.cre();
    [cre.small_portrait_name(), cre.large_portrait_name()]
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .find_map(|name| icons::bmp_image(&model.game_data, &name))
        .unwrap_or_default()
}
