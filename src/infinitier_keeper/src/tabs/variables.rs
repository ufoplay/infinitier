//! Local & Global Variables tabs: a Name · Value table each, every row
//! ending with Edit and Delete (pinned, in view however wide the table),
//! with an Add button under it. Add, Edit (or a double-click) open the
//! variable dialog ([`editor`]); Delete removes the variable at once.
//!
//! * **Local** — the selected creature's `LOCALS` script variables, in CRE
//!   order (`op187` effects, see `Cre::local_variables`). Only a creature
//!   with V2 effects can hold them.
//! * **Global** — the save's `GLOBAL` variables from the GAM, sorted with
//!   EEKeeper's collation (case-insensitive, `_` before letters).
//!
//! Rendered in a themekit `Table`, which only creates the rows in view (the
//! global list runs to ~2000 entries).

pub mod editor;

use infinitier_core::imported_resource::gam::ImportedGam;
use slint::SharedString;

use crate::state::Model;
use crate::tabs::CharacterTab;

/// Which variables a tab edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Local,
    Global,
}

impl Scope {
    /// The scope `tab` edits, if it's a variables tab.
    pub fn of(tab: CharacterTab) -> Option<Self> {
        match tab {
            CharacterTab::LocalVariables => Some(Self::Local),
            CharacterTab::GlobalVariables => Some(Self::Global),
            CharacterTab::Abilities
            | CharacterTab::Levels
            | CharacterTab::Characteristics
            | CharacterTab::Appearance
            | CharacterTab::Inventory
            | CharacterTab::Spells
            | CharacterTab::Feats
            | CharacterTab::Memorization
            | CharacterTab::Proficiencies
            | CharacterTab::Resistances
            | CharacterTab::Effects
            | CharacterTab::JournalEntries
            | CharacterTab::Miscellaneous => None,
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Global => "global",
        }
    }
}

/// A variable's row: its name and value.
pub type VarRow = [SharedString; 2];

/// What a variables tab shows.
pub struct Page {
    pub rows: Vec<VarRow>,
    /// Why there are no rows, when there are none.
    pub empty: &'static str,
    /// Whether a variable can be added (a creature with classic V1 effects
    /// can't hold local ones).
    pub can_add: bool,
}

/// One variable: where it is (its place among the creature's locals, or
/// its index in the GAM's globals), its name and its value.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Variable {
    address: usize,
    name: String,
    value: i32,
}

/// The `scope`'s variables, in display order.
fn variables(model: &Model, scope: Scope) -> Vec<Variable> {
    match scope {
        Scope::Local => model
            .selected_cre()
            .map(|cre| {
                cre.local_variables()
                    .enumerate()
                    .map(|(address, local)| Variable {
                        address,
                        name: local.name,
                        value: local.value,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Scope::Global => global_order(&model.gam)
            .into_iter()
            .filter_map(|index| {
                let variable = model.gam.variables.get(index)?;
                Some(Variable {
                    address: index,
                    name: variable.name.clone(),
                    value: variable.int_value,
                })
            })
            .collect(),
    }
}

/// The GAM's globals' indices, sorted the way EEKeeper presents them.
fn global_order(gam: &ImportedGam) -> Vec<usize> {
    let mut order: Vec<usize> = (0..gam.variables.len()).collect();
    order.sort_by_cached_key(|&index| collation_key(&gam.variables[index].name));
    order
}

/// EEKeeper sorts case-insensitively with `_` ordering *before* letters —
/// uppercase, then remap `_` (0x5F) to `@` (0x40, before `A`).
fn collation_key(name: &str) -> Vec<u8> {
    name.bytes()
        .map(|b| match b.to_ascii_uppercase() {
            b'_' => b'@',
            c => c,
        })
        .collect()
}

/// What the `scope`'s tab shows.
pub fn page(model: &Model, scope: Scope) -> Page {
    let rows = variables(model, scope)
        .into_iter()
        .map(|variable| [variable.name.into(), variable.value.to_string().into()])
        .collect();
    let holds_locals = model
        .selected_cre()
        .is_some_and(|cre| cre.has_local_variables());
    let (empty, can_add) = match scope {
        Scope::Local if !holds_locals => (
            "This character's effects are in the classic format, which has no room for local \
             variables.",
            false,
        ),
        Scope::Local => ("This character has no local variables.", true),
        Scope::Global => ("This save has no global variables.", true),
    };
    Page {
        rows,
        empty,
        can_add,
    }
}

/// The variable in `row` of the `scope`'s table.
fn at(model: &Model, scope: Scope, row: i32) -> Option<Variable> {
    let row = usize::try_from(row).ok()?;
    variables(model, scope).into_iter().nth(row)
}

/// Delete the variable in `row` of the active tab's table.
pub fn delete(model: &mut Model, row: i32) {
    let Some(scope) = Scope::of(model.selected_tab) else {
        return;
    };
    let Some(variable) = at(model, scope, row) else {
        return;
    };
    match scope {
        Scope::Local => {
            if let Some(cre) = model.selected_cre_mut() {
                cre.remove_local_variable(variable.address);
            }
        }
        Scope::Global => {
            if variable.address < model.gam.variables.len() {
                model.gam.variables.remove(variable.address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underscores_sort_before_letters_and_case_is_ignored() {
        let mut names = ["beta", "ALPHA", "_x", "Alpha_b"];
        names.sort_by_key(|name| collation_key(name));
        assert_eq!(names, ["_x", "ALPHA", "Alpha_b", "beta"]);
    }
}
