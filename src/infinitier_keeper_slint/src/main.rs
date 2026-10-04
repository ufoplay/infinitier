//! Slint edition of the Infinitier Keeper.
//!
//! Reuses the whole data layer from `infinitier_core` (game-data indexing,
//! save discovery, GAM/CRE parse + export, engine caps) and reimplements the
//! *view* in vanilla Slint instead of egui. The editable-field logic is
//! duplicated from the egui keeper (see [`fields`] / [`cre_fields`]) per the
//! port decision to keep the two keepers independent.

mod abilities;
mod appearance;
mod browser;
mod characteristics;
mod cre_fields;
mod effects;
mod feats;
mod fields;
mod inventory;
mod journal;
mod levels;
mod memorization;
mod miscellaneous;
mod proficiencies;
mod resistances;
mod spells;
mod state;
mod ui_model;
mod variables;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::Parser;
use infinitier_core::engine_caps::EngineCaps;
use infinitier_core::fs::Importer;
use infinitier_core::game::GameDataBuilder;
use infinitier_core::imported_resource::ImportedResource;
use infinitier_core::imported_resource::gam::{ImportedGam, ImportedGamNpc, NpcCre};
use infinitier_core::resource::ResourceType;
use infinitier_core::resource::gam::{GamExporter, GamImporter};
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

use crate::fields::AttacksOption;
use crate::state::{CharacterTab, Model};

slint::include_modules!();

/// Infinitier Keeper (Slint) — cross-engine Infinity Engine save-game editor.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Comma-separated list of game folders (must contain a `CHITIN.KEY`).
    #[arg(long, value_delimiter = ',', required = true, num_args = 1..)]
    game_path: Vec<PathBuf>,
    /// Save to open: a numeric index (alphabetical, from 0) or the folder name.
    #[arg(long)]
    savegame: String,
    /// Log filter, e.g. "warn", "infinitier=debug,warn".
    #[arg(long, default_value = "infinitier=debug,warn")]
    log: String,
}

/// Save-folder file extensions copied when saving (mirrors the egui keeper).
const SAVE_FILE_EXTS: &[&str] = &["sav", "gam", "bmp", "wmp"];

/// Repaint every character-dependent property from the model's current
/// selection + active tab.
fn refresh(ui: &AppWindow, model: &mut Model) {
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

    refresh_browsers(ui, model);
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
        CharacterTab::Inventory => show_inventory(ui, inventory::build(model)),
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

/// Push the shell state (party list, save-tab strip, Load dialog) to the UI.
/// Party + selection are set here because they change when the active save is
/// switched.
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

    ui.set_load_open(model.load_open);
    if model.load_open {
        ui.set_load_rows(ModelRc::new(VecModel::from(browser::load_rows(model))));
        ui.set_load_add_enabled(model.load_selected.is_some());
    }
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

/// Push the Item/Spell Browser overlay state to the UI. Rows are only built
/// while a browser is open (the full index can be thousands of entries).
fn refresh_browsers(ui: &AppWindow, model: &Model) {
    ui.set_item_browser_open(model.item_browser_open);
    if model.item_browser_open {
        ui.set_item_browser_rows(ModelRc::new(VecModel::from(browser::item_rows(model))));
        ui.set_item_add_enabled(browser::item_add_enabled(model));
        ui.set_item_add_label(browser::item_add_label(model));
    }
    ui.set_spell_browser_open(model.spell_browser_open);
    if model.spell_browser_open {
        ui.set_spell_browser_rows(ModelRc::new(VecModel::from(browser::spell_rows(model))));
        ui.set_spell_add_enabled(browser::spell_add_enabled(model));
        ui.set_spell_add_label(browser::spell_add_label(model));
    }
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

/// Show the Inventory tab (selectable slot rows).
fn show_inventory(ui: &AppWindow, rows: Vec<InvRow>) {
    clear_content(ui);
    ui.set_inv_rows(ModelRc::new(VecModel::from(rows)));
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

/// Copy the save folder to `parent/name` and re-export the edited GAM into
/// it. Mirrors the egui keeper's copy-then-export (never overwrites the
/// original save in place).
fn save_copy_and_export(
    name: &str,
    src: &Path,
    parent: &Path,
    gam: &ImportedGam,
) -> std::io::Result<PathBuf> {
    let dest = parent.join(name);
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("destination already exists: {}", dest.display()),
        ));
    }
    std::fs::create_dir(&dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let lower = file_name.to_string_lossy().to_lowercase();
        if !SAVE_FILE_EXTS
            .iter()
            .any(|ext| lower.ends_with(&format!(".{ext}")))
        {
            continue;
        }
        std::fs::copy(entry.path(), dest.join(&file_name))?;
    }
    let gam_path = std::fs::read_dir(&dest)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case("gam"))
        })
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no .GAM file in copied save folder {}", dest.display()),
            )
        })?;
    let exported = gam.clone().export()?;
    GamExporter.export_to_file(&exported, &gam_path)?;
    Ok(dest)
}

