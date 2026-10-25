//! The Item and Spell Browser windows of the Slint keeper.
//!
//! Ports the egui keeper's `ui/item_browser.rs` / `ui/spell_browser.rs`: a
//! floating list of every ITM / SPL in the loaded game, filtered by free text
//! and per-type checkboxes, with the selected entry's icon and description,
//! from which the entry is added to the selected character. A [`Browser`] is
//! one window's state (the same for both kinds); [`items`] / [`spells`] build
//! the indexes and add the entries. [`Browsers`] belongs to the loaded game,
//! so browsing works with no save open; adding needs one.

mod items;
mod spells;

use std::collections::BTreeSet;

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::spl::{ImportedSpl, Iwd2Placement};
use infinitier_core::resource::Engine;
use slint::{Image, ModelRc, SharedString, VecModel};

use crate::state::Model;
use crate::{BrowserRow, BrowserState, TypeFilter, icons};

/// Which browser a UI callback is for (`0` items, `1` spells).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserKind {
    Items,
    Spells,
}

impl BrowserKind {
    pub const ALL: [BrowserKind; 2] = [BrowserKind::Items, BrowserKind::Spells];
}

impl TryFrom<i32> for BrowserKind {
    type Error = i32;

    fn try_from(kind: i32) -> Result<Self, i32> {
        match kind {
            0 => Ok(Self::Items),
            1 => Ok(Self::Spells),
            other => Err(other),
        }
    }
}

/// One browsable resource, with what's needed to list, filter and describe
/// it.
struct Entry {
    /// The resref (the "Resource" column), lower-case like the resource
    /// index.
    resref: String,
    name: String,
    /// Display type ("Sword", "Wizard", …); the type filter's key.
    type_name: &'static str,
    /// Spell level as stored in the header; `None` for items.
    level: Option<u32>,
    /// `SPELL.IDS` symbol, title-cased; empty for items and unlisted spells.
    script_name: String,
    /// Inventory / spellbook icon BAM resref.
    icon: String,
    /// Description strref, looked up in `dialog.tlk` on selection.
    description_strref: u32,
}

impl Entry {
    /// Whether the entry matches the (already lower-cased) free-text needle.
    fn matches(&self, needle: &str) -> bool {
        needle.is_empty()
            || self.type_name.to_lowercase().contains(needle)
            || self.name.to_lowercase().contains(needle)
            || self.resref.to_lowercase().contains(needle)
            || self.script_name.to_lowercase().contains(needle)
    }

    fn row(&self) -> BrowserRow {
        BrowserRow {
            resref: self.resref.as_str().into(),
            name: self.name.as_str().into(),
            category: self.type_name.into(),
            level: self.level.map(|l| l.to_string()).unwrap_or_default().into(),
            script: self.script_name.as_str().into(),
        }
    }
}

/// The selected entry's icon and description, resolved when it's selected.
struct Detail {
    icon: Image,
    description: String,
}

/// The state of one browser window.
pub struct Browser {
    kind: BrowserKind,
    open: bool,
    /// Every resource of the kind, built on first open.
    entries: Option<Vec<Entry>>,
    /// Free-text filter (matched case-insensitively against type, name,
    /// resref and script name).
    text: String,
    /// Types the user has *unticked*: storing the excluded set means every
    /// type defaults to shown.
    hidden_types: BTreeSet<&'static str>,
    /// Selected resref; drives the detail panel and the add.
    selected: Option<String>,
    /// The selected entry's icon and description; `None` when the selection
    /// isn't in the index.
    detail: Option<Detail>,
    /// Set when the selection moved programmatically (revealed, or arrow
    /// keys): the list scrolls to it on the next repaint.
    scroll_pending: bool,
}

impl Browser {
    fn new(kind: BrowserKind) -> Self {
        Self {
            kind,
            open: false,
            entries: None,
            text: String::new(),
            hidden_types: BTreeSet::new(),
            selected: None,
            detail: None,
            scroll_pending: false,
        }
    }

    /// Open the window if closed, close it if open.
    pub fn toggle(&mut self, game_data: &GameData) {
        if self.open {
            self.open = false;
        } else {
            self.show(game_data);
        }
    }

