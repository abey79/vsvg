use crate::{Color, Layer, Path, Transforms};
use kurbo::{Affine, Shape};

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
    fn apply_affine(&mut self, affine: &Affine) {
        self.transform = *affine * self.transform;
    }
}

pub struct Draw<'layer, 'state> {
    state: &'state DrawState,
    layer: &'layer mut Layer,
}

impl<'layer, 'state> Draw<'layer, 'state> {
    /// Draw a rectangle centered on (`x`, `y`) with width `w` and height `h`.
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> &Self {
        self.add_shape(kurbo::Rect::new(
            x - w * 0.5,
            y - h * 0.5,
            x + w * 0.5,
            y + h * 0.5,
        ))
    }

    fn add_shape<T: Shape>(&mut self, shape: T) -> &Self {
        let mut path = Path::from_shape(shape);
        path.color = self.state.color;
        path.stroke_width = self.state.stroke_width;
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
