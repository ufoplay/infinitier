//! Spells tab for the Slint keeper (editable).
//!
//! Ports the egui keeper's `ui/tabs/spells/`. An inner tab strip filters the
//! creature's spells: AD&D (Innate/Wizard/Cleric over the flat `known_spells`)
//! or IWD2 (per-class categories over the per-level blocks). Rows show
//! Level · Memorized (editable) · Spell · Resource, sorted by (level, name),
//! with a Delete action. The extraction + name resolution are duplicated from
//! the egui `data.rs`/`view.rs`. Each row's `SpellRef` is stashed on
//! `Model::spell_refs` (parallel to the displayed rows) so an edit finds its
//! spell by row index. (The Spell Browser "add spell" flow is not yet ported.)

use infinitier_core::resource::Engine;
use infinitier_core::resource::cre::{Cre, Iwd2Spellbook, Iwd2Table, SpellType, SubSections};
use infinitier_core::resource::two_da::TwoDA;
use slint::SharedString;

use crate::SpellRowUi;
use crate::state::{Model, SPELL_TAB_AUTO, SpellRefKey};

// ── Inner-tab definitions (duplicated from the egui data.rs) ──────────

const ADND_TABS: &[(&str, SpellType)] = &[
    ("Innate", SpellType::Innate),
    ("Wizard", SpellType::Wizard),
    ("Cleric", SpellType::Priest),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpellCategory {
    Bard,
    Cleric,
    Domain,
    Druid,
    Innate,
    Paladin,
    Ranger,
    ShapeChange,
    Song,
    Sorcerer,
    Wizard,
}

impl SpellCategory {
    const ALL: &'static [(SpellCategory, &'static str)] = &[
        (SpellCategory::Bard, "Bard"),
        (SpellCategory::Cleric, "Cleric"),
        (SpellCategory::Domain, "Domain"),
        (SpellCategory::Druid, "Druid"),
        (SpellCategory::Innate, "Innate"),
        (SpellCategory::Paladin, "Paladin"),
        (SpellCategory::Ranger, "Ranger"),
        (SpellCategory::ShapeChange, "Shape Change"),
        (SpellCategory::Song, "Song"),
        (SpellCategory::Sorcerer, "Sorcerer"),
        (SpellCategory::Wizard, "Wizard"),
    ];

    fn list_2da(self) -> &'static str {
        match self {
            SpellCategory::Bard
            | SpellCategory::Cleric
            | SpellCategory::Druid
            | SpellCategory::Paladin
            | SpellCategory::Ranger
            | SpellCategory::Sorcerer
            | SpellCategory::Wizard => "listspll",
            SpellCategory::Domain => "listdomn",
            SpellCategory::Innate => "listinnt",
            SpellCategory::Song => "listsong",
            SpellCategory::ShapeChange => "listshap",
        }
    }

    fn spellbook(self) -> Iwd2Spellbook {
        match self {
            SpellCategory::Bard => Iwd2Spellbook::Bard,
            SpellCategory::Cleric => Iwd2Spellbook::Cleric,
            SpellCategory::Domain => Iwd2Spellbook::Domain,
            SpellCategory::Druid => Iwd2Spellbook::Druid,
            SpellCategory::Innate => Iwd2Spellbook::Innate,
            SpellCategory::Paladin => Iwd2Spellbook::Paladin,
            SpellCategory::Ranger => Iwd2Spellbook::Ranger,
            SpellCategory::ShapeChange => Iwd2Spellbook::ShapeChange,
            SpellCategory::Song => Iwd2Spellbook::Song,
            SpellCategory::Sorcerer => Iwd2Spellbook::Sorcerer,
            SpellCategory::Wizard => Iwd2Spellbook::Wizard,
        }
    }
}

/// A spell row before name resolution.
struct Row {
    level: u16,
    memorized: u32,
    resref: String,
    key: SpellRefKey,
}

fn adnd_rows(cre: &Cre, spell_type: SpellType) -> Vec<Row> {
    let SubSections::V1(sub) = &cre.sub_sections else {
        return Vec::new();
    };
    let mut rows: Vec<Row> = Vec::new();
    for known in &sub.known_spells {
        if known.spell_type != spell_type {
            continue;
        }
        if rows
            .iter()
            .any(|r| r.resref.eq_ignore_ascii_case(&known.spell))
        {
            continue;
        }
        let memorized = sub
            .memorized_spells
            .iter()
            .filter(|m| m.spell.eq_ignore_ascii_case(&known.spell))
            .count() as u32;
        rows.push(Row {
            level: known.level.saturating_add(1),
            memorized,
            resref: known.spell.clone(),
            key: SpellRefKey::Adnd {
                spell_type,
                resref: known.spell.clone(),
            },
        });
    }
    rows
}

