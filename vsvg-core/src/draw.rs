use crate::{Color, Layer, Path, PathTrait, Transforms};
use kurbo::{Affine, Shape, Vec2};

#[derive(Debug)]
pub struct DrawState {
    transform: Affine, //TODO: make it a stack?
    color: Color,
    stroke_width: f64,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            transform: Affine::default(),
            color: Color::default(),
            stroke_width: 1.0,
        }
    }
}

impl DrawState {
    #[must_use]
    pub fn new() -> Self {
        DrawState::default()
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn stroke_width(&mut self, stroke_width: f64) {
        self.stroke_width = stroke_width;
    }
}

impl Transforms for DrawState {
    fn transform(&mut self, affine: &Affine) {
        self.transform = *affine * self.transform;
    }
}

pub struct Draw<'layer, 'state> {
    state: &'state DrawState,
    layer: &'layer mut Layer,
}

impl<'layer, 'state> Draw<'layer, 'state> {
    /// Draw a cubic Bézier curve from (`x1`, `y1`) to (`x4`, `y4`) with control points at (`x2`,
    /// `y2`) and (`x3`, `y3`).
    #[allow(clippy::too_many_arguments)]
    pub fn cubic_bezier(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        x4: f64,
        y4: f64,
    ) -> &Self {
        self.add_shape(kurbo::CubicBez::new((x1, y1), (x2, y2), (x3, y3), (x4, y4)))
    }

    /// Draw a quadratic Bézier curve from (`x1`, `y1`) to (`x3`, `y3`) with control point at (`x2`,
    /// `y2`).
    pub fn quadratic_bezier(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    ) -> &Self {
        self.add_shape(kurbo::QuadBez::new((x1, y1), (x2, y2), (x3, y3)))
    }

    /// Draw an elliptical arc centered on (`x`, `y`) with radii `rx` and `ry`. The arc starts at
    /// `start` and sweeps `sweep` radians. `x_rot` is the rotation of the ellipse in radians.
    #[allow(clippy::too_many_arguments)]
    pub fn arc(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        start: f64,
        sweep: f64,
        x_rot: f64,
    ) -> &Self {
        self.add_shape(kurbo::Arc {
            center: kurbo::Point { x, y },
            radii: Vec2 { x: rx, y: ry },
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: x_rot,
        })
    }

    /// Draw a circle centered on (`x`, `y`) with radius `r`.
    pub fn circle(&mut self, x: f64, y: f64, r: f64) -> &Self {
        self.add_shape(kurbo::Circle::new(kurbo::Point { x, y }, r))
    }

    /// Draw an ellipse centered on (`x`, `y`) with radii `rx` and `ry`. `x_rot` is the rotation of
    /// the ellipse in radians.
    pub fn ellipse(&mut self, x: f64, y: f64, rx: f64, ry: f64, x_rot: f64) -> &Self {
        self.add_shape(kurbo::Ellipse::new((x, y), (rx, ry), x_rot))
    }

    /// Draw a line from (`x1`, `y1`) to (`x2`, `y2`).
    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> &Self {
        self.add_shape(kurbo::Line::new((x1, y1), (x2, y2)))
    }

    /// Draw a rectangle centered on (`x`, `y`) with width `w` and height `h`.
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> &Self {
        self.add_shape(kurbo::Rect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
        ))
    }

    /// Draw a rounded rectangle centered on (`x`, `y`) with width `w` and height `h`. The corners
    /// are rounded with radii `tl`, `tr`, `br`, `bl`.
    #[allow(clippy::too_many_arguments)]
    pub fn rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        tl: f64,
        tr: f64,
        br: f64,
        bl: f64,
    ) -> &Self {
        self.add_shape(kurbo::RoundedRect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
            (tl, tr, br, bl),
        ))
    }

    /// Draw from an SVG path representation.
    pub fn svg_path(&mut self, path: &str) -> Result<(), kurbo::SvgParseError> {
        self.add_shape(kurbo::BezPath::from_svg(path)?);
        Ok(())
    }

    fn add_shape<T: Shape>(&mut self, shape: T) -> &Self {
        let mut path: Path = shape.into();
        path.metadata_mut().color = self.state.color;
        path.metadata_mut().stroke_width = self.state.stroke_width;
        path.apply_transform(self.state.transform);
        self.layer.paths.push(path);
        self
    }
}

impl Layer {
    pub fn draw<'layer, 'state>(
        &'layer mut self,
        state: &'state DrawState,
    ) -> Draw<'layer, 'state> {
        Draw { state, layer: self }
    }
}
