//! Which art makes up a creature's paperdoll.
//!
//! It depends on the game:
//! - BG, BG2, IWD and their Enhanced Editions: one doll per animation and
//!   armour level, named by GemRB's `pdolls.2da` (a PLT or a paperdoll BAM),
//!   with the held weapon, the shield or off-hand weapon and the helmet drawn
//!   over it as the inventory screen does: `WP` + the doll's size + the
//!   item's animation code + `INV` (`OIN` for an off-hand weapon).
//! - PST and PST:EE: a painted picture per character, named by `pdolls.2da`.
//! - IWD2 has no paperdolls: its standing sprite, named by `avatars.2da`.

use infinitier_common::{Game, ResourceType};
use infinitier_datasource::Importer;
use infinitier_two_da_resource::{TwoDA, TwoDAImporter};

use super::art::Art;
use crate::game::GameData;
use crate::imported_resource::ImportedResource;
use crate::imported_resource::cre::ImportedCre;
use crate::unhardcoded::unhardcoded_resource;

/// The item slots the paperdoll shows, in the BG / BG2 / IWD layout (IWD2's
/// helmet and armour slots are the same).
const HELMET_SLOT: usize = 0;
const ARMOR_SLOT: usize = 1;
const SHIELD_SLOT: usize = 2;
const FIRST_WEAPON_SLOT: usize = 9;
const WEAPON_SLOTS: usize = 4;
const FIRST_QUIVER_SLOT: usize = 13;
const QUIVER_SLOTS: usize = 4;
/// A selected launcher is recorded as its ammunition's quiver, counted from
/// this slot: -24 the first quiver, -23 the second.
const QUIVER_SELECTION_BASE: i16 = 37;

/// Item types (`ITM` offset 0x1C), as IESDP numbers them.
const ARROWS: u16 = 5;
const BULLETS: u16 = 14;
const BOW: u16 = 15;
const SLING: u16 = 18;
const CROSSBOW: u16 = 27;
const BOLTS: u16 = 31;
const SHIELD_TYPES: [u16; 5] = [12, 41, 47, 49, 53];

/// A sprite's `G1` cycle where it stands, facing the viewer.
const SPRITE_STANDING_CYCLE: usize = 9;

/// The highest armour level a doll has art for.
const MAX_ARMOR_LEVEL: u8 = 4;

/// A creature's paperdoll art.
pub(super) struct Parts {
    /// The doll bare, as character generation shows it: the base armour
    /// level.
    pub(super) bare: Art,
    /// The doll in the armour the creature wears, when that's another level.
    pub(super) armored: Option<Art>,
    /// What it holds and wears over its armour, in the order the game draws
    /// them: weapon, off hand, helmet.
    pub(super) equipment: Vec<Art>,
}

/// A creature's paperdoll. `None` when its animation has no doll: a creature
/// that isn't humanoid, a polymorph.
pub(super) fn art(game_data: &GameData, cre: &ImportedCre) -> Option<Parts> {
    let game = game_data.game();
    let animation = cre.animation_id();
    match game {
        Game::Pst | Game::Pstee => {
            let table = table(Game::Pst, "pdolls")?;
            let row = format!("0x{:02X}", animation & 0xFF);
            Some(Parts {
                bare: picture(game_data, cell(&table, &row, "INVENTORY")?)?,
                armored: None,
                equipment: Vec::new(),
            })
        }
        Game::Iwd2 => {
            let table = table(game, "avatars")?;
            let row = format!("0x{animation:04X}");
            let (bare, armored) = bare_and_armored(worn_armor_level(game_data, cre), |level| {
                let prefix = cell(&table, &row, &format!("AT_{level}"))?;
                sprite(game_data, &format!("{prefix}G1"))
            })?;
            Some(Parts {
                bare,
                armored,
                equipment: Vec::new(),
            })
        }
        // No table of its own: the same art as BG2:EE.
        Game::Iwdee => paperdoll(game_data, cre, Game::Bg2ee, animation),
        Game::Bg
        | Game::Bg2
        | Game::Tutu
        | Game::Bgee
        | Game::BgeeSod
        | Game::Bg2ee
        | Game::Eet
        | Game::Iwd { .. } => paperdoll(game_data, cre, game, animation),
    }
}

