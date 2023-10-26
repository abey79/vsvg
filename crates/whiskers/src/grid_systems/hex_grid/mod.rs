use vsvg::Point;

use crate::Sketch;

use self::cell::*;

pub mod cell;

/// Hexagonal grid module
///
/// Implementation based on Amit Patel's [hex grid reference](https://www.redblobgames.com/grids/hexagons)
///
/// ```rust
/// HexGrid::with_flat_orientation()
///     .cell_size(30.0)
///     .columns(20)
///     .rows(40)
///     .spacing([10.0, 20.0])
///     .build(sketch, |sketch, hex_grid_cell| {
///         sketch.add_path(hex_grid_cell);
///     })
/// ```
pub struct HexGrid {
    orientation: Orientation,
    dimensions: [usize; 2],
    gutter: [f64; 2],
    cell_size: f64,
}

impl HexGrid {
    const DEFAULT_DIMENSIONS: [usize; 2] = [4, 4];
    const DEFAULT_GUTTER: [f64; 2] = [0.0, 0.0];
    const DEFAULT_CELL_SIZE: f64 = 10.0;

    /// Creates grid instance with flat-top orientation
    #[must_use]
    pub fn with_flat_orientation() -> Self {
        Self {
            orientation: Orientation::Flat,
            dimensions: HexGrid::DEFAULT_DIMENSIONS,
            gutter: HexGrid::DEFAULT_GUTTER,
            cell_size: HexGrid::DEFAULT_CELL_SIZE,
        }
    }

    /// Creates grid instance with pointy-top orientation
    #[must_use]
    pub fn with_pointy_orientation() -> Self {
        Self {
            orientation: Orientation::Pointy,
            dimensions: HexGrid::DEFAULT_DIMENSIONS,
            gutter: HexGrid::DEFAULT_GUTTER,
            cell_size: HexGrid::DEFAULT_CELL_SIZE,
        }
    }

    /// Overrides current columns value
    #[must_use]
    pub fn columns(mut self, value: usize) -> Self {
        self.dimensions[0] = value;
        self
    }

    /// Overrides current rows value
    #[must_use]
    pub fn rows(mut self, value: usize) -> Self {
        self.dimensions[1] = value;
        self
    }

    #[must_use]
    pub fn spacing(mut self, value: [f64; 2]) -> Self {
        self.gutter = value;
        self
    }

    #[must_use]
    pub fn horizontal_spacing(mut self, value: f64) -> Self {
        self.gutter[0] = value;
        self
    }

    #[must_use]
    pub fn vertical_spacing(mut self, value: f64) -> Self {
        self.gutter[1] = value;
        self
    }

    #[must_use]
    pub fn cell_size(mut self, value: f64) -> Self {
        self.cell_size = value;
        self
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn build(
        self,
        sketch: &mut Sketch,
        callback_fn: impl FnOnce(&mut Sketch, &HexGridCell) + Copy,
    ) {
        let [columns, rows] = self.dimensions;

        for row in 0..rows {
            for column in 0..columns {
                let horiz: f64;
                let vert: f64;
                let is_even_col = column % 2 == 0;
                let is_even_row = row % 2 == 0;
                let cell: HexGridCell;
                let x: f64;
                let y: f64;
                let gutter_x = self.gutter[0] * column as f64;
                let gutter_y = self.gutter[1] * row as f64;
                match self.orientation {
                    Orientation::Flat => {
                        horiz = 1.5 * self.cell_size;
                        vert = (3.0 as f64).sqrt() * self.cell_size;

                        x = horiz * column as f64 + gutter_x;
                        y = if is_even_col {
                            vert * row as f64
                        } else {
                            vert * row as f64 + (vert / 2.0)
                        } + gutter_y;

                        cell = HexGridCell::with_flat_orientation()
                            .size(self.cell_size)
                            .center(Point::new(x, y));

                        callback_fn(sketch, &cell);
                    }
                    Orientation::Pointy => {
                        horiz = self.cell_size * (3.0 as f64).sqrt();
                        vert = 1.5 * self.cell_size;

                        x = if is_even_row {
                            horiz * column as f64
                        } else {
                            horiz * column as f64 + (horiz / 2.0)
                        } + gutter_x;
                        y = (vert * row as f64 + vert) + gutter_y;

                        cell = HexGridCell::with_pointy_orientation()
                            .size(self.cell_size)
                            .center(Point::new(x, y));

                        callback_fn(sketch, &cell);
                    }
                }
            }
        }
    }
}
