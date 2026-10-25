//! The resource tree: clicks and keys select rows, open groups, and open the
//! selected resource.

use slint::ComponentHandle;

use super::Ctx;
use crate::tree::Outcome;
use crate::{AppWindow, TreeAdapter};

pub(super) fn wire(ui: &AppWindow, ctx: &Ctx) {
    let adapter = ui.global::<TreeAdapter>();
    {
        let ctx = ctx.clone();
        adapter.on_row_clicked(move |row| {
            let Ok(row) = usize::try_from(row) else {
                return;
            };
            let outcome = ctx.app.borrow().tree.click(row);
            follow(&ctx, outcome);
        });
    }
    {
        let ctx = ctx.clone();
        adapter.on_row_toggled(move |row| {
            let Ok(index) = usize::try_from(row) else {
                return;
            };
            ctx.app.borrow().tree.toggle(index);
            // Closing a long group can leave the view past the end.
            if let Some(ui) = ctx.ui.upgrade() {
                ui.invoke_tree_scroll_to(row);
            }
        });
    }
    {
        let ctx = ctx.clone();
        adapter.on_navigate(move |nav| {
            let outcome = ctx.app.borrow().tree.navigate(nav);
            follow(&ctx, outcome);
        });
    }
}

/// Open the resource a click or key selected, and keep its row in view.
fn follow(ctx: &Ctx, outcome: Outcome) {
    if let Some(id) = outcome.open {
        ctx.update(|app| app.open(id));
    }
    if let (Some(ui), Some(row)) = (ctx.ui.upgrade(), outcome.scroll_to) {
        ui.invoke_tree_scroll_to(i32::try_from(row).unwrap_or(i32::MAX));
    }
}