/// The animation's doll, bare and in the armour the creature wears, from
/// `table_game`'s `pdolls.2da`, and its equipment.
fn paperdoll(
    game_data: &GameData,
    cre: &ImportedCre,
    table_game: Game,
    animation: u32,
) -> Option<Parts> {
    let table = table(table_game, "pdolls")?;
    let row = format!("0x{animation:04X}");
    let (bare, armored) = bare_and_armored(worn_armor_level(game_data, cre), |level| {
        doll_art(game_data, cell(&table, &row, &format!("LEVEL{level}"))?)
    })?;
    let equipment = cell(&table, &row, "SIZE")
        .map(|size| equipment(game_data, cre, size))
        .unwrap_or_default();
    Some(Parts {
        bare,
        armored,
        equipment,
    })
}

/// A doll's art bare and in the armour worn: the plainest level with art,
/// and the worn level or, without art for it, the nearest plainer one. No
/// armoured art when that's the bare one. `None` when no level has art.
fn bare_and_armored<T>(worn: u8, art_at: impl Fn(u8) -> Option<T>) -> Option<(T, Option<T>)> {
    let (bare_level, bare) =
        (1..=MAX_ARMOR_LEVEL).find_map(|level| Some((level, art_at(level)?)))?;
    let armored = (bare_level + 1..=worn.min(MAX_ARMOR_LEVEL))
        .rev()
        .find_map(art_at);
    Some((bare, armored))
}

/// What the creature holds and wears over its armour, in drawing order, where
/// the game has art for it.
fn equipment(game_data: &GameData, cre: &ImportedCre, size: &str) -> Vec<Art> {
    let item = |slot: usize| held(game_data, cre, slot);
    let weapon = cre
        .selected_weapon()
        .and_then(|selected| weapon_slot(selected, |slot| item(slot).map(|held| held.item_type)))
        .and_then(item)
        .and_then(|held| item_art(size, held.animation, MAIN_HAND));
    let off_hand = item(SHIELD_SLOT)
        .and_then(|held| item_art(size, held.animation, off_hand_suffix(held.item_type)));
    let helmet = item(HELMET_SLOT).and_then(|held| item_art(size, held.animation, MAIN_HAND));
    [weapon, off_hand, helmet]
        .into_iter()
        .flatten()
        .filter_map(|resref| doll_art(game_data, &resref))
        .collect()
}

/// What the paperdoll needs of an item.
struct Held {
    item_type: u16,
    animation: [u8; 2],
}

/// The item in `slot`, if the game has it.
fn held(game_data: &GameData, cre: &ImportedCre, slot: usize) -> Option<Held> {
    let item = cre.item_in_slot(slot)?;
    let itm = game_data.import_itm_by_name(&item.item).ok()?;
    Some(Held {
        item_type: itm.header.item_type(),
        animation: itm.header.item_animation(),
    })
}

/// The slot of the weapon in hand, from the selected-weapon entry: one of the
/// quick weapons, or, when it names a quiver, the launcher for its ammunition.
/// `None` for fists or anything else. `item_type` gives the type of the item
/// in a slot.
fn weapon_slot(selected: i16, item_type: impl Fn(usize) -> Option<u16>) -> Option<usize> {
    let weapons = FIRST_WEAPON_SLOT..FIRST_WEAPON_SLOT + WEAPON_SLOTS;
    let quivers = FIRST_QUIVER_SLOT..FIRST_QUIVER_SLOT + QUIVER_SLOTS;
    if let Ok(quick) = usize::try_from(selected) {
        return weapons.clone().nth(quick);
    }
    let quiver = usize::try_from(selected.checked_add(QUIVER_SELECTION_BASE)?).ok()?;
    if !quivers.contains(&quiver) {
        return None;
    }
    let launcher = launcher_for(item_type(quiver)?)?;
    weapons
        .into_iter()
        .find(|slot| item_type(*slot) == Some(launcher))
}