/// Run the whole `infinitier_core` load pipeline for `args`, returning the
/// assembled [`Model`].
fn load_model(args: &Args) -> Result<Model, String> {
    let game_data = GameDataBuilder::new(args.game_path.as_slice(), None)
        .and_then(|b| b.build())
        .map_err(|e| format!("failed to load game data: {e}"))?;

    let save_games = game_data.save_games();
    let save_game = if let Ok(idx) = args.savegame.parse::<usize>() {
        save_games
            .by_index(idx)
            .cloned()
            .ok_or_else(|| format!("savegame index {idx} out of range"))?
    } else {
        save_games
            .by_name(&args.savegame)
            .cloned()
            .ok_or_else(|| format!("savegame '{}' not found", args.savegame))?
    };

    let gam = GamImporter {
        name: save_game.folder_name(),
        engine: game_data.game().engine(),
    }
    .import(&save_game.gam)
    .map_err(|e| format!("failed to import GAM: {e}"))?;
    let gam =
        ImportedGam::load(gam, &game_data).map_err(|e| format!("failed to resolve save: {e}"))?;

    let engine_caps =
        EngineCaps::new(&game_data).map_err(|e| format!("failed to build caps: {e}"))?;

    let selected = if gam.party_npcs.is_empty() {
        None
    } else {
        Some(0)
    };
    Ok(Model {
        game_data,
        engine_caps,
        gam: Box::new(gam),
        save_game,
        selected,
        selected_tab: CharacterTab::Abilities,
        inventory_selected: None,
        spell_tab: state::SPELL_TAB_AUTO,
        spell_refs: Vec::new(),
        item_browser_open: false,
        item_index: Vec::new(),
        item_query: String::new(),
        item_selected: None,
        spell_browser_open: false,
        spell_index: Vec::new(),
        spell_query: String::new(),
        spell_selected: None,
        parked: vec![None],
        active_tab: 0,
        load_open: false,
        load_query: String::new(),
        load_selected: None,
    })
}

