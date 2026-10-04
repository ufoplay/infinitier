//! Abilities-tab card builder for the Slint keeper.
//!
//! Mirrors the egui keeper's `ui/tabs/abilities.rs`: the same three-column
//! card layout, the same per-section field filter
//! ([`EditableField::section`] + [`EditableField::is_visible`]), and the same
//! live "effective THAC0 / AC / Max HP / Lore / thief-skill" previews. The
//! egui version recomputes previews from in-flight text on every keystroke;
//! Slint binds a model instead, so previews are recomputed from the
//! committed CRE whenever the model is rebuilt (i.e. after each commit or
//! selection change).

use infinitier_core::engine_caps::ThiefSkill;
use infinitier_core::imported_resource::gam::NpcCre;
use infinitier_core::resource::Engine;
use infinitier_core::resource::cre::{Cre, CreHeader, CreHeaderV22};

use crate::fields::{AttacksOption, EditableField, Section};
use crate::state::Model;
use crate::ui_model::{card, readonly_row};
use crate::{FieldCard, FieldRow};

/// Commit an edited Abilities field back to the model: parse + clamp +
/// write to the selected CRE (or the GAM for party-wide fields).
pub fn commit(model: &mut Model, id: i32, value: &str) {
    let Some(field) = EditableField::from_index(id) else {
        return;
    };
    if field.is_gam_field() {
        let Model {
            engine_caps, gam, ..
        } = model;
        field.write_clamped_gam(gam, value, engine_caps);
        return;
    }
    let Model {
        engine_caps,
        gam,
        selected,
        ..
    } = model;
    if let Some(cre) = selected
        .and_then(|idx| gam.party_npcs.get_mut(idx))
        .and_then(|npc| npc.cre.as_mut())
        .and_then(|c| match c {
            NpcCre::Cre(b) => Some(b.cre_mut()),
            NpcCre::Ref(_) => None,
        })
    {
        field.write_clamped_cre(cre, value, engine_caps);
    }
}

/// Commit the Attacks dropdown selection (an index into
/// [`AttacksOption::ALL`]) to the selected CRE.
pub fn commit_attacks(model: &mut Model, idx: i32) {
    let Some(option) = usize::try_from(idx)
        .ok()
        .and_then(|i| AttacksOption::ALL.get(i))
    else {
        return;
    };
    if let Some(cre) = model.selected_cre_mut() {
        cre.set_attacks_byte(option.byte);
    }
}

