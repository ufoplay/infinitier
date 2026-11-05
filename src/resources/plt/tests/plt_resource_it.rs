//! The PLT parser against real game files, in `assets/PLT`:
//!
//! - `PLT/`: genuine PLT V1 files, one of each kind the games ship: a
//!   character's doll (`bg2/CHMF1INV`, fighter, male, human), a shield
//!   (`bg2/WPLD1INV`), a helmet (`bg2/WPLH1INV`) and one of the monster dolls
//!   the Enhanced Editions still keep as PLTs (`bgee/MTROINV`, troll). Every
//!   PLT in BG2, IWD and the Enhanced Editions has the same header and size
//!   as these: they differ only in their pixels.
//! - `BAM/`: what the Enhanced Editions keep under the PLT type for most
//!   dolls instead: one compressed BAM (`bgee/CDFC1INV`), the same 1076 bytes
//!   for every doll.

use std::io;
use std::path::{Path, PathBuf};

use infinitier_datasource::{DataSource, Importer};
use infinitier_plt_resource::{PLT_SIGNATURE, Plt, PltImporter, PltPixel, is_plt};
use infinitier_test_utils::{get_all_in_folder_by_extension, get_assets_path};

/// Where the header ends and the pixels start.
const PIXELS_OFFSET: usize = 24;
const WIDTH: u32 = 128;
const HEIGHT: u32 = 160;
/// The colour group of a doll's hair.
const HAIR: u8 = 1;

fn assets() -> PathBuf {
    get_assets_path().join("PLT")
}

fn genuine() -> Vec<PathBuf> {
    let files = get_all_in_folder_by_extension(assets().join("PLT"), "plt", true);
    assert!(!files.is_empty(), "no genuine PLT fixtures found");
    files
}

fn import(bytes: Vec<u8>) -> io::Result<Plt> {
    PltImporter { name: "test" }.import(&DataSource::new(bytes))
}

fn import_file(path: &Path) -> Plt {
    PltImporter { name: "test" }
        .import(&DataSource::new(path))
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn pixel_at(plt: &Plt, x: u32, y: u32) -> PltPixel {
    plt.pixels()[(y * plt.width() + x) as usize]
}

#[test]
fn every_genuine_plt_imports() {
    for path in genuine() {
        let plt = import_file(&path);
        assert_eq!(
            (plt.width(), plt.height()),
            (WIDTH, HEIGHT),
            "{}",
            path.display()
        );
        assert_eq!(plt.pixels().len(), (WIDTH * HEIGHT) as usize);
        assert!(is_plt(&DataSource::new(path.as_path())).expect("readable"));
    }
}

/// Every pixel, against the file's bytes: rows are stored bottom first, two
/// bytes a pixel, shade then colour group.
#[test]
fn pixels_are_the_files_rows_bottom_first() {
    for path in genuine() {
        let bytes = std::fs::read(&path).expect("readable");
        let plt = import_file(&path);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let file_row = HEIGHT - 1 - y;
                let offset = PIXELS_OFFSET + ((file_row * WIDTH + x) * 2) as usize;
                let expected = PltPixel {
                    shade: bytes[offset],
                    color_group: bytes[offset + 1],
                };
                assert_eq!(
                    pixel_at(&plt, x, y),
                    expected,
                    "{} at ({x}, {y})",
                    path.display()
                );
            }
        }
    }
}

/// What each file holds, measured from its bytes: the box around the figure
/// and, for each colour group, its pixels and the rows they span.
struct Measured {
    file: &'static str,
    /// Left, top, right, bottom, inclusive.
    figure: (u32, u32, u32, u32),
    /// Group, pixels, first row, last row.
    groups: &'static [(u8, usize, u32, u32)],
}

const MEASURED: [Measured; 4] = [
    Measured {
        file: "bg2/CHMF1INV.PLT",
        figure: (25, 17, 91, 153),
        groups: &[
            (0, 1108, 24, 115),
            (1, 136, 17, 32),
            (2, 77, 63, 122),
            (4, 1186, 23, 147),
            (5, 315, 64, 96),
            (6, 1003, 36, 92),
            (7, 1425, 23, 153),
        ],
    },
    Measured {
        file: "bg2/WPLD1INV.PLT",
        figure: (47, 37, 100, 115),
        groups: &[(3, 601, 37, 115), (5, 166, 41, 97), (6, 1566, 38, 112)],
    },
    Measured {
        file: "bg2/WPLH1INV.PLT",
        figure: (38, 6, 85, 37),
        groups: &[(2, 318, 6, 37), (3, 82, 12, 36), (6, 34, 13, 24)],
    },
    Measured {
        file: "bgee/MTROINV.PLT",
        figure: (4, 4, 124, 159),
        groups: &[
            (0, 4161, 26, 154),
            (1, 399, 4, 31),
            (2, 69, 82, 153),
            (7, 2738, 9, 159),
        ],
    },
];

