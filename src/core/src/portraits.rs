//! The pictures a game has for a creature's two portraits.
//!
//! Every game draws each portrait slot from pictures of one size: the
//! Enhanced Editions' small portrait is a 169×266 BMP, the originals' a
//! 38×60 one, and so on (see [`PortraitSlot::format`]); and names it after
//! the size it is, `NMINSCS` / `NMINSCM` / `NMINSCL`. A slot offers the
//! game's resources in its format, which leaves out the BMPs that are
//! something else (area maps, the paperdoll's palettes, screens, the
//! credits' art…). Those resources include the player's own portraits, in
//! the `portraits` folder.

use infinitier_common::{Game, ResourceType};

use crate::game::GameData;
use crate::resource::{bam, bmp};

/// One of a creature's two portraits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortraitSlot {
    Small,
    Large,
}

/// What every picture a portrait slot shows has in common: its resource
/// type, its size and how its name ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortraitFormat {
    pub r#type: ResourceType,
    pub width: u32,
    pub height: u32,
    /// The letters a picture's name may end in: a portrait is named after
    /// its size, small, medium or large (IWD's `HMF_S` too). Empty when any
    /// name will do: PST's follow no such rule (`AMPNM1`, `WMPNAM1`).
    pub suffixes: &'static [char],
}

impl PortraitFormat {
    /// Whether `name` ends as this format's pictures do.
    pub fn fits_name(&self, name: &str) -> bool {
        self.suffixes.is_empty()
            || name
                .chars()
                .next_back()
                .is_some_and(|last| self.suffixes.contains(&last.to_ascii_uppercase()))
    }
}

impl PortraitSlot {
    /// This slot's format in `game`, as its own characters' portraits have
    /// it. PST keeps its large portraits as BAMs.
    pub fn format(self, game: Game) -> PortraitFormat {
        let (r#type, (width, height)) = match (game, self) {
            (Game::Bg | Game::Bg2 | Game::Tutu, Self::Small) => (ResourceType::Bmp, (38, 60)),
            (Game::Bg | Game::Bg2 | Game::Tutu | Game::Iwd { .. }, Self::Large) => {
                (ResourceType::Bmp, (110, 170))
            }
            (Game::Bgee | Game::BgeeSod | Game::Bg2ee | Game::Eet | Game::Iwdee, Self::Small) => {
                (ResourceType::Bmp, (169, 266))
            }
            (Game::Bgee | Game::BgeeSod | Game::Bg2ee | Game::Eet | Game::Iwd2, Self::Large) => {
                (ResourceType::Bmp, (210, 330))
            }
            (Game::Iwd { .. }, Self::Small) => (ResourceType::Bmp, (36, 58)),
            (Game::Iwdee, Self::Large) => (ResourceType::Bmp, (420, 660)),
            (Game::Iwd2, Self::Small) => (ResourceType::Bmp, (42, 42)),
            (Game::Pst | Game::Pstee, Self::Small) => (ResourceType::Bmp, (32, 32)),
            (Game::Pst | Game::Pstee, Self::Large) => (ResourceType::Bam, (65, 51)),
        };
        let suffixes: &[char] = match game {
            Game::Pst | Game::Pstee => &[],
            _ => &['S', 'M', 'L'],
        };
        PortraitFormat {
            r#type,
            width,
            height,
            suffixes,
        }
    }
}

