//! A BAM's size, read from its header alone.

use std::io::{self, Read};

use flate2::read::ZlibDecoder;
use infinitier_datasource::DataSource;

use crate::{BAM_V1_SIGNATURE, BAM_V2_SIGNATURE, BAMC_SIGNATURE};

/// The width and height of the BAM's first frame, read from its header and
/// frame table: no pixels are decoded, and of a BAMC only the start of the
/// stream is inflated. `None` for a BAM without frames.
pub fn first_frame_size(source: &DataSource) -> io::Result<Option<(u32, u32)>> {
    let mut reader = source.reader()?;
    let mut signature = [0; 8];
    reader.read_exact(&mut signature)?;
    if signature == *BAMC_SIGNATURE.as_bytes() {
        // The uncompressed size, then a zlib stream of a whole BAM V1.
        let mut uncompressed_size = [0; 4];
        reader.read_exact(&mut uncompressed_size)?;
        let mut inflated = ZlibDecoder::new(reader);
        inflated.read_exact(&mut signature)?;
        first_frame_size_after(&signature, &mut inflated)
    } else {
        first_frame_size_after(&signature, &mut reader)
    }
}

/// [`first_frame_size`] of an uncompressed BAM whose `signature` has been
/// read from `rest`.
fn first_frame_size_after(
    signature: &[u8; 8],
    rest: &mut impl Read,
) -> io::Result<Option<(u32, u32)>> {
    const HEADER: u64 = 24;
    // The header after the signature: V1 counts its frames in a u16 at 0x08
    // and keeps their table at the u32 at 0x0C; V2 in a u32 at 0x08, and at
    // 0x14.
    let mut header = [0; 16];
    rest.read_exact(&mut header)?;
    let u32_at = |at: usize| {
        u32::from_le_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]])
    };
    let (frames, table) = if signature == BAM_V1_SIGNATURE.as_bytes() {
        (
            u32::from(u16::from_le_bytes([header[0], header[1]])),
            u32_at(4),
        )
    } else if signature == BAM_V2_SIGNATURE.as_bytes() {
        (u32_at(0), u32_at(12))
    } else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a BAM"));
    };
    if frames == 0 {
        return Ok(None);
    }
    let skip = u64::from(table).checked_sub(HEADER).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "BAM frame table inside its header",
        )
    })?;
    io::copy(&mut rest.take(skip), &mut io::sink())?;
    // A frame entry starts with its width and height, u16 each, in both.
    let mut size = [0; 4];
    rest.read_exact(&mut size)?;
    Ok(Some((
        u32::from(u16::from_le_bytes([size[0], size[1]])),
        u32::from(u16::from_le_bytes([size[2], size[3]])),
    )))
}

#[cfg(test)]
mod tests {
    use infinitier_datasource::Importer;
    use infinitier_test_utils::get_assets_path;

    use super::*;
    use crate::{Bam, BamImporter};

    /// The first frame's size as a full import decodes it.
    fn imported(path: &str) -> (u32, u32) {
        let source = DataSource::new(get_assets_path().join(path));
        let bam = BamImporter { name: path }.import(&source).unwrap();
        match bam {
            Bam::V1(bam) => (bam.frames[0].width, bam.frames[0].height),
            Bam::V2(bam) => (bam.frames[0].width, bam.frames[0].height),
        }
    }

    #[test]
    fn reads_the_first_frame_size_of_every_kind_of_bam() {
        for path in [
            "BAM/V1/01/1chan03B_decompressed.BAM",
            "BAM/V1/01/1chan03B_compressed.BAM",
            "BAM/V2/1CHELM03.BAM",
        ] {
            let source = DataSource::new(get_assets_path().join(path));
            assert_eq!(
                first_frame_size(&source).unwrap(),
                Some(imported(path)),
                "{path}"
            );
        }
    }

    #[test]
    fn a_bam_without_frames_has_no_size() {
        let mut bam = BAM_V1_SIGNATURE.as_bytes().to_vec();
        bam.extend_from_slice(&[0; 16]);
        let source = DataSource::new(bam);
        assert_eq!(first_frame_size(&source).unwrap(), None);
    }

    #[test]
    fn anything_else_is_an_error() {
        let source = DataSource::new(b"NOT A BAM AT ALL, NOT EVEN CLOSE".to_vec());
        assert!(first_frame_size(&source).is_err());
    }
}
