use vsvg::{IntoBezPathTolerance, Point};

use crate::Sketch;

enum GridSize {
    CellBased([f64; 2]),
    GridBased([f64; 2]),
}

/// Stores basic grid's cell data, like column, row and canvas position
#[derive(Clone, Debug, PartialEq)]
pub struct GridCell {
    /// Cell's grid column index
    pub column: usize,
    /// Cell's grid row index
    pub row: usize,
    /// Cell's position within the grid coordinate system
    pub position: Point,
    /// Cell's width and height
    pub size: [f64; 2],
    /// Grid's width and height
    pub grid_size: [f64; 2],
}

impl IntoBezPathTolerance for &GridCell {
    fn into_bezpath_with_tolerance(self, tolerance: f64) -> kurbo::BezPath {
        let [width, height] = self.size;
        let position: kurbo::Point = self.position.into();

        kurbo::Rect::from_origin_size(position, kurbo::Size { width, height })
            .into_bezpath_with_tolerance(tolerance)
    }
}

/// 2-dimensional square grid module
///
/// Borrowed from [Programing Design Systems book by Rune Madsen](https://www.programmingdesignsystems.com/), this module
/// helps to work with 2-dimensional grids more efficiently.
///
/// ```rust
/// use whiskers::prelude::*;
///
/// let mut sketch = Sketch::new();
/// Grid::from_total_size([600.0, 800.0])
///     .columns(5)
///     .rows(10)
///     .position(Point::new(20.0, 100.0))
///     .spacing([10.0, 10.0])
///     .build(&mut sketch, |sketch, cell| {
///         sketch.add_path(cell);
///     });
/// ```
pub struct Grid {
    dimensions: [usize; 2],
    size: GridSize,
    gutter: [f64; 2],
    top_left_corner: Point,
}

impl Grid {
    const DEFAULT_DIMENSIONS: [usize; 2] = [4, 4];
    const DEFAULT_GUTTER: [f64; 2] = [0.0, 0.0];
    const DEFAULT_POSITION: [f64; 2] = [0.0, 0.0];

    /// Creates grid instance based on the total dimensions
    /// given by the user.
    pub fn from_total_size(size: [f64; 2]) -> Self {
        Self {
            dimensions: Grid::DEFAULT_DIMENSIONS,
            size: GridSize::GridBased(size),
            gutter: Grid::DEFAULT_GUTTER,
            top_left_corner: Point::from(Grid::DEFAULT_POSITION),
        }
    }

    /// Creates grid instance based on the cell dimensions
    /// given by the user.
    pub fn from_cell_size(size: [f64; 2]) -> Self {
        Self {
            dimensions: Grid::DEFAULT_DIMENSIONS,
            size: GridSize::CellBased(size),
            gutter: Grid::DEFAULT_GUTTER,
            top_left_corner: Point::from(Grid::DEFAULT_POSITION),
        }
    }

    /// Overrides grid's current number of rows.
    /// By default, grid instance will have 4 rows.
    #[must_use]
    pub fn rows(mut self, value: usize) -> Self {
        self.dimensions[1] = value;
        self
    }

    /// Overrides grid's current number of columns.
    /// By default, grid instance will have 4 columns.
    #[must_use]
    pub fn columns(mut self, value: usize) -> Self {
        self.dimensions[0] = value;
        self
    }

    /// Overrides grid's current horizontal and vertical spacing values.
    /// By default, grid instance will have zero spacing on both axes.
    #[must_use]
    pub fn spacing(mut self, value: [f64; 2]) -> Self {
        self.gutter = value;
        self
    }

    /// Overrides grid's current horizontal spacing value.
    #[must_use]
    pub fn horizontal_spacing(mut self, value: f64) -> Self {
        self.gutter[0] = value;
        self
    }

    /// Overrides grid's current vertical spacing value.
    #[must_use]
    pub fn vertical_spacing(mut self, value: f64) -> Self {
        self.gutter[1] = value;
        self
    }

    /// Overrides grid's current position. Default value is
    /// a Point instance with 0.0 value for both axes.
    #[must_use]
    pub fn position(mut self, value: Point) -> Self {
        self.top_left_corner = value;
        self
    }

    /// Computes grid's cell data such as coordinates (column and row),
    /// size and canvas position.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn build(self, sketch: &mut Sketch, mut callback_fn: impl FnMut(&mut Sketch, &GridCell)) {
        let [module_width, module_height] = self.module_size();
        let [gutter_width, gutter_height] = self.gutter;
        let [columns, rows] = self.dimensions;
        let grid_size = self.size();

        for row in 0..rows {
            for column in 0..columns {
                let pos_x = self.top_left_corner.x()
                    + (column as f64 * module_width + column as f64 * gutter_width);
                let pos_y = self.top_left_corner.y()
                    + (row as f64 * module_height + row as f64 * gutter_height);

                let cell = GridCell {
                    column,
                    row,
                    position: Point::new(pos_x, pos_y),
                    size: [module_width, module_height],
                    grid_size,
                };
                callback_fn(sketch, &cell);
            }
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn module_size(&self) -> [f64; 2] {
        match self.size {
            GridSize::GridBased([width, height]) => {
                let cols = self.dimensions[0] as f64;
                let rows = self.dimensions[1] as f64;
                let [gutter_width, gutter_height] = self.gutter;

                [
                    (width - (cols - 1.0) * gutter_width) / cols,
                    (height - (rows - 1.0) * gutter_height) / rows,
                ]
            }
            GridSize::CellBased(s) => s,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn size(&self) -> [f64; 2] {
        let columns = self.dimensions[0] as f64;
        let rows = self.dimensions[1] as f64;
        let [gutter_x, gutter_y] = self.gutter;
        let [module_width, module_height] = self.module_size();

        [
            columns * module_width + columns * gutter_x,
            rows * module_height + rows * gutter_y,
        ]
    }
}