/// Build the Abilities-tab cards for the model's selected creature.
/// Returns three per-column card lists (col 0/1/2). Empty when no CRE is
/// selected.
pub fn build(model: &Model) -> [Vec<FieldCard>; 3] {
    let Some(cre) = model.selected_cre() else {
        return [Vec::new(), Vec::new(), Vec::new()];
    };
    let caps = &model.engine_caps;
    let gam = model.gam.as_ref();
    let engine = model.game_data.game().engine();

    let strength = cre.strength();
    let strength_pct = cre.strength_bonus().unwrap_or(0);
    let dexterity = cre.dexterity();
    let constitution = cre.constitution();
    let bonuses = caps.ability_bonuses(strength, strength_pct, dexterity, constitution);

    let class_id = match &cre.header {
        CreHeader::V10(h) => h.class_class_ids,
        CreHeader::V12(h) => h.class_class_ids,
        CreHeader::V90(h) => h.class_class_ids,
        CreHeader::V22(_) => 0,
    };
    let (is_warrior, hp_roll_cap) = caps.class_hp_profile(class_id);
    let class_count = caps.class_count(class_id);

    // ── Ability scores card ──────────────────────────────────────────
    let con_hp_per_level = caps.constitution_hp_per_level(constitution, is_warrior) as i8;
    let mut ability_rows = vec![row(
        cre,
        EditableField::Strength,
        gam,
        Some(format_bonus(
            engine,
            BonusKind::ToHit,
            bonuses.thac0_from_strength,
        )),
    )];
    if EditableField::StrengthPct.is_visible(cre) {
        ability_rows.push(row(cre, EditableField::StrengthPct, gam, None));
    }
    ability_rows.push(row(
        cre,
        EditableField::Dexterity,
        gam,
        Some(format_bonus(
            engine,
            BonusKind::Ac,
            bonuses.ac_from_dexterity,
        )),
    ));
    ability_rows.push(row(
        cre,
        EditableField::Constitution,
        gam,
        Some(format_bonus(
            engine,
            BonusKind::HpPerLevel,
            con_hp_per_level,
        )),
    ));
    ability_rows.push(row(cre, EditableField::Intelligence, gam, None));
    ability_rows.push(row(cre, EditableField::Wisdom, gam, None));
    ability_rows.push(row(cre, EditableField::Charisma, gam, None));
    let ability_total = u32::from(cre.strength())
        + u32::from(cre.dexterity())
        + u32::from(cre.constitution())
        + u32::from(cre.intelligence())
        + u32::from(cre.wisdom())
        + u32::from(cre.charisma());
    ability_rows.push(readonly_row("Total", &ability_total.to_string()));

    // ── Combat & status card ─────────────────────────────────────────
    let effective_thac0: i32 = match engine {
        Engine::Iwd2 => i32::from(cre.thac0_or_bab()) + i32::from(bonuses.thac0_from_strength),
        Engine::Bg | Engine::Bg2 | Engine::Ee | Engine::Iwd | Engine::Pst => {
            i32::from(cre.thac0_or_bab()) - i32::from(bonuses.thac0_from_strength)
        }
    };
    let effective_ac: i32 = i32::from(cre.ac_natural()) + i32::from(bonuses.ac_from_dexterity);
    let con_bonus = caps.max_hp_constitution_bonus_for(
        constitution,
        is_warrior,
        hp_roll_cap,
        cre.primary_level(),
        cre.class_levels(),
        class_count,
        cre.is_dual_classed(),
    );
    let effective_max_hp: i32 = i32::from(cre.maximum_hit_points()) + con_bonus;
    let mut combat_rows = Vec::new();
    for &field in EditableField::ALL {
        if field.section() != Section::CombatStatus || !field.is_visible(cre) {
            continue;
        }
        let bonus = match field {
            EditableField::Thac0 => Some(format!("(effective: {effective_thac0})")),
            EditableField::AcNatural => Some(format!("(effective: {effective_ac})")),
            EditableField::MaxHp => Some(format!("(effective: {effective_max_hp})")),
            _ => None,
        };
        combat_rows.push(row(cre, field, gam, bonus));
    }

    // ── Experience & levels card ─────────────────────────────────────
    let mut xp_rows = Vec::new();
    for &field in EditableField::ALL {
        if field.section() != Section::ExperienceLevels || !field.is_visible(cre) {
            continue;
        }
        xp_rows.push(row(cre, field, gam, None));
    }
    if let CreHeader::V22(h) = &cre.header {
        xp_rows.push(readonly_row("Total levels", &h.total_levels.to_string()));
        xp_rows.push(readonly_row(
            "Per-class levels",
            &format_iwd2_class_levels(h),
        ));
    }

    // ── Morale card ──────────────────────────────────────────────────
    let mut morale_rows = Vec::new();
    for &field in EditableField::ALL {
        if field.section() != Section::Morale || !field.is_visible(cre) {
            continue;
        }
        morale_rows.push(row(cre, field, gam, None));
    }
    if morale_rows.is_empty() {
        morale_rows.push(readonly_row("Morale system", "disabled (d20)"));
    }

    // ── Skills card (d20 or thief) ───────────────────────────────────
    let skills_card = if EditableField::Alchemy.is_visible(cre) {
        let mut rows = Vec::new();
        for &field in EditableField::ALL {
            if field.section() != Section::D20Skills || !field.is_visible(cre) {
                continue;
            }
            rows.push(row(cre, field, gam, None));
        }
        card("Skills", 2, rows)
    } else {
        let effective_lore = (i32::from(cre.lore().unwrap_or(0))
            + caps.lore_bonus(cre.intelligence(), cre.wisdom()))
        .max(0);
        let mut rows = Vec::new();
        for &field in EditableField::ALL {
            if field.section() != Section::ThiefSkills || !field.is_visible(cre) {
                continue;
            }
            let bonus = if field == EditableField::Lore {
                Some(format!("(effective: {effective_lore})"))
            } else if let Some(skill) = thief_skill_of(field) {
                let base = thief_skill_base_value(cre, field);
                let effective = (i32::from(base)
                    + caps.thief_skill_bonus(skill, cre.dexterity(), cre.race()))
                .max(0);
                Some(format!("(effective: {effective})"))
            } else {
                None
            };
            rows.push(row(cre, field, gam, bonus));
        }
        card("Thief Skills", 2, rows)
    };

    [
        vec![
            card("Ability scores", 0, ability_rows),
            card("Combat & status", 0, combat_rows),
        ],
        vec![
            card("Experience & levels", 1, xp_rows),
            card("Morale", 1, morale_rows),
        ],
        vec![skills_card],
    ]
}

