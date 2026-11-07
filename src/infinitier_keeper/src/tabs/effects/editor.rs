//! The effect dialog, which edits a draft: a copy of one of the creature's
//! effects (Edit, Clone) or a new one (Add). Each field the user changes is
//! written into the draft, kept in range, and shown again from it. Save
//! writes the draft over the effect it copies (Edit), or after the last one
//! as a new effect (Add, Clone); Cancel drops it.

use super::fields::{Draft, Field, Kind, Number};
use super::labels::{RESISTANCE_BITS, choices};
use crate::FieldCard;
use crate::components::field_card::{card, editable_row, number_row, select_row};
use crate::state::Model;

/// What the dialog's draft is, which says where Save puts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The effect at this index of the creature's effect list, which the
    /// draft replaces.
    Edit(usize),
    /// A new effect, added last.
    Add,
    /// A copy of an effect, added last as a new one.
    Clone,
}

/// An open effect dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectEditor {
    mode: Mode,
    draft: Draft,
}

/// The dialog's cards, by title, and their fields. A V1 effect lacks most
/// of the V2-only ones, which its cards leave out.
const CARDS: [(&str, &[Field]); 3] = [
    (
        "Effect",
        &[
            Field::Type,
            Field::Opcode,
            Field::Target,
            Field::Power,
            Field::Param1,
            Field::Param2,
            Field::Param3,
            Field::Param4,
            Field::Param5,
            Field::Special,
            Field::Name,
            Field::Projectile,
            Field::FirstApply,
        ],
    ),
    (
        "Timing & saves",
        &[
            Field::Timing,
            Field::Duration,
            Field::TimeApplied,
            Field::Probability1,
            Field::Probability2,
            Field::Resistance,
            Field::SaveType,
            Field::SaveBonus,
            Field::DiceThrown,
            Field::DiceSides,
            Field::PrimaryType,
            Field::SecondaryType,
        ],
    ),
    (
        "Resources & source",
        &[
            Field::Resource0,
            Field::Resource1,
            Field::Resource2,
            Field::Resource3,
            Field::ParentType,
            Field::ParentFlags,
            Field::ParentSlot,
            Field::MinLevel,
            Field::MaxLevel,
            Field::CasterLevel,
            Field::CasterX,
            Field::CasterY,
            Field::TargetX,
            Field::TargetY,
        ],
    ),
];

/// The most values a number field is dragged through; past that it's typed.
const DRAG_MAX_VALUES: i64 = 256;

impl EffectEditor {
    /// A dialog editing `draft`, saved as `mode` says.
    pub fn new(mode: Mode, draft: Draft) -> Self {
        Self { mode, draft }
    }

    pub fn title(&self) -> &'static str {
        match self.mode {
            Mode::Edit(_) => "Edit effect",
            Mode::Add => "Add effect",
            Mode::Clone => "Clone effect",
        }
    }

    /// The dialog's cards, one per column.
    pub fn cards(&self) -> Vec<FieldCard> {
        (0..)
            .zip(CARDS)
            .map(|(column, (title, fields))| {
                let rows = fields.iter().filter_map(|&field| self.row(field)).collect();
                card(title, column, rows)
            })
            .collect()
    }

    /// `field`'s row: a dropdown, a number (dragged or typed) or a text;
    /// `None` where the effect's format has no such field.
    fn row(&self, field: Field) -> Option<crate::FieldRow> {
        let effect = self.draft.as_ref();
        let (id, label) = (field.id(), field.label());
        Some(match effect.kind(field)? {
            Kind::Choice => {
                let (choices, index) = choices(effect, field)?;
                let names = choices.into_iter().map(|(_, name)| name.into()).collect();
                select_row(id, label, names, i32::try_from(index).unwrap_or(-1))
            }
            Kind::Number(number) => {
                let value = effect.shown(field)?;
                match number_range(number) {
                    Some((minimum, maximum)) => number_row(id, label, value, minimum, maximum),
                    None => editable_row(id, label, &value.to_string()),
                }
            }
            Kind::Text(_) => editable_row(id, label, effect.text(field)?),
        })
    }

    /// Apply the dialog's edit of field `id`: `value` is the index picked in
    /// its dropdown, or the number or text entered.
    fn commit(&mut self, id: i32, value: &str) {
        let Some(field) = Field::from_id(id) else {
            return;
        };
        let draft = &mut self.draft;
        match draft.as_ref().kind(field) {
            Some(Kind::Choice) => {
                let Some((choices, _)) = choices(draft.as_ref(), field) else {
                    return;
                };
                let Some(&(picked, _)) = value
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| choices.get(index))
                else {
                    return;
                };
                // The dropdown names bits 0 and 1; the others stay.
                let picked = if field == Field::Resistance {
                    let others = draft.as_ref().number(field).unwrap_or(0) & !RESISTANCE_BITS;
                    others | picked
                } else {
                    picked
                };
                draft.set_number(field, picked);
            }
            Some(Kind::Number(number)) => {
                if let Some(raw) = number.parse(value) {
                    draft.set_number(field, raw);
                }
            }
            Some(Kind::Text(_)) => draft.set_text(field, value),
            None => {}
        }
    }
}

