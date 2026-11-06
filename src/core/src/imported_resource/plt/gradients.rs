//! The colour gradients a creature's colours select.

use std::borrow::Cow;

use infinitier_common::ResourceType;

use crate::game::GameData;
use crate::imported_resource::ImportedResource;

/// A game's colour gradients: one row of shades per colour index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gradients {
    ramps: Vec<Vec<[u8; 3]>>,
}

impl Gradients {
    /// The 12-shade gradients, brightest first, that sprite palettes and
    /// colour swatches use: `RANGES12.BMP`, else `MPALETTE.BMP`.
    pub fn load_coarse(game_data: &GameData) -> Option<Self> {
        Self::load_first(game_data, &["RANGES12", "MPALETTE"])
    }

    /// The 256-shade gradients, darkest first, that PLTs index:
    /// `MPAL256.BMP`.
    pub fn load_fine(game_data: &GameData) -> Option<Self> {
        Self::load_first(game_data, &["MPAL256"])
    }

    /// Gradients made of these rows of shades.
    pub fn from_ramps(ramps: Vec<Vec<[u8; 3]>>) -> Self {
        Self { ramps }
    }

    /// The first of these BMPs the game has, a row per gradient.
    fn load_first(game_data: &GameData, names: &[&str]) -> Option<Self> {
        let image = names.iter().find_map(|name| {
            match game_data.import_by_name_and_type(name, ResourceType::Bmp) {
                Ok(Cow::Borrowed(ImportedResource::Image(image))) => Some(Cow::Borrowed(image)),
                Ok(Cow::Owned(ImportedResource::Image(image))) => Some(Cow::Owned(image)),
                _ => None,
            }
        })?;
        let ramps = image
            .image
            .rows()
            .map(|row| {
                row.map(|pixel| {
                    let [r, g, b, _] = pixel.0;
                    [r, g, b]
                })
                .collect()
            })
            .collect();
        Some(Self { ramps })
    }

    /// How many gradients there are: colours `0..len()` have one, except
    /// for any empty row.
    pub fn len(&self) -> usize {
        self.ramps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ramps.is_empty()
    }

    /// The gradient colour `index` selects. `None` for an index the game has
    /// no gradient for.
    pub fn ramp(&self, index: u8) -> Option<&[[u8; 3]]> {
        self.ramps
            .get(usize::from(index))
            .map(Vec::as_slice)
            .filter(|ramp| !ramp.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_selects_its_row() {
        let gradients = Gradients::from_ramps(vec![vec![[1, 2, 3]], vec![[4, 5, 6], [7, 8, 9]]]);
        assert_eq!(gradients.ramp(1), Some([[4, 5, 6], [7, 8, 9]].as_slice()));
    }

    #[test]
    fn an_unknown_or_empty_row_is_no_gradient() {
        let gradients = Gradients::from_ramps(vec![vec![]]);
        assert_eq!(gradients.ramp(0), None);
        assert_eq!(gradients.ramp(1), None);
    }
}