/// The launcher that shoots this ammunition.
fn launcher_for(ammunition: u16) -> Option<u16> {
    match ammunition {
        ARROWS => Some(BOW),
        BOLTS => Some(CROSSBOW),
        BULLETS => Some(SLING),
        _ => None,
    }
}

/// How an item's art is named in the main hand, and on the head.
const MAIN_HAND: &str = "INV";
/// How a weapon's art is named in the off hand.
const OFF_HAND: &str = "OIN";

/// The off-hand slot holds a shield or a second weapon, each with its own art.
fn off_hand_suffix(item_type: u16) -> &'static str {
    if SHIELD_TYPES.contains(&item_type) {
        MAIN_HAND
    } else {
        OFF_HAND
    }
}

/// The resref of an item's art on a doll of this size. `None` when the item
/// has no animation code.
fn item_art(size: &str, animation: [u8; 2], suffix: &str) -> Option<String> {
    let [first, second] = animation;
    let code = [first, second]
        .iter()
        .all(u8::is_ascii_alphanumeric)
        .then(|| format!("{}{}", char::from(first), char::from(second)))?;
    Some(format!("WP{size}{code}{suffix}"))
}

/// The level, from 1, of the armour the creature wears.
fn worn_armor_level(game_data: &GameData, cre: &ImportedCre) -> u8 {
    held(game_data, cre, ARMOR_SLOT).map_or(1, |armor| armor_level(armor.animation))
}

/// An armour's animation code starts with its level (`2A` leather, `4A`
/// plate, `3W` a robe); anything else is the base level.
fn armor_level([first, _]: [u8; 2]) -> u8 {
    match first {
        b'1'..=b'4' => first - b'0',
        _ => 1,
    }
}

/// One of GemRB's embedded tables. Read directly because no game ships these,
/// and IWD:EE and PST:EE borrow another game's.
fn table(game: Game, name: &str) -> Option<TwoDA> {
    let resource = unhardcoded_resource(game, name, ResourceType::TwoDA)?;
    TwoDAImporter { name }.import(&resource.data_source()).ok()
}

/// A table's value, `None` for its `*` placeholder.
fn cell<'a>(table: &'a TwoDA, row: &str, column: &str) -> Option<&'a str> {
    let index = table
        .headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(column))?;
    let value = table.rows.get(row)?.get(index)?;
    (value != "*").then_some(value.as_str())
}

/// The doll art `resref`: its PLT, else its paperdoll BAM. Not what the
/// Enhanced Editions mostly keep under a doll's PLT name: the same small
/// placeholder BAM (a heart) for every doll, whose art is in its BAM.
fn doll_art(game_data: &GameData, resref: &str) -> Option<Art> {
    let source = |r#type| {
        game_data
            .get_by_name_and_type(resref, r#type)?
            .datasource
            .as_ref()
    };
    source(ResourceType::Plt)
        .and_then(|source| Art::plt(resref, source).ok())
        .or_else(|| Art::halves(resref, source(ResourceType::Bam)?).ok())
}

/// The sprite BAM `resref`, standing.
fn sprite(game_data: &GameData, resref: &str) -> Option<Art> {
    let source = game_data
        .get_by_name_and_type(resref, ResourceType::Bam)?
        .datasource
        .as_ref()?;
    Art::sprite(resref, source, SPRITE_STANDING_CYCLE).ok()
}