/// A number field's range when it's few enough values to drag through.
fn number_range(number: Number) -> Option<(i32, i32)> {
    let (minimum, maximum) = number.range();
    if maximum - minimum >= DRAG_MAX_VALUES {
        return None;
    }
    Some((i32::try_from(minimum).ok()?, i32::try_from(maximum).ok()?))
}

/// Field `id` of the open dialog was edited.
pub fn commit(model: &mut Model, id: i32, value: &str) {
    if let Some(editor) = model.effect_editor.as_mut() {
        editor.commit(id, value);
    }
}

/// Save: write the draft into the selected creature's effects, and close.
pub fn save(model: &mut Model) {
    let Some(EffectEditor { mode, draft }) = model.effect_editor.take() else {
        return;
    };
    let index = match mode {
        Mode::Edit(index) => Some(index),
        Mode::Add | Mode::Clone => None,
    };
    if let Some(cre) = model.selected_cre_mut() {
        draft.store(cre.effect_list_mut(), index);
    }
}

/// Cancel: close, dropping the draft.
pub fn cancel(model: &mut Model) {
    model.effect_editor = None;
}

#[cfg(test)]
mod tests {
    use infinitier_core::resource::cre::{Effect, EffectV1Body, Resistance};

    use super::*;

    fn editor(draft: Draft) -> EffectEditor {
        EffectEditor::new(Mode::Edit(0), draft)
    }

    fn row_labels(editor: &EffectEditor) -> Vec<Vec<String>> {
        use slint::Model as _;
        editor
            .cards()
            .iter()
            .map(|card| card.rows.iter().map(|row| row.label.to_string()).collect())
            .collect()
    }

    #[test]
    fn a_v2_effect_shows_every_field_once() {
        let editor = editor(Draft::V2(Box::new(Effect::permanent(0, 0, 0))));
        let labels: Vec<String> = row_labels(&editor).concat();
        let expected: Vec<String> = Field::ALL.iter().map(|f| f.label().to_owned()).collect();
        let mut sorted = labels.clone();
        sorted.sort();
        let mut expected_sorted = expected;
        expected_sorted.sort();
        assert_eq!(sorted, expected_sorted);
    }

    #[test]
    fn a_v1_effect_shows_its_own_fields() {
        let editor = editor(Draft::V1(EffectV1Body::permanent(0, 0, 0)));
        let labels = row_labels(&editor);
        assert_eq!(labels[2], ["Resource 0"]);
        assert!(!labels.concat().iter().any(|label| label == "Parameter 3"));
    }

    #[test]
    fn edits_are_kept_in_range_and_choices_by_index() {
        let mut editor = editor(Draft::V1(EffectV1Body::permanent(0, 0, 0)));
        editor.commit(Field::Probability1.id(), "300");
        editor.commit(Field::Param1.id(), "-2");
        editor.commit(Field::Resource0.id(), " spwi112 ");
        // "Instant, Permanent" is the 10th timing (9).
        editor.commit(Field::Timing.id(), "9");
        editor.commit(Field::Type.id(), "126");
        let effect = editor.draft.as_ref();
        assert_eq!(effect.number(Field::Probability1), Some(255));
        assert_eq!(effect.shown(Field::Param1), Some(-2));
        assert_eq!(effect.text(Field::Resource0), Some("SPWI112"));
        assert_eq!(effect.number(Field::Timing), Some(9));
        assert_eq!(effect.number(Field::Opcode), Some(126));
    }

    #[test]
    fn the_resistance_dropdown_keeps_the_other_bits() {
        let mut effect = Effect::permanent(0, 0, 0);
        effect.resistance = Resistance::from_bits_retain(0b1000 | 0b10);
        let mut editor = editor(Draft::V2(Box::new(effect)));
        editor.commit(Field::Resistance.id(), "1");
        assert_eq!(
            editor.draft.as_ref().number(Field::Resistance),
            Some(0b1001)
        );
    }
}
