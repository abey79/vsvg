//!
use vsvg::Point;

use self::cell::GridCell;

pub mod cell;

/// Grid's size can be set either by passing
/// the cell's or grid's dimensions. Pass one of the enum members
/// to choose
pub enum GridSize {
    /// Set cell size, grid's size will be computed
    CellBased([f64; 2]),
    /// Set fixed grid size
    GridBased([f64; 2]),
}

/// 2-dimensional square grid module
///
/// Borrowed from [Programing Design Systems book by Rune Madsen](https://www.programmingdesignsystems.com/), this module
/// helps to work with 2-dimensional grids more efficiently.
///
/// ```rust
/// use whiskers::grid::{Grid, GridSize};
///
/// let mut grid = Grid::<MyType>::new(
///     10,
///     10,
///     GridSize::GridBased([sketch.width(), sketch.height()]),
///     [10.0, 10.0],
///     Point::new(0.0, 0.0),
/// );
/// grid.init(None);
/// ```
pub struct Grid<T> {
    columns: usize,
    rows: usize,
    size: GridSize,
    gutter: [f64; 2],
    position: Point,
    /// List of grid cells filled with data of given type
    pub data: Vec<GridCell<T>>,
}

/// Function to populate the grid with data of given type
pub type GridInitFn<T> = fn(column: usize, row: usize, data: &Vec<GridCell<T>>) -> Option<T>;

impl<T> Grid<T> {
    /// Creates new instance of the grid with empty data vector
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        columns: usize,
        rows: usize,
        size: GridSize,
        gutter: [f64; 2],
        position: Point,
    ) -> Self {
        Self {
            columns,
            rows,
            size,
            gutter,
            position,
            data: vec![],
        }
    }

    /// Fills data vector with grid cells. You can pass option with grid cell
    /// filling function to pass data of your type to every grid cell
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn init(&mut self, init_fn: Option<GridInitFn<T>>) -> &mut Self {
        let [module_width, module_height] = self.module_size();
        let [gutter_width, gutter_height] = self.gutter;
        let mut data: Vec<GridCell<T>> = vec![];

        for row in 0..self.rows {
            for column in 0..self.columns {
                let pos_x = self.position.x()
                    + (column as f64 * module_width + column as f64 * gutter_width);
                let pos_y =
                    self.position.y() + (row as f64 * module_height + row as f64 * gutter_height);

                data.push(GridCell {
                    column,
                    row,
                    canvas_position: Point::new(pos_x, pos_y),
                    data: match init_fn {
                        Some(f) => f(column, row, &data),
                        None => None,
                    },
                    size: self.module_size(),
                });
            }
        }

        self.data = data;
        self
    }

    /// Returns optional reference to a grid cell at specific column and row index
    pub fn at(&mut self, column: usize, row: usize) -> Option<&mut GridCell<T>> {
        self.data
            .iter_mut()
            .find(|cell| cell.column == column && cell.row == row)
    }

    /// Resets grid cell's data at specific column and row if applicable
    pub fn reset_at(&mut self, column: usize, row: usize) {
        let cell = self.at(column, row);

        if let Some(cell) = cell {
            cell.reset_data();
        }
    }

    /// Resets all grid cell's data
    pub fn reset(&mut self) {
        self.data.iter_mut().for_each(GridCell::reset_data);
    }

    /// Aligns grid cells to specific dimensions
    pub fn align_to(&mut self, rect_dimensions: [f64; 2]) {
        let [width, height] = self.module_size();
        let diff_x = rect_dimensions[0] / width;
        let diff_y = rect_dimensions[1] / height;

        self.data.iter_mut().for_each(|cell| {
            cell.canvas_position = Point::new(
                cell.canvas_position.x() + diff_x,
                cell.canvas_position.y() + diff_y,
            );
        });
    }

    /// Returns width of the grid
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn width(&self) -> f64 {
        self.columns as f64 * self.module_size()[0] + self.columns as f64 * self.gutter[0]
    }

    /// Returns height of the grid
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn height(&self) -> f64 {
        self.rows as f64 * self.module_size()[1] + self.rows as f64 * self.gutter[1]
    }

    /// Returns grid module's size
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn module_size(&self) -> [f64; 2] {
        match self.size {
            GridSize::GridBased([width, height]) => {
                let cols = self.columns as f64;
                let rows = self.rows as f64;
                let [gutter_width, gutter_height] = self.gutter;

                [
                    (width - (cols - 1.0) * gutter_width) / cols,
                    (height - (rows - 1.0) * gutter_height) / rows,
                ]
            }
            GridSize::CellBased(s) => s,
        }
    }
}
