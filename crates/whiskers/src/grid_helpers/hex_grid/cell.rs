use vsvg::{IntoBezPathTolerance, Point};

pub enum Orientation {
    Flat,
    Pointy,
}

/// Stores basic hex grid's cell data, like size, orientation, or canvas position
///
/// Normally, you will not need to create it manually, as each cell in hexagonal
/// grid module is generated in the `build` method and passed to the callback function
/// as a reference.
///
/// But, you can, like this.
///
/// ```rust
/// let cell = HexGridCell::with_flat_orientation()
///     .center(Point::new(21.0, 37.0))
///     .size(129.0);
/// ```
pub struct HexGridCell {
    /// Center point of the grid cell
    pub center: Point,
    /// Size of the grid cell, meaning the distance from
    /// the cell's center point to each corner
    pub size: f64,
    /// Cell's column index in [doubled coordinates system](https://www.redblobgames.com/grids/hexagons/#coordinates-doubled)
    pub column: usize,
    /// Cell's row index in [doubled coordinates system](https://www.redblobgames.com/grids/hexagons/#coordinates-doubled)
    pub row: usize,
    orientation: Orientation,
}

impl HexGridCell {
    const DEFAULT_CENTER: [f64; 2] = [0.0, 0.0];
    const DEFAULT_SIZE: f64 = 10.0;
    const DEFAULT_POSITION: [usize; 2] = [0, 0];

    /// Creates cell with flat orientation and default center point and size
    #[must_use]
    pub fn with_flat_orientation() -> Self {
        Self {
            center: HexGridCell::DEFAULT_CENTER.into(),
            size: HexGridCell::DEFAULT_SIZE,
            orientation: Orientation::Flat,
            column: HexGridCell::DEFAULT_POSITION[0],
            row: HexGridCell::DEFAULT_POSITION[1],
        }
    }

    /// Creates cell with pointy orientation and default center point and size
    #[must_use]
    pub fn with_pointy_orientation() -> Self {
        Self {
            center: HexGridCell::DEFAULT_CENTER.into(),
            size: HexGridCell::DEFAULT_SIZE,
            orientation: Orientation::Pointy,
            column: HexGridCell::DEFAULT_POSITION[0],
            row: HexGridCell::DEFAULT_POSITION[1],
        }
    }

    /// Overrides cell's current size value
    #[must_use]
    pub fn size(mut self, value: f64) -> Self {
        self.size = value;
        self
    }

    /// Overrides cell's current position value
    #[must_use]
    pub fn center(mut self, value: Point) -> Self {
        self.center = value;
        self
    }

    /// Overrides cell's column index
    #[must_use]
    pub fn column(mut self, value: usize) -> Self {
        self.column = value;
        self
    }

    /// Overrides cell's row index
    #[must_use]
    pub fn row(mut self, value: usize) -> Self {
        self.row = value;
        self
    }

    /// Returns cell's width
    #[must_use]
    pub fn width(&self) -> f64 {
        match self.orientation {
            Orientation::Flat => self.size * 2.0,
            Orientation::Pointy => self.size * 3.0_f64.sqrt(),
        }
    }

    /// Returns cell's height
    #[must_use]
    pub fn height(&self) -> f64 {
        match self.orientation {
            Orientation::Flat => self.size * 3.0_f64.sqrt(),
            Orientation::Pointy => self.size * 2.0,
        }
    }

    /// Returns a list of points consisting a hexagon
    #[must_use]
    pub fn points(&self) -> Vec<Point> {
        (0..6).map(|index| self.corner(index)).collect()
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn corner(&self, index: usize) -> Point {
        match self.orientation {
            Orientation::Flat => self.corner_from_angle(60.0 * index as f64),
            Orientation::Pointy => self.corner_from_angle(60.0 * index as f64 - 30.0),
        }
    }

    fn corner_from_angle(&self, angle_deg: f64) -> Point {
        let angle_rad = angle_deg.to_radians();
        Point::new(
            self.center.x() + self.size * angle_rad.cos(),
            self.center.y() + self.size * angle_rad.sin(),
        )
    }
}

impl IntoBezPathTolerance for &HexGridCell {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> kurbo::BezPath {
        let mut bez_path = self
            .points()
            .iter()
            .enumerate()
            .map(|(index, point)| {
                if index == 0 {
                    kurbo::PathEl::MoveTo(point.into())
                } else {
                    kurbo::PathEl::LineTo(point.into())
                }
            })
            .fold(kurbo::BezPath::new(), |mut path, cmd| {
                path.push(cmd);
                path
            });
        bez_path.close_path();
        bez_path
    }
}
