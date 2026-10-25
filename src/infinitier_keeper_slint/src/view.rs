//! Pushing the keeper's state to the Slint UI: the shell (title, theme,
//! dialogs, loading overlay), the Item / Spell Browser windows, the empty
//! states, and — when a save is open — the party list, save tabs and the
//! active character tab's content.

use infinitier_core::imported_resource::ImportedResource;
use infinitier_core::imported_resource::gam::{ImportedGamNpc, NpcCre};
use infinitier_core::resource::ResourceType;
use slint::{Image, Model as _, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

use crate::app::{App, Session};
use crate::browser::{BrowserKind, BrowserView};
use crate::fields::AttacksOption;
use crate::settings::ThemeSetting;
use crate::state::{CharacterTab, Model};
use crate::{
    AppWindow, ColorSwatch, FieldCard, GridRow, InvRow, PartyMember, SpellRowUi, VarRow, abilities,
    appearance, characteristics, effects, feats, inventory, journal, levels, memorization,
    miscellaneous, proficiencies, resistances, spells, variables,
};

const APP_TITLE: &str = "Infinitier Keeper (Slint)";

/// Repaint the whole UI from `app`.
pub fn refresh(ui: &AppWindow, app: &mut App) {
    refresh_app(ui, app);
    match &mut app.session {
        Session::Saves(model) => refresh_model(ui, model),
        Session::NoGame | Session::Game(_) => clear_model(ui),
    }
    refresh_browsers(ui, app);
}

/// Repaint the Item and Spell Browser windows. Rows are only built while a
/// window is open (an index can be thousands of entries).
pub fn refresh_browsers(ui: &AppWindow, app: &mut App) {
    let App {
        session, browsers, ..
    } = app;
    let model = match &*session {
        Session::Saves(model) => Some(&**model),
        Session::NoGame | Session::Game(_) => None,
    };
    for kind in BrowserKind::ALL {
        let view = match session.game_data() {
            Some(game_data) => browsers.view(kind, game_data, model),
            None => BrowserView::closed(),
        };
        push_browser(ui, kind, view);
    }
}

/// Push one browser window's state. The list and type checkboxes are only
/// replaced when they change (see [`rows_changed`]); the search text only
/// when it isn't what the user typed.
fn push_browser(ui: &AppWindow, kind: BrowserKind, view: BrowserView) {
    let BrowserView {
        state,
        rows,
        types,
        text,
        all_types,
        scroll_to,
    } = view;
    match kind {
        BrowserKind::Items => {
            if rows_changed(&ui.get_item_browser_rows(), &rows) {
                ui.set_item_browser_rows(ModelRc::new(VecModel::from(rows)));
            }
            if rows_changed(&ui.get_item_browser_types(), &types) {
                ui.set_item_browser_types(ModelRc::new(VecModel::from(types)));
            }
            if ui.get_item_browser_text() != text {
                ui.set_item_browser_text(text);
            }
            ui.set_item_browser_all_types(all_types);
            ui.set_item_browser_state(state);
            if let Some(row) = scroll_to {
                ui.invoke_item_browser_scroll_to(row);
            }
        }
        BrowserKind::Spells => {
            if rows_changed(&ui.get_spell_browser_rows(), &rows) {
                ui.set_spell_browser_rows(ModelRc::new(VecModel::from(rows)));
            }
            if rows_changed(&ui.get_spell_browser_types(), &types) {
                ui.set_spell_browser_types(ModelRc::new(VecModel::from(types)));
            }
            if ui.get_spell_browser_text() != text {
                ui.set_spell_browser_text(text);
            }
            ui.set_spell_browser_all_types(all_types);
            ui.set_spell_browser_state(state);
            if let Some(row) = scroll_to {
                ui.invoke_spell_browser_scroll_to(row);
            }
        }
    }
}

/// Push the state that doesn't depend on an open save: title, theme,
/// status, the Settings / Load / Save dialogs and the loading overlay.
fn refresh_app(ui: &AppWindow, app: &App) {
    let title = match app.game() {
        Some(game) => format!("{APP_TITLE} - {game:?}"),
        None => APP_TITLE.to_owned(),
    };
    ui.set_game_title(title.into());
    ui.set_has_game(app.game().is_some());
    ui.set_has_save(app.model().is_some());
    ui.set_dark_theme(app.settings.theme() == ThemeSetting::Dark);
    ui.set_status(app.status.as_str().into());

    let settings = &app.settings;
    ui.set_settings_open(settings.is_open());
    if settings.is_open() {
        let rows = settings.rows();
        if rows_changed(&ui.get_settings_rows(), &rows) {
            ui.set_settings_rows(ModelRc::new(VecModel::from(rows)));
        }
        ui.set_settings_selected(index_or_none(settings.selected()));
        ui.set_settings_file_error(settings.file_error().unwrap_or_default().into());
        ui.set_settings_error(settings.error().unwrap_or_default().into());
    }

    let load = &app.load_dialog;
    ui.set_load_open(load.is_open());
    if load.is_open() {
        let rows = load.rows();
        if rows_changed(&ui.get_load_rows(), &rows) {
            ui.set_load_rows(ModelRc::new(VecModel::from(rows)));
        }
        ui.set_load_selected(index_or_none(load.selected()));
        ui.set_load_screenshot(load.screenshot());
        ui.set_load_portraits(ModelRc::new(VecModel::from(load.portraits())));
        ui.set_load_error(load.error().unwrap_or_default().into());
        ui.set_load_confirm_delete(load.confirming_delete());
        ui.set_load_delete_name(load.selected_name().into());
    }

    ui.set_save_open(app.save_dialog.is_open());
    ui.set_save_error(app.save_dialog.error().unwrap_or_default().into());

    ui.set_loading(app.pending.is_some());
    if let Some(pending) = &app.pending {
        ui.set_loading_text(format!("Loading {}…", pending.config.game_path.display()).into());
    }
}

/// Empty every save-dependent property (no save open).
fn clear_model(ui: &AppWindow) {
    ui.set_tab_names(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
    ui.set_character_name(SharedString::new());
    ui.set_has_character(false);
    ui.set_party_members(ModelRc::new(VecModel::from(Vec::<PartyMember>::new())));
    ui.set_save_tab_names(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
    clear_content(ui);
}

/// Whether a list property's rows differ from `rows`. Lists are only
/// replaced when they change: a new model rebuilds the row elements, which
/// drops a click in progress — the second click of a double-click lands on
/// a fresh row and never registers as one.
fn rows_changed<T: Clone + PartialEq + 'static>(current: &ModelRc<T>, rows: &[T]) -> bool {
    current.row_count() != rows.len() || current.iter().zip(rows).any(|(old, new)| old != *new)
}

/// A Slint row index for `index`, `-1` for none.
fn index_or_none(index: Option<usize>) -> i32 {
    index.and_then(|i| i32::try_from(i).ok()).unwrap_or(-1)
}

/// Repaint every save- and character-dependent property from the model's
/// active save, selection and tab.
fn refresh_model(ui: &AppWindow, model: &mut Model) {
    let visible_tabs = model.visible_tabs();
    let tab_names: Vec<SharedString> = visible_tabs.iter().map(|t| t.label().into()).collect();
    ui.set_tab_names(ModelRc::new(VecModel::from(tab_names)));
    let tab_index = visible_tabs.iter().position(|t| *t == model.selected_tab);
    ui.set_selected_tab(tab_index.and_then(|i| i32::try_from(i).ok()).unwrap_or(0));

    ui.set_character_name(model.character_name().into());
    let has_character = model.selected_cre().is_some();
    ui.set_has_character(has_character);

    // Attacks dropdown state (used by the Abilities tab's Combat card).
    let attacks_index = model
        .selected_cre()
        .and_then(|cre| AttacksOption::index_for_byte(cre.attacks_byte()))
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(-1);
    ui.set_attacks_index(attacks_index);

    refresh_shell(ui, model);

    if !has_character {
        show_placeholder(ui, SharedString::new());
        return;
    }

    match model.selected_tab {
        CharacterTab::Abilities => show_cards(ui, abilities::build(model)),
        CharacterTab::Resistances => show_cards(ui, resistances::build(model)),
        CharacterTab::Proficiencies => show_cards(ui, proficiencies::build(model)),
        CharacterTab::Memorization => show_cards(ui, memorization::build(model)),
        CharacterTab::Levels => show_cards(ui, levels::build(model)),
        CharacterTab::Feats => show_cards(ui, feats::build(model)),
        CharacterTab::Inventory => {
            show_inventory(ui, inventory::build(model), model.inventory_selected)
        }
        CharacterTab::Spells => {
            let (tabs, tab_index, rows) = spells::build(model);
            show_spells(ui, tabs, tab_index, rows);
        }
        CharacterTab::Characteristics => show_cards(ui, characteristics::build(model)),
        CharacterTab::Appearance => {
            let (name, swatches) = appearance::build(model);
            show_appearance(ui, name, swatches);
        }
        CharacterTab::Miscellaneous => show_cards(ui, miscellaneous::build(model)),
        CharacterTab::LocalVariables => show_table(ui, variables::local(model)),
        CharacterTab::GlobalVariables => show_table(ui, variables::global(model)),
        CharacterTab::Effects => show_grid(
            ui,
            effects::HEADERS,
            effects::WIDTHS,
            effects::build(model),
            false,
        ),
        CharacterTab::JournalEntries => show_grid(
            ui,
            journal::HEADERS,
            journal::WIDTHS,
            journal::build(model),
            true,
        ),
    }
}

/// Push the party list and the save-tab strip to the UI. Set on every
/// refresh because they change when the active save is switched.
fn refresh_shell(ui: &AppWindow, model: &Model) {
    let members: Vec<PartyMember> = model
        .gam
        .party_npcs
        .iter()
        .map(|n| PartyMember {
            name: n.display_name.as_str().into(),
            portrait: party_portrait(model, n),
        })
        .collect();
    ui.set_party_members(ModelRc::new(VecModel::from(members)));
    ui.set_selected_party(i32::try_from(model.selected.unwrap_or(0)).unwrap_or(0));

    let tabs: Vec<SharedString> = model.save_tab_names().into_iter().map(Into::into).collect();
    ui.set_save_tab_names(ModelRc::new(VecModel::from(tabs)));
    ui.set_active_save_tab(i32::try_from(model.active_tab).unwrap_or(0));
}

/// The party-list portrait for a member: the CRE's small (then large)
/// portrait BMP, decoded to a Slint image. Empty when unavailable.
fn party_portrait(model: &Model, npc: &ImportedGamNpc) -> Image {
    let Some(NpcCre::Cre(imported)) = npc.cre.as_ref() else {
        return Image::default();
    };
    let cre = imported.cre();
    for name in [cre.small_portrait_name(), cre.large_portrait_name()] {
        let name = name.to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if let Some(img) = load_bmp_image(model, &name) {
            return img;
        }
    }
    Image::default()
}

/// Decode a BMP resource into a Slint image (party portraits).
fn load_bmp_image(model: &Model, name: &str) -> Option<Image> {
    let imported = model
        .game_data
        .import_by_name_and_type(name, ResourceType::Bmp)
        .ok()?;
    let ImportedResource::Image(img) = imported.as_ref() else {
        return None;
    };
    let (w, h) = (img.image.width(), img.image.height());
    if w == 0 || h == 0 {
        return None;
    }
    let raw = img.image.as_raw();
    let mut pb = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    pb.make_mut_bytes().copy_from_slice(raw);
    Some(Image::from_rgba8(pb))
}

/// Reset every content mode (cards / table / grid / placeholder) to empty,
/// so each `show_*` helper only has to set its own.
fn clear_content(ui: &AppWindow) {
    ui.set_cards_col0(ModelRc::new(VecModel::from(Vec::<FieldCard>::new())));
    ui.set_cards_col1(ModelRc::new(VecModel::from(Vec::<FieldCard>::new())));
    ui.set_cards_col2(ModelRc::new(VecModel::from(Vec::<FieldCard>::new())));
    ui.set_show_table(false);
    ui.set_table_rows(ModelRc::new(VecModel::from(Vec::<VarRow>::new())));
    ui.set_show_grid(false);
    ui.set_grid_rows(ModelRc::new(VecModel::from(Vec::<GridRow>::new())));
    ui.set_show_appearance(false);
    ui.set_color_swatches(ModelRc::new(VecModel::from(Vec::<ColorSwatch>::new())));
    ui.set_animation_name(SharedString::new());
    ui.set_show_inventory(false);
    ui.set_inv_rows(ModelRc::new(VecModel::from(Vec::<InvRow>::new())));
    ui.set_show_spells(false);
    ui.set_spell_rows(ModelRc::new(VecModel::from(Vec::<SpellRowUi>::new())));
    ui.set_spell_tabs(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
    ui.set_placeholder(SharedString::new());
}

/// Show the three-column card grid.
fn show_cards(ui: &AppWindow, cards: [Vec<FieldCard>; 3]) {
    clear_content(ui);
    let [c0, c1, c2] = cards;
    ui.set_cards_col0(ModelRc::new(VecModel::from(c0)));
    ui.set_cards_col1(ModelRc::new(VecModel::from(c1)));
    ui.set_cards_col2(ModelRc::new(VecModel::from(c2)));
}

/// Show the read-only Name · Value table.
fn show_table(ui: &AppWindow, rows: Vec<VarRow>) {
    clear_content(ui);
    ui.set_table_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_show_table(true);
}

/// Show the wide read-only grid (headers + per-column widths + rows).
/// `wrap` makes cells wrap to multiple lines (rows grow to fit) instead of
/// eliding — used by the Journal Entries tab's full entry text.
fn show_grid(ui: &AppWindow, headers: &[&str], widths: &[f32], rows: Vec<GridRow>, wrap: bool) {
    clear_content(ui);
    let headers: Vec<SharedString> = headers.iter().map(|h| (*h).into()).collect();
    ui.set_grid_headers(ModelRc::new(VecModel::from(headers)));
    ui.set_grid_widths(ModelRc::new(VecModel::from(widths.to_vec())));
    ui.set_grid_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_grid_wrap(wrap);
    ui.set_show_grid(true);
}

/// Show the Appearance tab (animation name + colour swatches).
fn show_appearance(ui: &AppWindow, name: SharedString, swatches: Vec<ColorSwatch>) {
    clear_content(ui);
    ui.set_animation_name(name);
    ui.set_color_swatches(ModelRc::new(VecModel::from(swatches)));
    ui.set_show_appearance(true);
}

/// Show the Inventory tab (slot rows, `selected` highlighted).
fn show_inventory(ui: &AppWindow, rows: Vec<InvRow>, selected: Option<usize>) {
    clear_content(ui);
    ui.set_inv_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_inv_selected(index_or_none(selected));
    ui.set_show_inventory(true);
}

/// Show the Spells tab (inner tab strip + spell rows).
fn show_spells(ui: &AppWindow, tabs: Vec<SharedString>, tab_index: i32, rows: Vec<SpellRowUi>) {
    clear_content(ui);
    ui.set_spell_tabs(ModelRc::new(VecModel::from(tabs)));
    ui.set_spell_tab(tab_index);
    ui.set_spell_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_show_spells(true);
}

/// Show the placeholder text (empty = nothing).
fn show_placeholder(ui: &AppWindow, text: SharedString) {
    clear_content(ui);
    ui.set_placeholder(text);
}
