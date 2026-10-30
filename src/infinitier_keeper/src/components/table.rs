//! Cells and columns of themekit's `Table`, which every keeper table is
//! built on.

use slint::{Color, Image, SharedString};

use crate::{TableCell, TableCellKind, TableColumn};

/// How a column fits a text too long for it.
#[derive(Clone, Copy)]
pub enum Overflow {
    /// Cut off with "…"; hovering it shows it whole.
    Elide,
    /// Onto more lines, the row growing to fit (full journal entries).
    Wrap,
}

/// A column titled `title`, `width` logical pixels wide (0 shares the room
/// the other columns leave).
pub fn column(title: &str, width: f32, overflow: Overflow) -> TableColumn {
    TableColumn {
        title: title.into(),
        width,
        alignment: Default::default(),
        wrap: match overflow {
            Overflow::Elide => false,
            Overflow::Wrap => true,
        },
    }
}

/// A text cell.
pub fn text(text: impl Into<SharedString>) -> TableCell {
    cell(
        TableCellKind::Text,
        text.into(),
        Image::default(),
        0.0,
        0.0,
        0.0,
    )
}

/// A whole number the user drags or types, from `minimum` to `maximum`.
pub fn number(value: f32, minimum: f32, maximum: f32) -> TableCell {
    cell(
        TableCellKind::Number,
        SharedString::new(),
        Image::default(),
        value,
        minimum,
        maximum,
    )
}

/// An image, drawn as it is (an item's inventory icon).
pub fn image(image: Image) -> TableCell {
    cell(
        TableCellKind::Image,
        SharedString::new(),
        image,
        0.0,
        0.0,
        0.0,
    )
}

/// An icon button; `label` names it for screen readers and in its tooltip.
pub fn button(icon: Image, label: &str) -> TableCell {
    cell(TableCellKind::Button, label.into(), icon, 0.0, 0.0, 0.0)
}

/// Rows of text cells.
pub fn text_rows<const N: usize, S: Into<SharedString>>(rows: Vec<[S; N]>) -> Vec<Vec<TableCell>> {
    rows.into_iter()
        .map(|cells| cells.into_iter().map(text).collect())
        .collect()
}

fn cell(
    kind: TableCellKind,
    text: SharedString,
    image: Image,
    value: f32,
    minimum: f32,
    maximum: f32,
) -> TableCell {
    TableCell {
        kind,
        text,
        image,
        colorize: Color::default(),
        value,
        minimum,
        maximum,
        step: 1.0,
    }
}
