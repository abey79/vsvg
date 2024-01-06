use crate::path::into_bezpath::IntoBezPathTolerance;
use crate::{Point, Polyline};
use kurbo::Vec2;

pub trait Draw {
    /// Draw a single path.
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self;

    /// Draw a sequence of paths.
    #[inline]
    fn add_paths(
        &mut self,
        paths: impl IntoIterator<Item = impl IntoBezPathTolerance>,
    ) -> &mut Self {
        paths.into_iter().for_each(|path| {
            self.add_path(path);
        });
        self
    }

    /// Draw a cubic Bézier curve from (`x1`, `y1`) to (`x4`, `y4`) with control points at (`x2`,
    /// `y2`) and (`x3`, `y3`).
    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn cubic_bezier(
        &mut self,
        x1: impl Into<f64>,
        y1: impl Into<f64>,
        x2: impl Into<f64>,
        y2: impl Into<f64>,
        x3: impl Into<f64>,
        y3: impl Into<f64>,
        x4: impl Into<f64>,
        y4: impl Into<f64>,
    ) -> &mut Self {
        self.add_path(kurbo::CubicBez::new(
            (x1.into(), y1.into()),
            (x2.into(), y2.into()),
            (x3.into(), y3.into()),
            (x4.into(), y4.into()),
        ))
    }

    /// Draw a quadratic Bézier curve from (`x1`, `y1`) to (`x3`, `y3`) with control point at (`x2`,
    /// `y2`).
    #[inline]
    fn quadratic_bezier(
        &mut self,
        x1: impl Into<f64>,
        y1: impl Into<f64>,
        x2: impl Into<f64>,
        y2: impl Into<f64>,
        x3: impl Into<f64>,
        y3: impl Into<f64>,
    ) -> &mut Self {
        self.add_path(kurbo::QuadBez::new(
            (x1.into(), y1.into()),
            (x2.into(), y2.into()),
            (x3.into(), y3.into()),
        ))
    }

    /// Draw an elliptical arc centered on (`x`, `y`) with radii `rx` and `ry`. The arc starts at
    /// `start` and sweeps `sweep` radians. `x_rot` is the rotation of the ellipse in radians.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn arc(
        &mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        rx: impl Into<f64>,
        ry: impl Into<f64>,
        start: f64,
        sweep: f64,
        x_rot: f64,
    ) -> &mut Self {
        self.add_path(kurbo::Arc {
            center: kurbo::Point {
                x: x.into(),
                y: y.into(),
            },
            radii: Vec2 {
                x: rx.into(),
                y: ry.into(),
            },
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: x_rot,
        })
    }

    /// Draw a circle centered on (`x`, `y`) with radius `r`.
    #[inline]
    fn circle(&mut self, x: impl Into<f64>, y: impl Into<f64>, r: impl Into<f64>) -> &mut Self {
        self.add_path(kurbo::Circle::new(
            kurbo::Point {
                x: x.into(),
                y: y.into(),
            },
            r.into(),
        ))
    }

    /// Draw an ellipse centered on (`x`, `y`) with radii `rx` and `ry`. `x_rot` is the rotation of
    /// the ellipse in radians.
    #[inline]
    fn ellipse(
        &mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        rx: impl Into<f64>,
        ry: impl Into<f64>,
        x_rot: f64,
    ) -> &mut Self {
        self.add_path(kurbo::Ellipse::new(
            (x.into(), y.into()),
            (rx.into(), ry.into()),
            x_rot,
        ))
    }

    /// Draw a line from (`x1`, `y1`) to (`x2`, `y2`).
    #[inline]
    fn line(
        &mut self,
        x1: impl Into<f64>,
        y1: impl Into<f64>,
        x2: impl Into<f64>,
        y2: impl Into<f64>,
    ) -> &mut Self {
        self.add_path(kurbo::Line::new(
            (x1.into(), y1.into()),
            (x2.into(), y2.into()),
        ))
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
    #[inline]
    fn rect(
        &mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        w: impl Into<f64>,
        h: impl Into<f64>,
    ) -> &mut Self {
        let x = x.into();
        let y = y.into();
        let w = w.into();
        let h = h.into();
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
    #[inline]
    fn rounded_rect(
        &mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        w: impl Into<f64>,
        h: impl Into<f64>,
        tl: impl Into<f64>,
        tr: impl Into<f64>,
        br: impl Into<f64>,
        bl: impl Into<f64>,
    ) -> &mut Self {
        let x = x.into();
        let y = y.into();
        let w = w.into();
        let h = h.into();
        self.add_path(kurbo::RoundedRect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
            (tl.into(), tr.into(), br.into(), bl.into()),
        ))
    }

    /// Draw a Catmull-Rom spline from a sequence of points and a tension parameter.
    ///
    /// See [`crate::CatmullRom`] for more information.
    fn catmull_rom(
        &mut self,
        points: impl IntoIterator<Item = impl Into<Point>>,
        tension: f64,
    ) -> &mut Self {
        self.add_path(crate::CatmullRom::from_points(points).tension(tension));
        self
    }

    /// Draw from an SVG path representation.
    fn svg_path(&mut self, path: &str) -> Result<&mut Self, kurbo::SvgParseError> {
        self.add_path(kurbo::BezPath::from_svg(path)?);
        Ok(self)
    }
}
