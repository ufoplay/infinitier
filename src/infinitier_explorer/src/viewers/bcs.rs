//! Compiled BCS scripts, shown decompiled to BAF with syntax highlighting.

use infinitier_core::{game::GameResource, imported_resource::bcs::ImportedBcs};
use slint::{ModelRc, SharedString, VecModel};

use super::baf_highlight::{self, TokenKind};
use super::info::{origin_label, size_label};
use crate::{BafLine, BafSpan, TokenKind as UiTokenKind};

pub struct BcsViewer {
    cr_count: usize,
    line_count: usize,
    /// The highlighted rows, built once: a script runs to thousands of lines,
    /// and a new model would also scroll the list back to the top.
    lines: ModelRc<BafLine>,
    /// The longest line, in characters.
    columns: usize,
    source: SharedString,
}

impl BcsViewer {
    pub fn new(bcs: ImportedBcs) -> Self {
        let highlighted = baf_highlight::lines(&bcs.baf);
        let columns = highlighted
            .iter()
            .map(|line| baf_highlight::line_width(line))
            .max()
            .unwrap_or(0);
        let lines: Vec<BafLine> = highlighted.into_iter().map(ui_line).collect();
        Self {
            cr_count: bcs.bcs.condition_responses.len(),
            line_count: bcs.baf.lines().count(),
            lines: ModelRc::new(VecModel::from(lines)),
            columns,
            source: bcs.baf.as_str().into(),
        }
    }

    pub fn lines(&self) -> ModelRc<BafLine> {
        self.lines.clone()
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn source(&self) -> SharedString {
        self.source.clone()
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        vec![
            format!("{} CR blocks", self.cr_count),
            format!("{} BAF lines", self.line_count),
            size_label(resource),
            origin_label(resource),
        ]
    }
}

fn ui_line(spans: Vec<baf_highlight::Span>) -> BafLine {
    let spans: Vec<BafSpan> = spans
        .into_iter()
        .map(|span| BafSpan {
            text: span.text.into(),
            kind: ui_kind(span.kind),
        })
        .collect();
    BafLine {
        spans: ModelRc::new(VecModel::from(spans)),
    }
}

fn ui_kind(kind: TokenKind) -> UiTokenKind {
    match kind {
        TokenKind::Keyword => UiTokenKind::Keyword,
        TokenKind::Trigger => UiTokenKind::Trigger,
        TokenKind::Action => UiTokenKind::Action,
        TokenKind::Identifier => UiTokenKind::Identifier,
        TokenKind::String => UiTokenKind::String,
        TokenKind::Number => UiTokenKind::Number,
        TokenKind::Comment => UiTokenKind::Comment,
        TokenKind::Punctuation => UiTokenKind::Punctuation,
        TokenKind::Whitespace => UiTokenKind::Whitespace,
    }
}
