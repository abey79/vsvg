use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for (f64, f64) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

impl From<[f64; 2]> for Point {
    fn from([x, y]: [f64; 2]) -> Self {
        Self { x, y }
    }
}

impl From<Point> for [f64; 2] {
    fn from(p: Point) -> Self {
        [p.x, p.y]
    }
}

impl From<kurbo::Point> for Point {
    fn from(p: kurbo::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<Point> for kurbo::Point {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<&Point> for [f64; 2] {
    fn from(p: &Point) -> Self {
        [p.x, p.y]
    }
}

impl From<&Point> for kurbo::Point {
    fn from(p: &Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<&kurbo::Point> for Point {
    fn from(p: &kurbo::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Mul<Point> for kurbo::Affine {
    type Output = Point;

    #[inline]
    fn mul(self, other: Point) -> Point {
        let coeffs = self.as_coeffs();
        Point::new(
            coeffs[0] * other.x + coeffs[2] * other.y + coeffs[4],
            coeffs[1] * other.x + coeffs[3] * other.y + coeffs[5],
        )
    }
}
