//! Characteristics tab.
//!
//! The Identity card's fields are edited, each picked by name from the
//! game's own tables ([`identity`]). The Special Edits card ([`special`])
//! holds shortcuts to features the record has no field for. The kill stats come from the GAM party
//! slot and the Miscellaneous checkboxes from `CreatureFlags::MISC`, both
//! shown read-only. `dialog.tlk` strrefs are resolved during `build`, which
//! runs once per refresh (tab switch / selection / commit), so no cache is
//! needed.

mod identity;
mod special;

use infinitier_core::game::GameData;
use infinitier_core::imported_resource::gam::NpcCre;
use infinitier_core::resource::cre::CreatureFlags;

use crate::FieldCard;
use crate::components::field_card::{card, check_row, readonly_row};
use crate::state::Model;

/// The text of `strref` in `dialog.tlk`; empty for none.
fn resolve_strref(game_data: &GameData, strref: u32) -> String {
    if strref == 0 || strref == 0xFFFF_FFFF {
        return String::new();
    }
    game_data
        .dialog_tlk()
        .ok()
        .and_then(|tlk| tlk.get(strref))
        .unwrap_or_default()
}

/// Build the Characteristics-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(npc) = model.selected_npc() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let Some(NpcCre::Cre(imported)) = npc.cre.as_ref() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let game_data = &model.game_data;
    let cre = imported.cre();

    let stats = &npc.char_stats;
    let kill_rows = vec![
        readonly_row(
            "Strongest Kill Name",
            &resolve_strref(game_data, stats.most_powerful_vanquished_name),
        ),
        readonly_row(
            "Strongest Kill XP",
            &stats.most_powerful_vanquished_xp.to_string(),
        ),
        readonly_row("Chapter Kills", &stats.kills_number_chapter.to_string()),
        readonly_row("Chapter Kills XP", &stats.kills_xp_chapter.to_string()),
        readonly_row("Game Kills", &stats.kills_number_game.to_string()),
        readonly_row("Game Kills XP", &stats.kills_xp_game.to_string()),
    ];

    let flags = cre.creature_flags();
    let misc_rows = CreatureFlags::MISC
        .iter()
        .map(|(label, bit)| check_row(label, flags.contains(*bit)))
        .collect();

    [
        vec![card("Identity", 0, identity::rows(game_data, cre))],
        vec![
            card("Kill Stats", 1, kill_rows),
            card("Miscellaneous", 1, misc_rows),
        ],
        vec![special::card_for(game_data.game(), cre, 2)],
    ]
}

/// Apply an edit of row `id`, on whichever card it is.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    if !special::commit(model, id, value) {
        identity::commit(model, id, value);
    }
}
