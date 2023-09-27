use crate::path::into_bezpath::IntoBezPathTolerance;
use crate::{Point, Polyline};
use kurbo::Vec2;

pub trait Draw {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self;

    /// Draw a cubic Bézier curve from (`x1`, `y1`) to (`x4`, `y4`) with control points at (`x2`,
    /// `y2`) and (`x3`, `y3`).
    #[allow(clippy::too_many_arguments)]
    fn cubic_bezier(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        x4: f64,
        y4: f64,
    ) -> &mut Self {
        self.add_path(kurbo::CubicBez::new((x1, y1), (x2, y2), (x3, y3), (x4, y4)))
    }

    /// Draw a quadratic Bézier curve from (`x1`, `y1`) to (`x3`, `y3`) with control point at (`x2`,
    /// `y2`).
    fn quadratic_bezier(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    ) -> &mut Self {
        self.add_path(kurbo::QuadBez::new((x1, y1), (x2, y2), (x3, y3)))
    }

    /// Draw an elliptical arc centered on (`x`, `y`) with radii `rx` and `ry`. The arc starts at
    /// `start` and sweeps `sweep` radians. `x_rot` is the rotation of the ellipse in radians.
    #[allow(clippy::too_many_arguments)]
    fn arc(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        start: f64,
        sweep: f64,
        x_rot: f64,
    ) -> &mut Self {
        self.add_path(kurbo::Arc {
            center: kurbo::Point { x, y },
            radii: Vec2 { x: rx, y: ry },
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: x_rot,
        })
    }

    /// Draw a circle centered on (`x`, `y`) with radius `r`.
    fn circle(&mut self, x: f64, y: f64, r: f64) -> &mut Self {
        self.add_path(kurbo::Circle::new(kurbo::Point { x, y }, r))
    }

    /// Draw an ellipse centered on (`x`, `y`) with radii `rx` and `ry`. `x_rot` is the rotation of
    /// the ellipse in radians.
    fn ellipse(&mut self, x: f64, y: f64, rx: f64, ry: f64, x_rot: f64) -> &mut Self {
        self.add_path(kurbo::Ellipse::new((x, y), (rx, ry), x_rot))
    }

    /// Draw a line from (`x1`, `y1`) to (`x2`, `y2`).
    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> &mut Self {
        self.add_path(kurbo::Line::new((x1, y1), (x2, y2)))
    }

    /// Draw a polyline from a sequence of points, optionally closing it.
    fn polyline(
        &mut self,
        points: impl IntoIterator<Item = impl Into<Point>>,
        close: bool,
    ) -> &mut Self {
        let mut polyline = Polyline::from_iter(points);
        if close {
            polyline.close();
        }

        self.add_path(polyline)
    }

    /// Draw a rectangle centered on (`x`, `y`) with width `w` and height `h`.
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> &mut Self {
        self.add_path(kurbo::Rect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
        ))
    }

    /// Draw a rounded rectangle centered on (`x`, `y`) with width `w` and height `h`. The corners
    /// are rounded with radii `tl`, `tr`, `br`, `bl`.
    #[allow(clippy::too_many_arguments)]
    fn rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        tl: f64,
        tr: f64,
        br: f64,
        bl: f64,
    ) -> &mut Self {
        self.add_path(kurbo::RoundedRect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
            (tl, tr, br, bl),
        ))
    }

    /// Draw from an SVG path representation.
    fn svg_path(&mut self, path: &str) -> Result<&mut Self, kurbo::SvgParseError> {
        self.add_path(kurbo::BezPath::from_svg(path)?);
        Ok(self)
    }
}
