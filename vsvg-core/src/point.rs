use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    data: [f64; 2],
}

impl Point {
    pub const ZERO: Point = Point { data: [0.0, 0.0] };

    #[must_use]
    #[inline]
    pub fn new<T, U>(x: T, y: U) -> Self
    where
        T: Into<f64>,
        U: Into<f64>,
    {
        Self {
            data: [x.into(), y.into()],
        }
    }

    #[must_use]
    #[inline]
    pub fn x(&self) -> f64 {
        self.data[0]
    }

    #[must_use]
    #[inline]
    pub fn y(&self) -> f64 {
        self.data[1]
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for (f64, f64) {
    fn from(p: Point) -> Self {
        (p.x(), p.y())
    }
}

impl From<[f64; 2]> for Point {
    fn from([x, y]: [f64; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for [f64; 2] {
    fn from(p: Point) -> Self {
        p.data
    }
}

impl AsRef<[f64]> for Point {
    fn as_ref(&self) -> &[f64] {
        &self.data
    }
}

impl AsRef<[f64; 2]> for Point {
    fn as_ref(&self) -> &[f64; 2] {
        &self.data
    }
}

impl From<kurbo::Point> for Point {
    fn from(p: kurbo::Point) -> Self {
        Self::new(p.x, p.y)
    }
}

impl From<Point> for kurbo::Point {
    fn from(p: Point) -> Self {
        Self { x: p.x(), y: p.y() }
    }
}

impl From<&Point> for [f64; 2] {
    fn from(p: &Point) -> Self {
        p.data
    }
}

impl From<&Point> for kurbo::Point {
    fn from(p: &Point) -> Self {
        Self { x: p.x(), y: p.y() }
    }
}

impl From<&kurbo::Point> for Point {
    fn from(p: &kurbo::Point) -> Self {
        Self::new(p.x, p.y)
    }
}

impl Mul<Point> for kurbo::Affine {
    type Output = Point;

    #[inline]
    fn mul(self, other: Point) -> Point {
        let coeffs = self.as_coeffs();
        Point::new(
            coeffs[0] * other.x() + coeffs[2] * other.y() + coeffs[4],
            coeffs[1] * other.x() + coeffs[3] * other.y() + coeffs[5],
        )
    }
}