// ── Row / card constructors ──────────────────────────────────────────

fn row(
    cre: &Cre,
    field: EditableField,
    gam: &infinitier_core::imported_resource::gam::ImportedGam,
    bonus: Option<String>,
) -> FieldRow {
    FieldRow {
        id: field.index(),
        label: field.label(cre).into(),
        value: field.read_text(cre, gam).into(),
        bonus: bonus.unwrap_or_default().into(),
        editable: field != EditableField::Attacks,
        is_attacks: field == EditableField::Attacks,
        is_check: false,
        checked: false,
    }
}

// ── Bonus formatter (copied from the egui abilities tab) ─────────────

#[derive(Debug, Clone, Copy)]
enum BonusKind {
    ToHit,
    Ac,
    HpPerLevel,
}

fn format_bonus(engine: Engine, kind: BonusKind, value: i8) -> String {
    let suffix = match kind {
        BonusKind::ToHit => match engine {
            Engine::Iwd2 => "to hit",
            Engine::Bg | Engine::Bg2 | Engine::Ee | Engine::Iwd | Engine::Pst => "THAC0",
        },
        BonusKind::Ac => "AC",
        BonusKind::HpPerLevel => "HP/lvl",
    };
    format!("{value:+} {suffix}")
}

/// Map an editable thief-skill field to its [`ThiefSkill`] for the
/// SKILLDEX dexterity lookup. `None` for Lore (LOREBON, handled separately).
fn thief_skill_of(field: EditableField) -> Option<ThiefSkill> {
    Some(match field {
        EditableField::Lockpicking => ThiefSkill::OpenLocks,
        EditableField::FindTraps => ThiefSkill::FindTraps,
        EditableField::SetTraps => ThiefSkill::SetTraps,
        EditableField::PickPockets => ThiefSkill::PickPockets,
        EditableField::DetectIllusion => ThiefSkill::DetectIllusion,
        EditableField::HideInShadows => ThiefSkill::HideInShadows,
        EditableField::MoveSilently => ThiefSkill::MoveSilently,
        _ => return None,
    })
}

/// Read the stored base byte for an AD&D thief-skill field from the CRE.
fn thief_skill_base_value(cre: &Cre, field: EditableField) -> u8 {
    match field {
        EditableField::HideInShadows => cre.hide_in_shadows().unwrap_or(0),
        EditableField::MoveSilently => cre.move_silently(),
        EditableField::Lockpicking => cre.lockpicking().unwrap_or(0),
        EditableField::FindTraps => cre.find_traps().unwrap_or(0),
        EditableField::SetTraps => cre.set_traps().unwrap_or(0),
        EditableField::PickPockets => cre.pick_pockets().unwrap_or(0),
        EditableField::DetectIllusion => cre.detect_illusion().unwrap_or(0),
        _ => 0,
    }
}

fn format_iwd2_class_levels(h: &CreHeaderV22) -> String {
    let entries = [
        ("Barbarian", h.barbarian_levels),
        ("Bard", h.bard_levels),
        ("Cleric", h.cleric_levels),
        ("Druid", h.druid_levels),
        ("Fighter", h.fighter_levels),
        ("Monk", h.monk),
        ("Paladin", h.paladin_levels),
        ("Ranger", h.ranger_levels),
        ("Rogue", h.rogue_levels),
        ("Sorcerer", h.sorcerer_levels),
        ("Wizard", h.wizard_levels),
    ];
    let parts: Vec<String> = entries
        .iter()
        .filter(|(_, lvl)| *lvl > 0)
        .map(|(name, lvl)| format!("{name} {lvl}"))
        .collect();
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join(", ")
    }
}
