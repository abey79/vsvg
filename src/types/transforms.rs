use crate::types::{DocumentImpl, LayerImpl, PathImpl, Polyline};
use kurbo::{Affine, BezPath};

#[allow(dead_code)]
fn skew_affine(kx: f64, ky: f64) -> Affine {
    // this is missing from kurbo, so we implement it here
    // | a c e |
    // | b d f |
    // | 0 0 1 |

    Affine::new([1.0, ky.tan(), kx.tan(), 1.0, 0.0, 0.0])
}

pub trait Transforms: Sized {
    /// Apply a 2D affine transform
    #[must_use]
    fn apply_affine(self, affine: Affine) -> Self;

    /// Translate the geometry by `dx` and `dy`.
    #[must_use]
    fn translate(self, dx: f64, dy: f64) -> Self {
        self.apply_affine(Affine::translate((dx, dy)))
    }

    /// Scale the geometry by `s` around the origin.
    #[must_use]
    fn scale(self, s: f64) -> Self {
        self.apply_affine(Affine::scale(s))
    }

    /// Scale the geometry by `sx` and `sy` around the origin.
    #[must_use]
    fn scale_non_uniform(self, sx: f64, sy: f64) -> Self {
        self.apply_affine(Affine::scale_non_uniform(sx, sy))
    }

    /// Scale the geometry by `sx` and `sy` around the point `(cx, cy)`.
    #[must_use]
    fn scale_around(self, sx: f64, sy: f64, cx: f64, cy: f64) -> Self {
        let transform = Affine::translate((cx, cy))
            * Affine::scale_non_uniform(sx, sy)
            * Affine::translate((-cx, -cy));
        self.apply_affine(transform)
    }

    /// Rotate the geometry by `theta` radians around the origin.
    #[must_use]
    fn rotate(self, theta: f64) -> Self {
        self.apply_affine(Affine::rotate(theta))
    }

    /// Rotate the geometry by `theta` radians around the point `(cx, cy)`.
    #[must_use]
    fn rotate_around(self, theta: f64, cx: f64, cy: f64) -> Self {
        let transform =
            Affine::translate((cx, cy)) * Affine::rotate(theta) * Affine::translate((-cx, -cy));
        self.apply_affine(transform)
    }

    /// Skew the geometry by `kx` and `ky` radians around the origin.
    #[must_use]
    fn skew(self, kx: f64, ky: f64) -> Self {
        self.apply_affine(skew_affine(kx, ky))
    }

    /// Skew the geometry by `kx` and `ky` radians around the point `(cx, cy)`.
    #[must_use]
    fn skew_around(self, kx: f64, ky: f64, cx: f64, cy: f64) -> Self {
        let transform =
            Affine::translate((cx, cy)) * skew_affine(kx, ky) * Affine::translate((-cx, -cy));
        self.apply_affine(transform)
    }
}

impl Transforms for BezPath {
    // `BezPath` has a built-in `apply_affine` method, but it mutates the path. We reimplement it
    // here to return a new path instead.
    fn apply_affine(self, affine: Affine) -> Self {
        self.into_iter().map(|el| affine * el).collect()
    }
}

impl Transforms for Polyline {
    fn apply_affine(self, affine: Affine) -> Self {
        self.into_iter()
            .map(|point| {
                let new_pt = affine * kurbo::Point::new(point[0], point[1]);
                [new_pt.x, new_pt.y]
            })
            .collect()
    }
}

impl<T: Transforms + Default> Transforms for PathImpl<T> {
    fn apply_affine(self, affine: Affine) -> Self {
        Self {
            data: self.data.apply_affine(affine),
            ..self
        }
    }
}

impl<T: Transforms + Default> Transforms for LayerImpl<T> {
    fn apply_affine(self, affine: Affine) -> Self {
        self.map_paths(|path| path.apply_affine(affine))
    }
}

impl<T: Transforms + Default> Transforms for DocumentImpl<T> {
    fn apply_affine(self, affine: Affine) -> Self {
        self.map_layers(|layer| layer.apply_affine(affine))
    }
}

// TODO: tests