    /// Open the window, building the index on first open.
    fn show(&mut self, game_data: &GameData) {
        self.open = true;
        if self.entries.is_none() {
            self.entries = Some(match self.kind {
                BrowserKind::Items => items::build_index(game_data),
                BrowserKind::Spells => spells::build_index(game_data),
            });
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn search(&mut self, text: &str) {
        self.text = text.to_owned();
    }

    /// Select `resref` and resolve its icon and description.
    pub fn select(&mut self, game_data: &GameData, resref: &str) {
        self.detail = self.entry(resref).map(|entry| Detail {
            icon: icons::bam_icon(game_data, &entry.icon).unwrap_or_default(),
            description: game_data
                .dialog_tlk()
                .ok()
                .and_then(|t| t.get(entry.description_strref))
                .unwrap_or_default(),
        });
        self.selected = Some(resref.to_owned());
    }

    /// Reveal `resref`: open the window, select it and scroll to it. Clears
    /// the text filter and re-shows its type, so it can't stay hidden behind
    /// a stale filter. The resref is lower-cased to match the index (a CRE
    /// stores it in whatever case it was written).
    pub fn reveal(&mut self, game_data: &GameData, resref: &str) {
        self.show(game_data);
        let resref = resref.to_lowercase();
        self.text.clear();
        if let Some(entry) = self.entry(&resref) {
            self.hidden_types.remove(entry.type_name);
        }
        self.select(game_data, &resref);
        self.scroll_pending = true;
    }

    /// Move the selection `delta` rows through the filtered list (arrow
    /// keys); selects the first row when nothing listed is selected.
    pub fn step(&mut self, game_data: &GameData, delta: i32) {
        let filtered = self.filtered();
        let Some(last) = filtered.len().checked_sub(1) else {
            return;
        };
        let next = match self.selected_position(&filtered) {
            None => 0,
            Some(p) if delta > 0 => (p + 1).min(last),
            Some(p) => p.saturating_sub(1),
        };
        let resref = filtered[next].resref.clone();
        self.select(game_data, &resref);
        self.scroll_pending = true;
    }

    /// The master "Types" checkbox: show or hide every type.
    pub fn set_all_types(&mut self, shown: bool) {
        self.hidden_types = if shown { BTreeSet::new() } else { self.types() };
    }

    /// One type's checkbox.
    pub fn set_type_shown(&mut self, name: &str, shown: bool) {
        let Some(type_name) = self.types().into_iter().find(|t| *t == name) else {
            return;
        };
        if shown {
            self.hidden_types.remove(type_name);
        } else {
            self.hidden_types.insert(type_name);
        }
    }

    /// The selected entry, when it's in the index.
    fn selected_entry(&self) -> Option<&Entry> {
        self.entry(self.selected.as_deref()?)
    }

    fn entry(&self, resref: &str) -> Option<&Entry> {
        self.entries.as_deref()?.iter().find(|e| e.resref == resref)
    }

    /// The entries the type checkboxes and the text filter let through.
    fn filtered(&self) -> Vec<&Entry> {
        let needle = self.text.trim().to_lowercase();
        self.entries
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|e| !self.hidden_types.contains(e.type_name))
            .filter(|e| e.matches(&needle))
            .collect()
    }

    /// The selection's position in `filtered`.
    fn selected_position(&self, filtered: &[&Entry]) -> Option<usize> {
        let selected = self.selected.as_deref()?;
        filtered.iter().position(|e| e.resref == selected)
    }

    /// The distinct types present, sorted, for the checkbox list.
    fn types(&self) -> BTreeSet<&'static str> {
        self.entries
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|e| e.type_name)
            .collect()
    }

    /// What the window shows, with `add` deciding its Add button.
    fn view(&mut self, add: AddControl) -> BrowserView {
        if !self.open {
            return BrowserView::closed();
        }
        let filtered = self.filtered();
        let rows: Vec<BrowserRow> = filtered.iter().map(|e| e.row()).collect();
        let selected_row = self
            .selected_position(&filtered)
            .and_then(|p| i32::try_from(p).ok());
        let noun = match self.kind {
            BrowserKind::Items => "Items",
            BrowserKind::Spells => "Spells",
        };
        let types = self
            .types()
            .into_iter()
            .map(|t| TypeFilter {
                name: t.into(),
                shown: !self.hidden_types.contains(t),
            })
            .collect();
        let AddControl {
            enabled,
            label,
            placements,
        } = add;
        let (icon, description) = match &self.detail {
            Some(Detail { icon, description }) => (icon.clone(), description.as_str().into()),
            None => (Image::default(), SharedString::new()),
        };
        let state = BrowserState {
            open: true,
            count: format!("{noun}: {}", rows.len()).into(),
            selected_row: selected_row.unwrap_or(-1),
            has_selection: self.detail.is_some(),
            icon,
            description,
            add_enabled: enabled,
            add_label: label.into(),
            book_menu: placements.is_some(),
            placements: ModelRc::new(VecModel::from(placements.unwrap_or_default())),
        };
        let scroll_to = std::mem::take(&mut self.scroll_pending)
            .then_some(selected_row)
            .flatten();
        BrowserView {
            state,
            rows,
            types,
            text: self.text.as_str().into(),
            all_types: self.hidden_types.is_empty(),
            scroll_to,
        }
    }
}

/// What a browser's Add button does.
struct AddControl {
    enabled: bool,
    label: String,
    /// IWD2 spells: the labels of the spellbooks to pick from (the button
    /// opens a menu of them). `None` for a plain add.
    placements: Option<Vec<SharedString>>,
}

