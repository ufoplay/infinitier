//! Miscellaneous tab for the Slint keeper (read-only).
//!
//! Ports the egui keeper's `ui/tabs/miscellaneous/`. The "Other" card is the
//! V1.0 CRE-header fields (turn-undead, tracking, actor identifier, death
//! variable, the five creature scripts); the three time cards are derived
//! from the GAM `game_time` clock and the party-slot `join_time`. Data
//! extraction (`MiscData` + `misc_data`) is duplicated verbatim from the egui
//! `data.rs`. Display-only — no `commit`.

use infinitier_core::imported_resource::gam::{ImportedGam, ImportedGamNpc, NpcCre};
use infinitier_core::resource::cre::{Cre, CreHeader};
use infinitier_core::resource::gam::Dhm;

use crate::FieldCard;
use crate::state::Model;
use crate::ui_model::{card, readonly_row};

#[derive(Default)]
struct MiscData {
    turn_undead: u8,
    tracking_skill: u8,
    tracking_target: String,
    identifier: u32,
    script_name: String,
    override_script: String,
    class_script: String,
    race_script: String,
    general_script: String,
    default_script: String,
    world_time: Dhm,
    game_time: Dhm,
    joined_party: Dhm,
}

fn misc_data(cre: &Cre, gam: &ImportedGam, npc: &ImportedGamNpc) -> MiscData {
    let game_time = gam.header.game_time;
    // "Joined Party" is the game time elapsed since the member joined: now
    // minus their join timestamp, computed in ticks to keep sub-second
    // precision.
    let joined = game_time
        .to_ticks()
        .saturating_sub(npc.char_stats.join_time);
    let mut data = MiscData {
        world_time: game_time.dhm(),
        game_time: game_time.dhm(),
        joined_party: joined.dhm(),
        ..MiscData::default()
    };

    // The "Other" box is the classic V1.0 header layout (BG/BG2/EE); other
    // engines keep the time cards but leave these blank.
    if let CreHeader::V10(h) = &cre.header {
        data.turn_undead = h.turn_undead_level;
        data.tracking_skill = h.tracking_skill;
        data.tracking_target = h.tracking_target.resref().unwrap_or_default().to_owned();
        data.identifier = u32::from(h.global_actor_enumeration_value)
            | (u32::from(h.local_area_actor_enumeration_value) << 16);
        data.script_name = h.death_variable_set_sprite_is_deadvariable.clone();
        data.override_script = h.creature_script_override.clone();
        data.class_script = h.creature_script_class.clone();
        data.race_script = h.creature_script_race.clone();
        data.general_script = h.creature_script_general.clone();
        data.default_script = h.creature_script_default.clone();
    }
    data
}

fn time_card(title: &str, column: i32, t: &Dhm) -> FieldCard {
    card(
        title,
        column,
        vec![
            readonly_row("Day", &t.day.to_string()),
            readonly_row("Hour", &t.hour.to_string()),
            readonly_row("Minute", &t.minute.to_string()),
        ],
    )
}

/// Build the Miscellaneous-tab cards for the selected creature.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(npc) = model.selected_npc() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let Some(NpcCre::Cre(imported)) = npc.cre.as_ref() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let data = misc_data(imported.cre(), model.gam.as_ref(), npc);

    let other = vec![
        readonly_row("Turn Undead", &data.turn_undead.to_string()),
        readonly_row("Tracking Skill", &data.tracking_skill.to_string()),
        readonly_row("Tracking Target", &data.tracking_target),
        readonly_row("Identifier", &data.identifier.to_string()),
        readonly_row("Script Name", &data.script_name),
        readonly_row("Override Script", &data.override_script),
        readonly_row("Class Script", &data.class_script),
        readonly_row("Race Script", &data.race_script),
        readonly_row("General Script", &data.general_script),
        readonly_row("Default Script", &data.default_script),
    ];

    [
        vec![card("Other", 0, other)],
        vec![
            time_card("World Time", 1, &data.world_time),
            time_card("Game Time", 1, &data.game_time),
            time_card("Joined Party", 1, &data.joined_party),
        ],
        Vec::new(),
    ]
}