fn adnd_count(cre: &Cre, spell_type: SpellType) -> usize {
    adnd_rows(cre, spell_type).len()
}

fn iwd2_rows(cre: &Cre, category: SpellCategory, list: Option<&TwoDA>) -> Vec<Row> {
    let SubSections::V22(sub) = &cre.sub_sections else {
        return Vec::new();
    };
    let book = category.spellbook();
    let resref = |index: u32| -> String {
        list.and_then(|l| l.rows.get(&index.to_string()))
            .and_then(|cells| cells.last())
            .cloned()
            .unwrap_or_default()
    };
    let mut rows = Vec::new();
    let push_leveled = |tables: &[Iwd2Table; 9], rows: &mut Vec<Row>| {
        for (i, table) in tables.iter().enumerate() {
            let level = i as u16 + 1;
            for slot in &table.entries {
                rows.push(Row {
                    level,
                    memorized: slot.memorized,
                    resref: resref(slot.index),
                    key: SpellRefKey::Iwd2 {
                        book,
                        level,
                        index: slot.index,
                    },
                });
            }
        }
    };
    let push_flat = |table: &Iwd2Table, rows: &mut Vec<Row>| {
        for slot in &table.entries {
            rows.push(Row {
                level: 1,
                memorized: slot.memorized,
                resref: resref(slot.index),
                key: SpellRefKey::Iwd2 {
                    book,
                    level: 1,
                    index: slot.index,
                },
            });
        }
    };
    match category {
        SpellCategory::Bard => push_leveled(&sub.bard_spells, &mut rows),
        SpellCategory::Cleric => push_leveled(&sub.cleric_spells, &mut rows),
        SpellCategory::Domain => push_leveled(&sub.domain_spells, &mut rows),
        SpellCategory::Druid => push_leveled(&sub.druid_spells, &mut rows),
        SpellCategory::Paladin => push_leveled(&sub.paladin_spells, &mut rows),
        SpellCategory::Ranger => push_leveled(&sub.ranger_spells, &mut rows),
        SpellCategory::Sorcerer => push_leveled(&sub.sorcerer_spells, &mut rows),
        SpellCategory::Wizard => push_leveled(&sub.wizard_spells, &mut rows),
        SpellCategory::Innate => push_flat(&sub.abilities, &mut rows),
        SpellCategory::Song => push_flat(&sub.songs, &mut rows),
        SpellCategory::ShapeChange => push_flat(&sub.shapes, &mut rows),
    }
    rows
}

fn iwd2_count(cre: &Cre, category: SpellCategory) -> usize {
    let SubSections::V22(sub) = &cre.sub_sections else {
        return 0;
    };
    let leveled = |tables: &[Iwd2Table; 9]| tables.iter().map(|t| t.entries.len()).sum();
    match category {
        SpellCategory::Bard => leveled(&sub.bard_spells),
        SpellCategory::Cleric => leveled(&sub.cleric_spells),
        SpellCategory::Domain => leveled(&sub.domain_spells),
        SpellCategory::Druid => leveled(&sub.druid_spells),
        SpellCategory::Paladin => leveled(&sub.paladin_spells),
        SpellCategory::Ranger => leveled(&sub.ranger_spells),
        SpellCategory::Sorcerer => leveled(&sub.sorcerer_spells),
        SpellCategory::Wizard => leveled(&sub.wizard_spells),
        SpellCategory::Innate => sub.abilities.entries.len(),
        SpellCategory::Song => sub.songs.entries.len(),
        SpellCategory::ShapeChange => sub.shapes.entries.len(),
    }
}

/// Resolve a spell resref to its display name (SPL generic name → tlk).
fn resolve_name(model: &Model, resref: &str) -> String {
    if resref.is_empty() {
        return String::new();
    }
    let name = model
        .game_data
        .import_spl_by_name(resref)
        .ok()
        .and_then(|spl| {
            model
                .game_data
                .dialog_tlk()
                .ok()
                .and_then(|tlk| tlk.get(spl.header.name_strref()))
        })
        .filter(|s| !s.is_empty());
    name.unwrap_or_else(|| resref.to_string())
}

fn label(name: &str, count: usize) -> String {
    if count > 0 {
        format!("{name} ({count})")
    } else {
        name.to_string()
    }
}