#[test]
fn each_file_holds_what_was_measured() {
    for measured in &MEASURED {
        let plt = import_file(&assets().join("PLT").join(measured.file));
        let mut figure = (WIDTH, HEIGHT, 0, 0);
        let mut groups: Vec<(u8, usize, u32, u32)> = Vec::new();
        for y in 0..plt.height() {
            for x in 0..plt.width() {
                let pixel = pixel_at(&plt, x, y);
                if pixel.is_transparent() {
                    continue;
                }
                figure = (
                    figure.0.min(x),
                    figure.1.min(y),
                    figure.2.max(x),
                    figure.3.max(y),
                );
                match groups.iter_mut().find(|g| g.0 == pixel.color_group) {
                    Some(group) => {
                        group.1 += 1;
                        group.2 = group.2.min(y);
                        group.3 = group.3.max(y);
                    }
                    None => groups.push((pixel.color_group, 1, y, y)),
                }
            }
        }
        groups.sort();
        assert_eq!(figure, measured.figure, "{}", measured.file);
        assert_eq!(groups, measured.groups, "{}", measured.file);
    }
}

/// The plainest sign of the rows' order: a doll's hair and a helmet are at
/// the top, a doll's feet and shadow at the bottom.
#[test]
fn dolls_stand_upright() {
    let doll = import_file(&assets().join("PLT/bg2/CHMF1INV.PLT"));
    let rows = |plt: &Plt, keep: &dyn Fn(PltPixel) -> bool| -> Vec<u32> {
        (0..plt.height())
            .filter(|y| (0..plt.width()).any(|x| keep(pixel_at(plt, x, *y))))
            .collect()
    };
    let hair_rows = rows(&doll, &|p| !p.is_transparent() && p.color_group == HAIR);
    let shadow_rows = rows(&doll, &|p| !p.is_transparent() && p.is_shadow());
    assert!(hair_rows.iter().all(|y| *y < HEIGHT / 4), "{hair_rows:?}");
    assert!(shadow_rows.iter().max() > hair_rows.iter().max());

    let helmet = import_file(&assets().join("PLT/bg2/WPLH1INV.PLT"));
    let helmet_rows = rows(&helmet, &|p| !p.is_transparent());
    assert!(
        helmet_rows.iter().all(|y| *y < HEIGHT / 4),
        "{helmet_rows:?}"
    );
}

#[test]
fn equipment_casts_no_shadow() {
    for file in ["bg2/WPLD1INV.PLT", "bg2/WPLH1INV.PLT"] {
        let plt = import_file(&assets().join("PLT").join(file));
        assert!(
            plt.pixels()
                .iter()
                .all(|p| p.is_transparent() || !p.is_shadow()),
            "{file}"
        );
    }
}

#[test]
fn a_bam_under_the_plt_type_is_no_plt() {
    let files = get_all_in_folder_by_extension(assets().join("BAM"), "plt", true);
    assert!(!files.is_empty(), "no BAM-under-PLT fixtures found");
    for path in files {
        let bytes = std::fs::read(&path).expect("readable");
        assert!(bytes.starts_with(b"BAMCV1  "), "{}", path.display());
        assert!(!is_plt(&DataSource::new(path.as_path())).expect("readable"));
        let error = PltImporter { name: "test" }
            .import(&DataSource::new(path.as_path()))
            .expect_err("not a PLT");
        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{}",
            path.display()
        );
    }
}

#[test]
fn a_cut_short_real_plt_is_rejected() {
    let bytes = std::fs::read(assets().join("PLT/bg2/CHMF1INV.PLT")).expect("readable");
    for length in [
        bytes.len() - 1,
        PIXELS_OFFSET + 2,
        PIXELS_OFFSET,
        PLT_SIGNATURE.len(),
    ] {
        let error = import(bytes[..length].to_vec()).expect_err("cut short");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{length} bytes");
    }
}

#[test]
fn bytes_after_a_real_plts_pixels_are_ignored() {
    let path = assets().join("PLT/bg2/CHMF1INV.PLT");
    let mut bytes = std::fs::read(&path).expect("readable");
    bytes.extend([1, 2, 3, 4, 5]);
    let extended = import(bytes).expect("still a PLT");
    assert_eq!(extended, import_file(&path));
}