/// The painted BMP `resref`.
fn picture(game_data: &GameData, resref: &str) -> Option<Art> {
    let imported = game_data
        .import_by_name_and_type(resref, ResourceType::Bmp)
        .ok()?;
    match imported.as_ref() {
        ImportedResource::Image(image) => Some(Art::picture(&image.image)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use infinitier_datasource::DataSource;
    use infinitier_fs::CaseInsensitiveFS;
    use infinitier_test_utils::get_assets_path;

    use super::*;
    use crate::game::{DataOrigin, GameResource};

    /// A game holding these `(name, type, bytes)` resources.
    fn game_with(resources: Vec<(&str, ResourceType, Vec<u8>)>) -> GameData {
        let resources = resources
            .into_iter()
            .map(|(name, r#type, bytes)| GameResource {
                game_type: Game::Bg2ee,
                name: name.to_owned(),
                r#type,
                file_size: Some(bytes.len() as u64),
                datasource: Some(DataSource::new(bytes)),
                data_origin: DataOrigin::Missing,
                imported: None,
            })
            .collect();
        GameData::new(resources, Game::Bg2ee, CaseInsensitiveFS::empty())
    }

    #[test]
    fn a_bam_under_a_dolls_plt_name_is_passed_over_for_its_bam() -> std::io::Result<()> {
        let bam = std::fs::read(get_assets_path().join("BAM/V1/01/1chan03B_compressed.BAM"))?;
        let placeholder_only = game_with(vec![("chmf1inv", ResourceType::Plt, bam.clone())]);
        assert!(doll_art(&placeholder_only, "CHMF1INV").is_none());

        let with_bam = game_with(vec![
            ("chmf1inv", ResourceType::Plt, bam.clone()),
            ("chmf1inv", ResourceType::Bam, bam),
        ]);
        assert!(matches!(
            doll_art(&with_bam, "CHMF1INV"),
            Some(Art::Halves { .. })
        ));
        Ok(())
    }

    /// `weapon_slot` over these slots' item types.
    fn weapon_in(selected: i16, types: &[(usize, u16)]) -> Option<usize> {
        let types: HashMap<usize, u16> = types.iter().copied().collect();
        weapon_slot(selected, |slot| types.get(&slot).copied())
    }

    #[test]
    fn a_quick_weapon_is_selected_by_its_place() {
        assert_eq!(weapon_in(0, &[]), Some(FIRST_WEAPON_SLOT));
        assert_eq!(weapon_in(3, &[]), Some(FIRST_WEAPON_SLOT + 3));
        // Fists, and past the quick weapons.
        assert_eq!(weapon_in(1000, &[]), None);
        assert_eq!(weapon_in(4, &[]), None);
    }

    #[test]
    fn a_selected_quiver_means_its_launcher() {
        // Arrows in the first quiver; a sword, then a bow in the quick slots.
        let slots = [(13, ARROWS), (9, 20), (10, BOW)];
        assert_eq!(weapon_in(-24, &slots), Some(10));
        // Bullets in the second quiver.
        let slots = [(14, BULLETS), (9, BOW), (11, SLING)];
        assert_eq!(weapon_in(-23, &slots), Some(11));
        // Bolts.
        let slots = [(13, BOLTS), (12, CROSSBOW)];
        assert_eq!(weapon_in(-24, &slots), Some(12));
    }

    #[test]
    fn a_quiver_without_its_launcher_means_no_weapon() {
        assert_eq!(weapon_in(-24, &[(13, ARROWS), (9, SLING)]), None);
        // An empty quiver, something that isn't ammunition, no quiver at all.
        assert_eq!(weapon_in(-24, &[(9, BOW)]), None);
        assert_eq!(weapon_in(-24, &[(13, 20), (9, BOW)]), None);
        assert_eq!(weapon_in(-30, &[(9, BOW)]), None);
        assert_eq!(weapon_in(i16::MIN, &[]), None);
    }

    #[test]
    fn a_shield_and_an_off_hand_weapon_have_their_own_art() {
        assert_eq!(off_hand_suffix(12), MAIN_HAND);
        assert_eq!(off_hand_suffix(49), MAIN_HAND);
        assert_eq!(off_hand_suffix(20), OFF_HAND);
    }

    #[test]
    fn item_art_is_named_by_size_and_code() {
        assert_eq!(
            item_art("L", *b"S1", MAIN_HAND).as_deref(),
            Some("WPLS1INV")
        );
        assert_eq!(item_art("M", *b"WH", OFF_HAND).as_deref(), Some("WPMWHOIN"));
        assert_eq!(item_art("N", *b"  ", MAIN_HAND), None);
        assert_eq!(item_art("N", [0, 0], MAIN_HAND), None);
    }

    /// `bare_and_armored` for art at these levels: the level itself.
    fn levels(worn: u8, with_art: &[u8]) -> Option<(u8, Option<u8>)> {
        bare_and_armored(worn, |level| with_art.contains(&level).then_some(level))
    }

    #[test]
    fn a_doll_is_bare_at_the_base_level_and_armoured_at_the_worn_one() {
        let all = [1, 2, 3, 4];
        assert_eq!(levels(4, &all), Some((1, Some(4))));
        assert_eq!(levels(2, &all), Some((1, Some(2))));
        // No armour, or past the levels there's art for.
        assert_eq!(levels(1, &all), Some((1, None)));
        assert_eq!(levels(9, &all), Some((1, Some(4))));
    }

    #[test]
    fn missing_art_falls_back_to_the_nearest_plainer_level() {
        assert_eq!(levels(4, &[1, 2]), Some((1, Some(2))));
        // Without a base level, the plainest there is is bare.
        assert_eq!(levels(4, &[2, 4]), Some((2, Some(4))));
        // Only one level: bare, nothing armoured.
        assert_eq!(levels(4, &[3]), Some((3, None)));
        assert_eq!(levels(2, &[]), None);
    }

    #[test]
    fn armor_level_comes_from_the_animation_code() {
        assert_eq!(armor_level(*b"2A"), 2);
        assert_eq!(armor_level(*b"4A"), 4);
        assert_eq!(armor_level(*b"3W"), 3);
        // A shield, a helmet, no animation.
        assert_eq!(armor_level(*b"D1"), 1);
        assert_eq!(armor_level(*b"H0"), 1);
        assert_eq!(armor_level(*b"  "), 1);
        assert_eq!(armor_level(*b"5A"), 1);
    }

    #[test]
    fn every_game_with_paperdolls_names_the_fighter_and_its_size() {
        for game in [
            Game::Bg,
            Game::Bg2,
            Game::Tutu,
            Game::Bgee,
            Game::BgeeSod,
            Game::Bg2ee,
            Game::Eet,
        ] {
            let table = table(game, "pdolls").expect("the game has a doll table");
            assert_eq!(cell(&table, "0x6100", "LEVEL1"), Some("CHMF1INV"));
            assert_eq!(cell(&table, "0x6100", "LEVEL4"), Some("CHMF4INV"));
            assert_eq!(cell(&table, "0x6100", "SIZE"), Some("L"));
        }
    }

    #[test]
    fn pst_names_its_pictures_by_the_animations_low_byte() {
        let table = table(Game::Pst, "pdolls").expect("PST has a doll table");
        assert_eq!(cell(&table, "0x2E", "INVENTORY"), Some("IVPMRT"));
    }

    #[test]
    fn iwd2_names_its_sprites_by_animation_and_armor_level() {
        let table = table(Game::Iwd2, "avatars").expect("IWD2 has an avatar table");
        assert_eq!(cell(&table, "0x6100", "AT_1"), Some("CHMB1"));
        assert_eq!(cell(&table, "0x6100", "AT_4"), Some("CHMF4"));
    }

    #[test]
    fn a_table_placeholder_is_no_value() {
        let table = TwoDA {
            headers: vec!["LEVEL1".to_owned()],
            default: "*".to_owned(),
            rows: [("0x1".to_owned(), vec!["*".to_owned()])].into(),
        };
        assert_eq!(cell(&table, "0x1", "LEVEL1"), None);
        assert_eq!(cell(&table, "0x2", "LEVEL1"), None);
        assert_eq!(cell(&table, "0x1", "LEVEL2"), None);
    }
}