/// Build the Spells tab: inner-tab labels, selected index, and the displayed
/// rows for that tab. Records the per-row `SpellRef` on `model.spell_refs`.
pub fn build(model: &mut Model) -> (Vec<SharedString>, i32, Vec<SpellRowUi>) {
    let Some(cre) = model.selected_cre() else {
        model.spell_refs.clear();
        return (Vec::new(), 0, Vec::new());
    };
    let is_iwd2 = model.game_data.game().engine() == Engine::Iwd2;

    // Inner-tab labels + counts, and the selected tab (auto-picks the first
    // non-empty tab the first time a creature is shown).
    let (tabs, tab_index, mut rows): (Vec<SharedString>, usize, Vec<Row>) = if is_iwd2 {
        let counts: Vec<usize> = SpellCategory::ALL
            .iter()
            .map(|(c, _)| iwd2_count(cre, *c))
            .collect();
        let default_idx = counts.iter().position(|&n| n > 0).unwrap_or(0);
        let sel = if model.spell_tab == SPELL_TAB_AUTO {
            default_idx
        } else {
            model.spell_tab.min(SpellCategory::ALL.len() - 1)
        };
        let labels = SpellCategory::ALL
            .iter()
            .zip(&counts)
            .map(|((_, name), &n)| label(name, n).into())
            .collect();
        let category = SpellCategory::ALL[sel].0;
        let list = model.game_data.import_2da_by_name(category.list_2da()).ok();
        let rows = iwd2_rows(cre, category, list.as_deref());
        (labels, sel, rows)
    } else {
        let counts: Vec<usize> = ADND_TABS
            .iter()
            .map(|(_, ty)| adnd_count(cre, *ty))
            .collect();
        let sel = if model.spell_tab == SPELL_TAB_AUTO {
            0
        } else {
            model.spell_tab.min(ADND_TABS.len() - 1)
        };
        let labels = ADND_TABS
            .iter()
            .zip(&counts)
            .map(|((name, _), &n)| label(name, n).into())
            .collect();
        let rows = adnd_rows(cre, ADND_TABS[sel].1);
        (labels, sel, rows)
    };

    // Resolve names, then sort by (level, name) like EEKeeper.
    let names: Vec<String> = rows
        .iter()
        .map(|r| resolve_name(model, &r.resref))
        .collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        rows[a]
            .level
            .cmp(&rows[b].level)
            .then_with(|| names[a].to_lowercase().cmp(&names[b].to_lowercase()))
    });

    let mut refs = Vec::with_capacity(order.len());
    let mut ui_rows = Vec::with_capacity(order.len());
    for (display_i, &i) in order.iter().enumerate() {
        refs.push(rows[i].key.clone());
        ui_rows.push(SpellRowUi {
            ref_id: display_i as i32,
            level: rows[i].level.to_string().into(),
            memorized: rows[i].memorized.to_string().into(),
            name: names[i].as_str().into(),
            resref: rows[i].resref.as_str().into(),
        });
    }

    // Drop the borrow-derived data into the model.
    let _ = &mut rows; // rows no longer needed
    model.spell_refs = refs;
    model.spell_tab = tab_index;
    (tabs, tab_index as i32, ui_rows)
}

/// Switch the inner spell tab.
pub fn select_tab(model: &mut Model, idx: i32) {
    if let Ok(i) = usize::try_from(idx) {
        model.spell_tab = i;
    }
}

/// Commit an edited "Memorized" count for the row `ref_id`.
pub fn commit_memorized(model: &mut Model, ref_id: i32, value: &str) {
    let Some(key) = usize::try_from(ref_id)
        .ok()
        .and_then(|i| model.spell_refs.get(i).cloned())
    else {
        return;
    };
    let count = value.trim().parse::<u32>().unwrap_or(0).min(99);
    if let Some(cre) = model.selected_cre_mut() {
        set_memorized(cre, &key, count);
    }
}

/// Delete the spell in row `ref_id`.
pub fn delete(model: &mut Model, ref_id: i32) {
    let Some(key) = usize::try_from(ref_id)
        .ok()
        .and_then(|i| model.spell_refs.get(i).cloned())
    else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        match key {
            SpellRefKey::Adnd { spell_type, resref } => {
                cre.remove_known_spell(spell_type, &resref);
            }
            SpellRefKey::Iwd2 { book, level, index } => {
                cre.remove_iwd2_spell(book, level as usize, index);
            }
        }
    }
}

fn set_memorized(cre: &mut Cre, key: &SpellRefKey, count: u32) {
    match key {
        SpellRefKey::Adnd { spell_type, resref } => {
            cre.set_known_spell_memorized_count(*spell_type, resref, count);
        }
        SpellRefKey::Iwd2 { book, level, index } => {
            cre.set_iwd2_spell_memorized(*book, *level as usize, *index, count);
        }
    }
}
