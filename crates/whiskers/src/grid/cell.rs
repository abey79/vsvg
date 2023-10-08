//!
use vsvg::Point;

/// Grid cell data structure
///
/// It holds grid coordinate values, canvas position, size info
/// and optional data one could pass to it when using grid system
/// to implement one's own algorithm (like tiles for example).
pub struct GridCell<T> {
    /// Column index
    pub column: usize,
    /// Row index
    pub row: usize,
    /// Canvas position
    pub canvas_position: Point,
    /// User-defined data
    pub data: Option<T>,
    /// Size in pixel units
    pub size: [f64; 2],
}

impl<T> GridCell<T> {
    /// Reset data in the cell
    pub fn reset_data(&mut self) {
        self.data = None;
    }
}
