#![doc = include_str!("../readme.md")]

use std::io::{self, Read};

use infinitier_datasource::{DataSource, Importer};
use log::debug;

/// The file's signature and version.
pub const PLT_SIGNATURE: &[u8; 8] = b"PLT V1  ";

/// Signature, 8 unknown bytes, width and height.
const HEADER_SIZE: usize = 24;

/// The bytes one pixel takes on disk.
const PIXEL_SIZE: usize = 2;

/// One pixel: which colour of the creature, and which shade of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PltPixel {
    /// The column of the colour's gradient. [`PltPixel::TRANSPARENT_SHADE`]
    /// marks a pixel the doll doesn't cover.
    pub shade: u8,
    /// Which colour: 0 skin, 1 hair, 2 metal, 3 armor, 4 leather, 5 minor
    /// clothing, 6 major clothing, or [`PltPixel::SHADOW_GROUP`].
    pub color_group: u8,
}

impl PltPixel {
    /// The shade of a pixel outside the doll.
    pub const TRANSPARENT_SHADE: u8 = 0xFF;
    /// The group of a pixel of the doll's shadow, which takes no colour.
    pub const SHADOW_GROUP: u8 = 7;

    #[must_use]
    pub fn is_transparent(self) -> bool {
        self.shade == Self::TRANSPARENT_SHADE
    }

    #[must_use]
    pub fn is_shadow(self) -> bool {
        self.color_group == Self::SHADOW_GROUP
    }
}

/// A PLT image. Its size can't disagree with its pixels: an imported one was
/// validated when read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plt {
    width: u32,
    height: u32,
    /// Row by row, top row first (unlike the file, which starts at the
    /// bottom).
    pixels: Vec<PltPixel>,
}

impl Plt {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// `width × height` pixels, row by row, top row first.
    #[must_use]
    pub fn pixels(&self) -> &[PltPixel] {
        &self.pixels
    }
}

/// A PLT file importer.
pub struct PltImporter<'a> {
    pub name: &'a str,
}

impl Importer for PltImporter<'_> {
    type T = Plt;

    fn import(&self, source: &DataSource) -> io::Result<Plt> {
        let mut bytes = Vec::new();
        source.reader()?.read_to_end(&mut bytes)?;
        let plt = parse(&bytes)?;
        debug!("Loaded {} [PLT]: {}x{}", self.name, plt.width, plt.height);
        Ok(plt)
    }
}

/// Whether `source` holds a PLT. The Enhanced Editions keep BAMs under this
/// resource type.
pub fn is_plt(source: &DataSource) -> io::Result<bool> {
    let mut signature = [0; PLT_SIGNATURE.len()];
    match source.reader()?.read_exact(&mut signature) {
        Ok(()) => Ok(signature == *PLT_SIGNATURE),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn parse(bytes: &[u8]) -> io::Result<Plt> {
    let (header, data) = bytes
        .split_first_chunk::<HEADER_SIZE>()
        .ok_or_else(|| invalid(format!("PLT is too short: {} bytes", bytes.len())))?;
    let [signature @ .., w0, w1, w2, w3, h0, h1, h2, h3] = header;
    if !signature.starts_with(PLT_SIGNATURE) {
        return Err(invalid("not a PLT V1 file".to_owned()));
    }
    let width = u32::from_le_bytes([*w0, *w1, *w2, *w3]);
    let height = u32::from_le_bytes([*h0, *h1, *h2, *h3]);
    if width == 0 || height == 0 {
        return Err(invalid(format!("PLT has no pixels: {width}x{height}")));
    }

    let truncated = || {
        invalid(format!(
            "PLT is truncated: {width}x{height} in {} bytes",
            data.len()
        ))
    };
    let row_size = (width as usize)
        .checked_mul(PIXEL_SIZE)
        .ok_or_else(truncated)?;
    let rows = row_size
        .checked_mul(height as usize)
        .and_then(|size| data.get(..size))
        .ok_or_else(truncated)?;

    let pixels = rows
        .chunks_exact(row_size)
        .rev()
        .flat_map(|row| row.as_chunks::<PIXEL_SIZE>().0)
        .map(|[shade, color_group]| PltPixel {
            shade: *shade,
            color_group: *color_group,
        })
        .collect();
    Ok(Plt {
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(width: u32, height: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = PLT_SIGNATURE.to_vec();
        bytes.extend([0; 8]);
        bytes.extend(width.to_le_bytes());
        bytes.extend(height.to_le_bytes());
        bytes.extend(data);
        bytes
    }

    fn pixel(shade: u8, color_group: u8) -> PltPixel {
        PltPixel { shade, color_group }
    }

    #[test]
    fn rows_come_out_top_first() -> io::Result<()> {
        // The file's first row is the image's bottom one.
        let plt = parse(&file(2, 2, &[1, 10, 2, 20, 3, 30, 4, 40]))?;
        assert_eq!((plt.width(), plt.height()), (2, 2));
        assert_eq!(
            plt.pixels(),
            [pixel(3, 30), pixel(4, 40), pixel(1, 10), pixel(2, 20)]
        );
        Ok(())
    }

    #[test]
    fn imports_through_a_data_source() -> io::Result<()> {
        let source = DataSource::new(file(1, 1, &[7, 3]));
        let plt = PltImporter { name: "test" }.import(&source)?;
        assert_eq!(plt.pixels(), [pixel(7, 3)]);
        Ok(())
    }

    #[test]
    fn trailing_bytes_are_ignored() -> io::Result<()> {
        let plt = parse(&file(1, 1, &[7, 3, 9, 9, 9]))?;
        assert_eq!(plt.pixels(), [pixel(7, 3)]);
        Ok(())
    }

    #[test]
    fn only_the_top_shade_is_transparent() {
        assert!(pixel(PltPixel::TRANSPARENT_SHADE, 0).is_transparent());
        assert!(!pixel(PltPixel::TRANSPARENT_SHADE - 1, 0).is_transparent());
    }

    #[test]
    fn the_shadow_is_its_own_group() {
        assert!(pixel(0, PltPixel::SHADOW_GROUP).is_shadow());
        assert!(!pixel(0, PltPixel::SHADOW_GROUP - 1).is_shadow());
    }

    #[test]
    fn recognizes_a_plt_by_its_signature() -> io::Result<()> {
        assert!(is_plt(&DataSource::new(file(1, 1, &[0, 0])))?);
        assert!(!is_plt(&DataSource::new(b"BAMCV1  and the rest".to_vec()))?);
        assert!(!is_plt(&DataSource::new(b"PLT".to_vec()))?);
        Ok(())
    }

    #[test]
    fn rejects_malformed_files() {
        let wrong_signature = {
            let mut bytes = file(1, 1, &[0, 0]);
            bytes[0] = b'X';
            bytes
        };
        for bytes in [
            wrong_signature,
            b"PLT".to_vec(),
            file(0, 1, &[]),
            file(1, 0, &[]),
            // A pixel short.
            file(2, 1, &[0, 0]),
            // Doesn't fit in memory, however it's multiplied.
            file(u32::MAX, u32::MAX, &[0, 0]),
        ] {
            let error = parse(&bytes).err().map(|error| error.kind());
            assert_eq!(error, Some(io::ErrorKind::InvalidData));
        }
    }
}