/// One browser window's state, ready to push to the UI.
pub struct BrowserView {
    pub state: BrowserState,
    pub rows: Vec<BrowserRow>,
    pub types: Vec<TypeFilter>,
    pub text: SharedString,
    pub all_types: bool,
    /// The list row to scroll into view.
    pub scroll_to: Option<i32>,
}

impl BrowserView {
    /// A closed window: nothing listed.
    pub fn closed() -> Self {
        Self {
            state: BrowserState::default(),
            rows: Vec::new(),
            types: Vec::new(),
            text: SharedString::new(),
            all_types: true,
            scroll_to: None,
        }
    }
}

/// Both browser windows of the loaded game.
pub struct Browsers {
    items: Browser,
    spells: Browser,
    /// IWD2: the spellbooks the selected spell can go into, keyed by its
    /// resref — read from the list 2DAs, so cached per selection.
    placements: Option<(String, Vec<Iwd2Placement>)>,
}

impl Default for Browsers {
    fn default() -> Self {
        Self {
            items: Browser::new(BrowserKind::Items),
            spells: Browser::new(BrowserKind::Spells),
            placements: None,
        }
    }
}

impl Browsers {
    pub fn get_mut(&mut self, kind: BrowserKind) -> &mut Browser {
        match kind {
            BrowserKind::Items => &mut self.items,
            BrowserKind::Spells => &mut self.spells,
        }
    }

    /// What browser `kind` shows; `model` is the open saves, if any (the
    /// add target).
    pub fn view(
        &mut self,
        kind: BrowserKind,
        game_data: &GameData,
        model: Option<&Model>,
    ) -> BrowserView {
        let add = match kind {
            BrowserKind::Items => AddControl {
                enabled: model.and_then(items::add_target).is_some()
                    && self.items.selected_entry().is_some(),
                label: items::add_label(model),
                placements: None,
            },
            BrowserKind::Spells => self.spell_add_control(game_data, model),
        };
        self.get_mut(kind).view(add)
    }

    /// The Spell Browser's Add button. AD&D adds to the one spellbook the
    /// SPL implies; IWD2 lets the user pick among the books it fits.
    fn spell_add_control(&mut self, game_data: &GameData, model: Option<&Model>) -> AddControl {
        let can_add = model.is_some_and(spells::can_add);
        let label = spells::add_label(model);
        if game_data.game().engine() != Engine::Iwd2 {
            return AddControl {
                enabled: can_add && self.spells.selected_entry().is_some(),
                label,
                placements: None,
            };
        }
        let placements = self.placements(game_data);
        AddControl {
            enabled: can_add && !placements.is_empty(),
            label,
            placements: Some(placements.iter().map(spells::placement_label).collect()),
        }
    }

    /// The IWD2 spellbooks the selected spell can go into.
    fn placements(&mut self, game_data: &GameData) -> &[Iwd2Placement] {
        let Self {
            spells, placements, ..
        } = self;
        let Some(resref) = spells.selected_entry().map(|e| e.resref.as_str()) else {
            return &[];
        };
        if placements
            .as_ref()
            .is_none_or(|(cached, _)| cached != resref)
        {
            let found = ImportedSpl::iwd2_placements(game_data, resref);
            *placements = Some((resref.to_owned(), found));
        }
        placements.as_ref().map_or(&[], |(_, p)| p.as_slice())
    }

    /// Add browser `kind`'s selected entry to `model`'s selected character:
    /// an item into the Inventory tab's selected slot, a spell into its
    /// spellbook — for IWD2 the book of `placement` (an index into the
    /// offered placements).
    pub fn add(&mut self, kind: BrowserKind, model: &mut Model, placement: i32) {
        match kind {
            BrowserKind::Items => {
                if let (Some(slot), Some(entry)) =
                    (items::add_target(model), self.items.selected_entry())
                {
                    items::add(model, slot, &entry.resref);
                }
            }
            BrowserKind::Spells if model.game_data.game().engine() == Engine::Iwd2 => {
                let picked = usize::try_from(placement)
                    .ok()
                    .and_then(|i| self.placements(&model.game_data).get(i).copied());
                if let Some(placement) = picked {
                    spells::add_iwd2(model, placement);
                }
            }
            BrowserKind::Spells => {
                if let Some(entry) = self.spells.selected_entry() {
                    spells::add_adnd(model, &entry.resref);
                }
            }
        }
    }
}

/// The `primary` strref, falling back to `fallback` when it's empty / the
/// no-string sentinel (`0xFFFFFFFF`).
fn first_strref(primary: u32, fallback: u32) -> u32 {
    if primary == 0 || primary == 0xFFFF_FFFF {
        fallback
    } else {
        primary
    }
}
