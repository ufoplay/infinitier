//! FNT font envelopes, modelled on NearInfinity's `FntResource` struct view.
//!
//! FNT is a 4-byte envelope: just a `# extra letters` count. The `Letters`
//! BAM and `Extra letters` BMP NearInfinity shows aren't stored in the file:
//! they're named after the FNT itself (`DIALOG.FNT` ⇒ `DIALOG.BAM` +
//! `DIALOG.BMP`), and the importer does the same. Vanilla EE FNTs carry
//! 23–100 KB of engine-internal bytes after the header, which aren't parsed:
//! the viewer notes it and dumps their start.

use infinitier_core::{
    game::GameResource,
    resource::fnt::{Fnt, HEADER_LEN},
};

use super::info::{origin_label, size_label};

/// How many body bytes the hex dump shows.
const PREVIEW_BYTES: usize = 256;

/// A row of the struct table.
pub struct Field {
    pub attribute: &'static str,
    pub value: String,
    pub offset: &'static str,
    pub size: String,
}

pub struct FntViewer {
    fnt: Fnt,
}

impl FntViewer {
    pub fn new(fnt: Fnt) -> Self {
        Self { fnt }
    }

    /// NearInfinity's "Edit" tab. The two resource references show
    /// offset 0 / size 8, a `ResourceRef`'s nominal layout, although their
    /// bytes never appear in the FNT.
    pub fn fields(&self) -> Vec<Field> {
        vec![
            Field {
                attribute: "# extra letters",
                value: self.fnt.extra_letters_count.to_string(),
                offset: "0 h",
                size: HEADER_LEN.to_string(),
            },
            Field {
                attribute: "Letters",
                value: self.fnt.letters_bam.clone(),
                offset: "0 h",
                size: "8".to_owned(),
            },
            Field {
                attribute: "Extra letters",
                value: self.fnt.extra_letters_bmp.clone(),
                offset: "0 h",
                size: "8".to_owned(),
            },
        ]
    }

    /// Why the file is bigger than its fields; `None` when it isn't.
    pub fn note(&self) -> Option<String> {
        let body_len = self.fnt.body().len();
        (body_len > 0).then(|| {
            format!(
                "Note: FNT is a stub. Glyph data lives in {} and {}; \
                 the {body_len} bytes past offset 0x04 in this file are \
                 engine-internal and not parsed (NearInfinity treats them \
                 the same way).",
                self.fnt.letters_bam, self.fnt.extra_letters_bmp,
            )
        })
    }

    /// The line above the hex dump.
    pub fn dump_summary(&self) -> String {
        let body = self.fnt.body();
        if body.is_empty() {
            return "(no body bytes)".to_owned();
        }
        format!(
            "Showing first {} of {} bytes (offset 0x{HEADER_LEN:X} in file).",
            body.len().min(PREVIEW_BYTES),
            body.len(),
        )
    }

    /// The body's first bytes as a hex dump; empty for an empty body.
    pub fn dump(&self) -> String {
        let body = self.fnt.body();
        hex_dump(&body[..body.len().min(PREVIEW_BYTES)], HEADER_LEN)
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        vec![
            "FNT".to_owned(),
            format!("# extra letters: {}", self.fnt.extra_letters_count),
            size_label(resource),
            format!("body: {} B (opaque)", self.fnt.body().len()),
            origin_label(resource),
        ]
    }
}

/// 16 bytes a line: the file offset (`bytes` start at `base`), the bytes in
/// hex, then as ASCII with `.` for the unprintable ones.
fn hex_dump(bytes: &[u8], base: usize) -> String {
    let mut dump = String::with_capacity(bytes.len() * 4);
    for (i, chunk) in bytes.chunks(16).enumerate() {
        dump.push_str(&format!("{:08x}  ", base + i * 16));
        for b in chunk {
            dump.push_str(&format!("{b:02x} "));
        }
        for _ in chunk.len()..16 {
            dump.push_str("   ");
        }
        dump.push(' ');
        dump.extend(chunk.iter().map(|&b| {
            if (32..127).contains(&b) {
                b as char
            } else {
                '.'
            }
        }));
        dump.push('\n');
    }
    dump
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_dump_pads_the_last_line() {
        let bytes: Vec<u8> = (0x41..0x41 + 18).collect();
        let dump = hex_dump(&bytes, 4);
        let lines: Vec<&str> = dump.lines().collect();
        assert_eq!(
            lines,
            [
                "00000004  41 42 43 44 45 46 47 48 49 4a 4b 4c 4d 4e 4f 50  ABCDEFGHIJKLMNOP",
                "00000014  51 52                                            QR",
            ]
        );
    }

    #[test]
    fn hex_dump_hides_unprintable_bytes() {
        assert!(hex_dump(&[0, 0x7f, b'a'], 0).ends_with("  ..a\n"));
        assert_eq!(hex_dump(&[], 0), "");
    }
}
