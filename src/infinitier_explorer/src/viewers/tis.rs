//! TIS tilesets, modelled on NearInfinity's `TisResource` panel: a "tiles per
//! row" slider and a "Show grid" toggle over the composite image.
//!
//! The slider starts at [`ImportedTis::expected_columns`]: from the area's
//! WED when one is registered, otherwise NearInfinity's `min(tile_count, 5)`.

use std::ops::RangeInclusive;

use infinitier_core::{
    game::GameResource, imported_resource::tis::ImportedTis, resource::tis::Type as TisType,
};
use slint::Image;

use super::image::rgba_image;
use super::info::{origin_label, size_label};

/// The largest composite side, in pixels: the software renderer has no
/// texture limit, but this keeps a composite's memory bounded.
const MAX_IMAGE_SIDE: u32 = 8192;

pub struct TisViewer {
    tis: ImportedTis,
    image: Image,
    image_size: (u32, u32),
    /// Tiles per row; always inside `range`.
    columns: u32,
    /// The columns that keep the composite within `MAX_IMAGE_SIDE` on
    /// both axes.
    range: RangeInclusive<u32>,
    /// Tiles per axis that fit in `MAX_IMAGE_SIDE`.
    tiles_per_axis: u32,
    /// The tileset is so large (more than `tiles_per_axis²` tiles) that even
    /// the squarest arrangement overflows, so only the leading slice that
    /// fits is shown.
    truncated: bool,
    show_grid: bool,
}

impl TisViewer {
    pub fn new(tis: ImportedTis) -> Self {
        let limits = ColumnLimits::new(tis.tile_count, tis.tile_dimension, MAX_IMAGE_SIDE);
        let columns = tis
            .expected_columns
            .clamp(*limits.range.start(), *limits.range.end());
        let mut viewer = Self {
            tis,
            image: Image::default(),
            image_size: (0, 0),
            columns,
            range: limits.range,
            tiles_per_axis: limits.tiles_per_axis,
            truncated: limits.truncated,
            show_grid: false,
        };
        viewer.compose();
        viewer
    }

    /// Composite the tiles `columns` per row.
    fn compose(&mut self) {
        // The truncated path renders only the leading slice of tiles that
        // fits (see `ColumnLimits`).
        let image = if self.truncated {
            let visible = (self.columns as usize) * (self.tiles_per_axis as usize);
            let count = self.tis.tile_count.min(visible);
            let stride =
                (self.tis.tile_dimension as usize) * (self.tis.tile_dimension as usize) * 4;
            let view = ImportedTis {
                tile_dimension: self.tis.tile_dimension,
                tile_count: count,
                tile_pixels: self.tis.tile_pixels[..count * stride].to_vec(),
                expected_columns: self.tis.expected_columns,
                variant: self.tis.variant,
            };
            view.compose(self.columns)
        } else {
            self.tis.compose(self.columns)
        };
        self.image_size = (image.width(), image.height());
        self.image = rgba_image(image.as_raw(), image.width(), image.height());
    }

    pub fn set_columns(&mut self, columns: u32) {
        let columns = columns.clamp(*self.range.start(), *self.range.end());
        if columns != self.columns {
            self.columns = columns;
            self.compose();
        }
    }

    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_size(&self) -> (u32, u32) {
        self.image_size
    }

    pub fn tile_dimension(&self) -> u32 {
        self.tis.tile_dimension
    }

    pub fn columns(&self) -> u32 {
        self.columns
    }

    pub fn range(&self) -> &RangeInclusive<u32> {
        &self.range
    }

    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }

    pub fn info(&self, resource: &GameResource) -> Vec<String> {
        let (width, height) = self.image_size;
        let variant = match self.tis.variant {
            TisType::Palette => "Palette",
            TisType::Pvrz => "PVRZ",
        };
        let grid = if self.truncated {
            // The grid shown is the first `columns × tiles_per_axis` tiles;
            // the rest don't fit.
            let visible = self.columns * self.tiles_per_axis;
            format!(
                "{} × {} tiles (showing first {} of {})",
                self.columns,
                self.tiles_per_axis,
                visible.min(self.tis.tile_count as u32),
                self.tis.tile_count,
            )
        } else {
            let rows = (self.tis.tile_count as u32).div_ceil(self.columns);
            format!("{} × {rows} tiles", self.columns)
        };
        vec![
            format!("{width} × {height} px"),
            size_label(resource),
            "TIS".to_owned(),
            variant.to_owned(),
            format!("{} tiles", self.tis.tile_count),
            grid,
            origin_label(resource),
        ]
    }
}