/// Every picture the game has for `slot`, by name: upper-case, as creatures
/// name them, and sorted. Only the header of each resource named like a
/// portrait is read.
pub fn portrait_names(game_data: &GameData, slot: PortraitSlot) -> Vec<String> {
    let format = slot.format(game_data.game());
    let mut names: Vec<String> = game_data
        .get_all_resources_by_type(format.r#type)
        .filter(|resource| format.fits_name(&resource.name))
        .filter(|resource| {
            let Some(source) = resource.datasource.as_ref() else {
                return false;
            };
            let size = match format.r#type {
                ResourceType::Bam => bam::first_frame_size(source).ok().flatten(),
                _ => bmp::dimensions(source).ok(),
            };
            size == Some((format.width, format.height))
        })
        .map(|resource| resource.name.to_ascii_uppercase())
        .collect();
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use infinitier_datasource::DataSource;
    use infinitier_fs::CaseInsensitiveFS;
    use infinitier_test_utils::constants::BG2_RESOURCES_DIR;
    use infinitier_test_utils::get_assets_path;

    use super::*;
    use crate::game::{DataOrigin, GameDataBuilder, GameResource};

    /// The start of a BMP that declares itself `width × height`.
    fn bmp(width: i32, height: i32) -> DataSource {
        let mut header = vec![0; 26];
        header[..2].copy_from_slice(b"BM");
        header[14] = 40;
        header[18..22].copy_from_slice(&width.to_le_bytes());
        header[22..26].copy_from_slice(&height.to_le_bytes());
        DataSource::new(header)
    }

    fn resource(name: &str, source: DataSource) -> GameResource {
        GameResource {
            game_type: Game::Bg2ee,
            name: name.to_owned(),
            r#type: ResourceType::Bmp,
            file_size: None,
            datasource: Some(source),
            data_origin: DataOrigin::Missing,
            imported: None,
        }
    }

    #[test]
    fn a_slot_lists_the_pictures_of_its_size_by_name() {
        let game_data = GameData::new(
            vec![
                resource("nminscm", bmp(169, 266)),
                resource("ajantism", bmp(169, 266)),
                resource("nminscl", bmp(210, 330)),
                resource("ar0300lm", bmp(320, 320)),
                resource("broken", DataSource::new(b"not a picture".to_vec())),
                // A portrait's size, not a portrait's name: the credits' art.
                resource("credi_01", bmp(210, 330)),
            ],
            Game::Bg2ee,
            CaseInsensitiveFS::empty(),
        );
        assert_eq!(
            portrait_names(&game_data, PortraitSlot::Small),
            ["AJANTISM", "NMINSCM"]
        );
        assert_eq!(portrait_names(&game_data, PortraitSlot::Large), ["NMINSCL"]);
    }

    #[test]
    fn a_portrait_ends_in_its_size_but_in_pst() {
        let bg2ee = PortraitSlot::Small.format(Game::Bg2ee);
        for name in ["NMINSCS", "nminscm", "NMINSCL", "HMF_S"] {
            assert!(bg2ee.fits_name(name), "{name}");
        }
        for name in ["CREDI_01", "NOPORTLG", "TESTPOR", ""] {
            assert!(!bg2ee.fits_name(name), "{name}");
        }

        let pst = GameData::new(
            vec![resource("ampnm1", bmp(32, 32))],
            Game::Pst,
            CaseInsensitiveFS::empty(),
        );
        assert_eq!(portrait_names(&pst, PortraitSlot::Small), ["AMPNM1"]);
    }

    /// An EE game's `portraits` folder is in the user's documents, an overlay
    /// root over the install.
    #[test]
    fn the_players_own_portraits_are_offered() {
        let documents = tempfile::tempdir().unwrap();
        let portraits = documents.path().join("Portraits");
        std::fs::create_dir(&portraits).unwrap();
        // A 169×266 picture: an EE small portrait.
        std::fs::copy(
            get_assets_path().join("BMP/MINSCM.BMP"),
            portraits.join("MyHeroM.bmp"),
        )
        .unwrap();
        std::fs::write(portraits.join("readme.txt"), "not a picture").unwrap();
        let install = get_assets_path().join("KEY").join(BG2_RESOURCES_DIR.0);

        let game_data = GameDataBuilder::new(
            [install, documents.path().to_owned()].as_slice(),
            Some(Game::Bg2ee),
        )
        .unwrap()
        .build()
        .unwrap();

        let resource = game_data
            .get_by_name_and_type("myherom", ResourceType::Bmp)
            .unwrap();
        assert!(
            matches!(&resource.data_origin, DataOrigin::Dir { name, .. } if name == "portraits"),
            "{:?}",
            resource.data_origin
        );
        assert!(portrait_names(&game_data, PortraitSlot::Small).contains(&"MYHEROM".to_owned()));
        assert!(
            game_data
                .get_by_name_and_type("readme", ResourceType::Unknown(0))
                .is_none()
        );
    }
}
