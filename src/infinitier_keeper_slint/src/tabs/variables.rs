//! Local & Global Variables tabs for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/local_variables/` and
//! `ui/tabs/global_variables/`. Both are two-column Name · Value tables:
//!
//! * **Local** — the selected creature's per-creature `LOCALS` script
//!   variables, in CRE file order (`Cre::local_variables`).
//! * **Global** — the save's `GLOBAL` variables from the GAM, sorted with
//!   EEKeeper's collation (case-insensitive, `_` before letters). The
//!   `global_variable_rows` + `collation_key` logic is duplicated from the
//!   egui `data.rs`.
//!
//! Rendered via a virtualised Slint `ListView` (the global list runs to
//! ~2000 entries). Display-only — no `commit`.

use infinitier_core::imported_resource::gam::ImportedGam;

use crate::VarRow;
use crate::state::Model;

/// The selected creature's LOCALS variables, in CRE file order.
pub fn local(model: &Model) -> Vec<VarRow> {
    let Some(cre) = model.selected_cre() else {
        return Vec::new();
    };
    cre.local_variables()
        .map(|v| VarRow {
            name: v.name.as_str().into(),
            value: v.value.to_string().into(),
        })
        .collect()
}

/// The save's GLOBAL variables, sorted the way EEKeeper presents them.
pub fn global(model: &Model) -> Vec<VarRow> {
    global_rows(model.gam.as_ref())
}

fn global_rows(gam: &ImportedGam) -> Vec<VarRow> {
    let mut rows: Vec<(&str, i32)> = gam
        .variables
        .iter()
        .map(|v| (v.name.as_str(), v.int_value))
        .collect();
    rows.sort_by_cached_key(|(name, _)| collation_key(name));
    rows.into_iter()
        .map(|(name, value)| VarRow {
            name: name.into(),
            value: value.to_string().into(),
        })
        .collect()
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
