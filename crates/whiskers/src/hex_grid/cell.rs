use vsvg::{IntoBezPathTolerance, Point};

pub enum Orientation {
    Flat,
    Pointy,
}

pub struct HexGridCell {
    pub center: Point,
    pub size: f64,
    orientation: Orientation,
}

impl HexGridCell {
    pub const DEFAULT_CENTER: [f64; 2] = [0.0, 0.0];
    pub const DEFAULT_SIZE: f64 = 10.0;

    pub fn with_flat_orientation() -> Self {
        Self {
            center: HexGridCell::DEFAULT_CENTER.into(),
            size: HexGridCell::DEFAULT_SIZE,
            orientation: Orientation::Flat,
        }
    }

    pub fn with_pointy_orientation() -> Self {
        Self {
            center: HexGridCell::DEFAULT_CENTER.into(),
            size: HexGridCell::DEFAULT_SIZE,
            orientation: Orientation::Pointy,
        }
    }

    pub fn size(mut self, value: f64) -> Self {
        self.size = value;
        self
    }

    pub fn center(mut self, value: Point) -> Self {
        self.center = value;
        self
    }

    fn corner(&self, index: usize) -> Point {
        match self.orientation {
            Orientation::Flat => self.get_corner_point(60.0 * index as f64),
            Orientation::Pointy => self.get_corner_point(60.0 * index as f64 - 30.0),
        }
    }

    fn get_corner_point(&self, angle_deg: f64) -> Point {
        let angle_rad = angle_deg.to_radians();
        Point::new(
            self.center.x() + self.size * angle_rad.cos(),
            self.center.y() + self.size * angle_rad.sin(),
        )
    }
}

impl IntoBezPathTolerance for &HexGridCell {
    fn into_bezpath_with_tolerance(self, _tolerance: f64) -> kurbo::BezPath {
        let mut bez_path = (0..6)
            .map(|index| self.corner(index))
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
        bez_path.push(kurbo::PathEl::ClosePath);
        bez_path
    }
}
