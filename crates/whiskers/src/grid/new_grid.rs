use egui::emath::Numeric;
use vsvg::Point;

/// Grid's size can be set either by passing
/// the cell's or grid's dimensions. Pass one of the enum members
/// to choose
enum GridSize {
    /// Set cell size, grid's size will be computed
    CellBased([f64; 2]),
    /// Set fixed grid size
    GridBased([f64; 2]),
}

pub struct GridCell {
    column: usize,
    row: usize,
    position: Point,
    size: [f64; 2],
}

/// 2-dimensional square grid module
///
/// Borrowed from [Programing Design Systems book by Rune Madsen](https://www.programmingdesignsystems.com/), this module
/// helps to work with 2-dimensional grids more efficiently.
///
/// ```rust
/// use whiskers::grid::{Grid};
///
/// Grid::from_total_size([600.0, 800.0])
///     .columns(5)
///     .rows(10)
///     .translate(Point::new(20.0, 100.0))
///     .spacing([10.0, 10.0])
///     .build();
/// ```
pub struct Grid {
    dimensions: [usize; 2],
    size: GridSize,
    gutter: [f64; 2],
    position: Point,
    data: Vec<GridCell>,
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
            position: Point::from(Grid::DEFAULT_POSITION),
            data: vec![],
        }
    }

    /// Creates grid instance based on the cell dimensions
    /// given by the user.
    pub fn from_cell_size(size: [f64; 2]) -> Self {
        Self {
            dimensions: Grid::DEFAULT_DIMENSIONS,
            size: GridSize::CellBased(size),
            gutter: Grid::DEFAULT_GUTTER,
            position: Point::from(Grid::DEFAULT_POSITION),
            data: vec![],
        }
    }

    /// Overrides grid's current number of rows.
    /// By default, grid instance will have 4 rows.
    pub fn rows(&mut self, value: usize) -> &mut Self {
        self.dimensions[1] = value;
        self
    }

    /// Overrides grid's current number of columns.
    /// By default, grid instance will have 4 columns.
    pub fn columns(&mut self, value: usize) -> &mut Self {
        self.dimensions[0] = value;
        self
    }

    /// Overrides grid's current horizontal and vertical spacing values.
    /// By default, grid instance will have zero spacing on both axes.
    pub fn spacing(&mut self, value: [f64; 2]) -> &mut Self {
        self.gutter = value;
        self
    }

    /// Overrides grid's current horizontal spacing value.
    pub fn horizontal_spacing(&mut self, value: f64) -> &mut Self {
        self.gutter[0] = value;
        self
    }

    /// Overrides grid's current vertical spacing value.
    pub fn vertical_spacing(&mut self, value: f64) -> &mut Self {
        self.gutter[1] = value;
        self
    }

    /// Overrides grid's current position. Default value is
    /// a Point instance with 0.0 value for both axes.
    pub fn translate(&mut self, value: Point) -> &mut Self {
        self.position = value;
        self
    }

    /// Computes grid's cell data such as coordinates (column and row),
    /// size and canvas position.
    pub fn build(&mut self) {
        let [module_width, module_height] = self.module_size();
        let [gutter_width, gutter_height] = self.gutter;
        let [columns, rows] = self.dimensions;
        let mut cells: Vec<GridCell> = vec![];

        for row in 0..rows {
            for column in 0..columns {
                let pos_x = self.position.x()
                    + (column as f64 * module_width + column as f64 * gutter_width);
                let pos_y =
                    self.position.y() + (row as f64 * module_height + row as f64 * gutter_height);

                cells.push(GridCell {
                    column,
                    row,
                    position: Point::new(pos_x, pos_y),
                    size: self.module_size(),
                });
            }
        }
        self.data = cells;
    }

    /// Returns grid module's size
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn module_size(&self) -> [f64; 2] {
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

    /// Returns width of the grid
    pub fn width(&self) -> f64 {
        let columns = self.dimensions[0].to_f64();
        columns * self.module_size()[0] + columns * self.gutter[0]
    }

    /// Returns height of the grid
    pub fn height(&self) -> f64 {
        let rows = self.dimensions[0].to_f64();
        rows * self.module_size()[1] + rows * self.gutter[1]
    }
}
