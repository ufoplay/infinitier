//! Small constructors for the Slint `FieldRow` / `FieldCard` structs,
//! shared by the per-tab card builders.

use slint::{ModelRc, SharedString, VecModel};

use crate::{FieldCard, FieldRow};

/// An editable text row. `id` is the tab-local field id routed back on
/// commit (dispatched to the active tab by `callbacks::character`).
pub fn editable_row(id: i32, label: &str, value: &str) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        value: value.into(),
        editable: true,
        ..FieldRow::default()
    }
}

/// An editable number row, from `minimum` to `maximum`: a number field
/// (drag or type) whose every change commits `id` with the new value.
pub fn number_row(id: i32, label: &str, value: i64, minimum: i32, maximum: i32) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        value: value.to_string().into(),
        editable: true,
        numeric: true,
        minimum,
        maximum,
        ..FieldRow::default()
    }
}

/// A dropdown row: picking one of `options` commits `id` with its index.
/// `index` is the current option's.
pub fn select_row(id: i32, label: &str, options: Vec<SharedString>, index: i32) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        is_select: true,
        options: ModelRc::new(VecModel::from(options)),
        option_index: index,
        ..FieldRow::default()
    }
}

/// A derived, read-only value row (id `-1`, never committed).
pub fn readonly_row(label: &str, value: &str) -> FieldRow {
    FieldRow {
        id: -1,
        label: label.into(),
        value: value.into(),
        ..FieldRow::default()
    }
}

/// A read-only (disabled) checkbox row.
pub fn check_row(label: &str, checked: bool) -> FieldRow {
    editable_check_row(-1, label, checked)
}

/// An editable checkbox row. Toggling commits `commit-field(id, "1"|"0")`.
pub fn editable_check_row(id: i32, label: &str, checked: bool) -> FieldRow {
    FieldRow {
        id,
        label: label.into(),
        is_check: true,
        checked,
        ..FieldRow::default()
    }
}

/// A titled card in display `column` (0..2); no tooltip.
pub fn card(title: &str, column: i32, rows: Vec<FieldRow>) -> FieldCard {
    FieldCard {
        title: title.into(),
        column,
        rows: ModelRc::new(VecModel::from(rows)),
        tooltip: SharedString::new(),
    }
}

/// The longest line of a [`tooltip`], in characters.
const TOOLTIP_LINE: usize = 64;

/// `text` as a row's or card's tooltip: broken into lines of at most
/// [`TOOLTIP_LINE`] characters, as themekit's tooltip doesn't wrap. Its own
/// line breaks stay.
pub fn tooltip(text: &str) -> SharedString {
    let mut out = String::with_capacity(text.len() + text.len() / TOOLTIP_LINE);
    for (index, paragraph) in text.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let mut line = 0;
        for word in paragraph.split_whitespace() {
            let len = word.chars().count();
            if line > 0 && line + 1 + len > TOOLTIP_LINE {
                out.push('\n');
                line = 0;
            } else if line > 0 {
                out.push(' ');
                line += 1;
            }
            out.push_str(word);
            line += len;
        }
    }
    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_wraps_between_words() {
        let text = "word ".repeat(40);
        let wrapped = tooltip(&text);
        assert!(
            wrapped
                .lines()
                .all(|line| line.chars().count() <= TOOLTIP_LINE)
        );
        assert_eq!(wrapped.split_whitespace().count(), 40);
        assert_eq!(tooltip("one\ntwo").as_str(), "one\ntwo");
    }
}