/// Load a save game by its discovery index and open it in a new tab.
fn load_save_into_tab(model: &mut Model, index: usize) -> Result<(), String> {
    let save_game = model
        .game_data
        .save_games()
        .by_index(index)
        .cloned()
        .ok_or_else(|| format!("savegame index {index} out of range"))?;
    let gam = GamImporter {
        name: save_game.folder_name(),
        engine: model.game_data.game().engine(),
    }
    .import(&save_game.gam)
    .map_err(|e| format!("failed to import GAM: {e}"))?;
    let gam = ImportedGam::load(gam, &model.game_data)
        .map_err(|e| format!("failed to resolve save: {e}"))?;
    model.open_save(save_game, Box::new(gam));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::Builder::new().parse_filters(&args.log).init();

    let model = load_model(&args).unwrap_or_else(|e| {
        log::error!("{e}");
        std::process::exit(1);
    });
    let title = format!("Infinitier Keeper (Slint) - {:?}", model.game_data.game());
    let visible_tabs = Rc::new(model.visible_tabs());
    let model = Rc::new(RefCell::new(model));

    let ui = AppWindow::new()?;
    ui.set_game_title(title.into());

    // Static: tab strip, Attacks options. (Party list is set per-refresh in
    // `refresh_shell`, since it changes with the active save.)
    let tab_names: Vec<SharedString> = visible_tabs.iter().map(|t| t.label().into()).collect();
    ui.set_tab_names(ModelRc::new(VecModel::from(tab_names)));
    ui.set_selected_tab(0);
    let attacks_labels: Vec<SharedString> =
        AttacksOption::ALL.iter().map(|o| o.label.into()).collect();
    ui.set_attacks_options(ModelRc::new(VecModel::from(attacks_labels)));

    refresh(&ui, &mut model.borrow_mut());

    // Select a party member → re-read its tab content.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_select_party(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            m.selected = usize::try_from(idx).ok();
            m.inventory_selected = None; // selection is per-creature
            m.spell_tab = state::SPELL_TAB_AUTO; // re-default inner spell tab
            refresh(&ui, &mut m);
        });
    }

    // Switch the active character sub-tab.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        let visible_tabs = visible_tabs.clone();
        ui.on_select_tab(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            let Some(&tab) = usize::try_from(idx).ok().and_then(|i| visible_tabs.get(i)) else {
                return;
            };
            let mut m = model.borrow_mut();
            m.selected_tab = tab;
            refresh(&ui, &mut m);
        });
    }

    // Commit an edited field → clamp + write + re-read.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_commit_field(move |id, value| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            // Field ids are tab-local; dispatch to the active tab's committer.
            match m.selected_tab {
                CharacterTab::Abilities => abilities::commit(&mut m, id, value.as_str()),
                CharacterTab::Resistances => resistances::commit(&mut m, id, value.as_str()),
                CharacterTab::Proficiencies => proficiencies::commit(&mut m, id, value.as_str()),
                CharacterTab::Memorization => memorization::commit(&mut m, id, value.as_str()),
                CharacterTab::Levels => levels::commit(&mut m, id, value.as_str()),
                CharacterTab::Feats => feats::commit(&mut m, id, value.as_str()),
                _ => {}
            }
            refresh(&ui, &mut m);
        });
    }

    // Commit the Attacks dropdown selection.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_commit_attacks(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            abilities::commit_attacks(&mut m, idx);
            refresh(&ui, &mut m);
        });
    }

    // Inventory: select a slot / edit a quantity / delete an item.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_inv_select(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            inventory::select(&mut m, slot);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_inv_quantity(move |slot, which, value| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            inventory::commit_quantity(&mut m, slot, which, value.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_inv_delete(move |slot| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            inventory::delete(&mut m, slot);
            refresh(&ui, &mut m);
        });
    }

    // Spells: switch inner tab / edit memorised count / delete a spell.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_select_tab(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            spells::select_tab(&mut m, idx);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_memorize(move |ref_id, value| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            spells::commit_memorized(&mut m, ref_id, value.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_delete(move |ref_id| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            spells::delete(&mut m, ref_id);
            refresh(&ui, &mut m);
        });
    }

    // Item Browser: open/close, search, select, add.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_items_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::toggle_items(&mut m);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_item_search(move |q| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::search_items(&mut m, q.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_item_select(move |r| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::select_item(&mut m, r.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_item_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::add_item(&mut m);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_item_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::close_items(&mut m);
            refresh(&ui, &mut m);
        });
    }

    // Spell Browser: open/close, search, select, add.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spells_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::toggle_spells(&mut m);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_browser_search(move |q| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::search_spells(&mut m, q.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_browser_select(move |r| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::select_spell(&mut m, r.as_str());
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_browser_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::add_spell(&mut m);
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_spell_browser_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            browser::close_spells(&mut m);
            refresh(&ui, &mut m);
        });
    }

    // Multi-save tab strip: switch / close.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_select_save_tab(move |i| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            if let Ok(i) = usize::try_from(i) {
                m.switch_save(i);
            }
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_close_save_tab(move |i| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            if let Ok(i) = usize::try_from(i) {
                m.close_save(i);
            }
            refresh(&ui, &mut m);
        });
    }

    // Load dialog: open / search / select / load / close.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_load_clicked(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            m.load_open = true;
            m.load_query.clear();
            m.load_selected = None;
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_load_search(move |q| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            m.load_query = q.to_string();
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_load_select(move |r| {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            m.load_selected = r.as_str().parse::<usize>().ok();
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_load_add(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            if let Some(idx) = m.load_selected {
                match load_save_into_tab(&mut m, idx) {
                    Ok(()) => ui.set_status(SharedString::new()),
                    Err(e) => ui.set_status(format!("Load failed: {e}").into()),
                }
            }
            m.load_open = false;
            m.load_selected = None;
            refresh(&ui, &mut m);
        });
    }
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_load_close(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut m = model.borrow_mut();
            m.load_open = false;
            refresh(&ui, &mut m);
        });
    }

    // Save a copy of the save folder with the edited GAM.
    {
        let model = model.clone();
        let weak = ui.as_weak();
        ui.on_save(move || {
            let Some(ui) = weak.upgrade() else { return };
            let m = model.borrow();
            let name = format!("{} (Slint edit)", m.save_game.folder_name());
            let src = m.save_game.folder_path.as_path();
            let parent = src.parent().unwrap_or(src);
            match save_copy_and_export(&name, src, parent, &m.gam) {
                Ok(dest) => ui.set_status(format!("Saved to {}", dest.display()).into()),
                Err(e) => ui.set_status(format!("Save failed: {e}").into()),
            }
        });
    }

    ui.run()?;
    Ok(())
}