/// Composite-image limits derived from the largest image side and the TIS
/// tile size.
struct ColumnLimits {
    /// Tiles per axis that fit in one image.
    tiles_per_axis: u32,
    /// Inclusive bounds the slider must stay within.
    range: RangeInclusive<u32>,
    /// Not every tile fits — see [`TisViewer::truncated`].
    truncated: bool,
}

impl ColumnLimits {
    /// The `min..=max` columns that keep both `columns * tile_dim` (width)
    /// and `rows * tile_dim` (height) within `max_side`. When the tileset is
    /// so large that no value satisfies both, falls back to the widest
    /// fitting grid and flags the result as truncated.
    fn new(tile_count: usize, tile_dim: u32, max_side: u32) -> Self {
        let tile_count = tile_count.max(1) as u32;
        let tiles_per_axis = (max_side / tile_dim.max(1)).max(1);

        // Width:  columns ≤ tiles_per_axis           ⇒ width ≤ max_side
        // Height: rows = ceil(tile_count / columns) ≤ tiles_per_axis
        //         ⇒ columns ≥ ceil(tile_count / tiles_per_axis)
        let max_cols = tile_count.min(tiles_per_axis);
        let min_cols = tile_count.div_ceil(tiles_per_axis);

        if min_cols > max_cols {
            // tile_count > tiles_per_axis²: pin to the widest fitting grid,
            // which shows only the first `tiles_per_axis²` tiles.
            ColumnLimits {
                tiles_per_axis,
                range: max_cols..=max_cols,
                truncated: true,
            }
        } else {
            ColumnLimits {
                tiles_per_axis,
                range: min_cols.max(1)..=max_cols.max(1),
                truncated: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_limits_small_tileset_fits_with_no_min_constraint() {
        // 6 tiles, 64 px tiles, 8192 px max → tiles_per_axis = 128.
        // ceil(6 / 128) = 1, so slider can go from 1 to 6.
        let l = ColumnLimits::new(6, 64, 8192);
        assert_eq!(l.tiles_per_axis, 128);
        assert_eq!(l.range, 1..=6);
        assert!(!l.truncated);
    }

    #[test]
    fn column_limits_medium_tileset_enforces_min() {
        // 2400 tiles, 64 px tiles, 8192 px max → tiles_per_axis = 128.
        // ceil(2400 / 128) = 19, so the slider can't go below 19 (else
        // rows × 64 would exceed 8192).
        let l = ColumnLimits::new(2400, 64, 8192);
        assert_eq!(l.tiles_per_axis, 128);
        assert_eq!(*l.range.start(), 19);
        assert_eq!(*l.range.end(), 128);
        assert!(!l.truncated);
    }

    #[test]
    fn column_limits_too_large_tileset_truncates() {
        // tiles_per_axis = 128, tile_count = 20_000 > 128² = 16_384.
        // No columns value lets every tile fit — collapse the slider to the
        // widest fitting grid and flag truncation.
        let l = ColumnLimits::new(20_000, 64, 8192);
        assert_eq!(l.tiles_per_axis, 128);
        assert_eq!(l.range, 128..=128);
        assert!(l.truncated);
    }

    #[test]
    fn column_limits_clamps_to_at_least_one() {
        // Pathological inputs (zero tiles or absurdly small limits) must not
        // produce an empty slider range.
        let l = ColumnLimits::new(0, 64, 8192);
        assert_eq!(l.range, 1..=1);

        let tiny = ColumnLimits::new(10, 64, 32); // max_side < tile_dim
        assert_eq!(tiny.tiles_per_axis, 1);
        assert!(*tiny.range.start() >= 1);
    }

    #[test]
    fn column_limits_minimum_keeps_the_height_in_bounds() {
        // A 2.4k-ish tileset: at the slider's minimum the composite must
        // still be at most 8192 px tall.
        let tile_count = 2400usize;
        let l = ColumnLimits::new(tile_count, 64, 8192);
        let min_cols = *l.range.start();
        let height_at_min = (tile_count as u32).div_ceil(min_cols) * 64;
        assert!(
            height_at_min <= 8192,
            "min_cols={min_cols} ⇒ height={height_at_min} (should be ≤ 8192)"
        );
    }
}
