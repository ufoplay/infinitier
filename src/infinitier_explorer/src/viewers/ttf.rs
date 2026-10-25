//! TrueType fonts: the font's `name`-table metadata, and a sample sentence
//! drawn in the font itself at several sizes. The font is registered with
//! Slint's font collection on its first view (see [`crate::fonts`]) and drawn
//! by its family name.

use infinitier_core::{game::GameResource, resource::ttf::Ttf};

use super::info::{origin_label, size_label};

/// The sample sentence's sizes, in pixels.
pub const SAMPLE_SIZES: [f32; 5] = [12.0, 16.0, 24.0, 36.0, 48.0];

pub struct TtfViewer {
    ttf: Ttf,
    /// The family the font was registered under; `None` draws in the
    /// default font.
    family: Option<String>,
}

impl TtfViewer {
    pub fn new(ttf: Ttf, family: Option<String>) -> Self {
        Self { ttf, family }
    }

    pub fn family(&self) -> &str {
        self.family.as_deref().unwrap_or_default()
    }

    pub fn full_name(&self) -> &str {
        &self.ttf.full_name
    }

    /// The optional `name`-table strings, labelled.
    pub fn details(&self) -> Vec<(&'static str, &str)> {
        [
            ("Version", &self.ttf.version),
            ("Designer", &self.ttf.designer),
            ("Foundry", &self.ttf.manufacturer),
            ("Copyright", &self.ttf.copyright),
            ("PostScript", &self.ttf.postscript_name),
        ]
        .into_iter()
        .filter_map(|(label, value)| value.as_deref().map(|value| (label, value)))
        .collect()
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        let ttf = &self.ttf;
        let mut info = vec![
            "TTF".to_owned(),
            format!("{} {}", ttf.family_name, ttf.subfamily_name),
            format!("{} glyphs", ttf.glyph_count),
            format!(
                "em={} asc={} desc={} line_gap={}",
                ttf.units_per_em, ttf.ascender, ttf.descender, ttf.line_gap,
            ),
        ];
        if ttf.is_monospaced {
            info.push("monospaced".to_owned());
        }
        info.extend([size_label(resource), origin_label(resource)]);
        info
    }
}

/// The label over the sample at `size`: `" 12 px:"`.
pub fn sample_label(size: f32) -> String {
    format!("{size:>3.0} px:")
}
