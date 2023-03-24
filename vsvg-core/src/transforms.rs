use crate::{DocumentImpl, LayerImpl, PathImpl, PathType, Polyline};
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
    fn apply_affine(&mut self, affine: &Affine);

    /// Translate the geometry by `dx` and `dy`.
    fn translate(&mut self, dx: f64, dy: f64) {
        self.apply_affine(&Affine::translate((dx, dy)));
    }

    /// Scale the geometry by `s` around the origin.
    fn scale(&mut self, s: f64) {
        self.apply_affine(&Affine::scale(s));
    }

    /// Scale the geometry by `sx` and `sy` around the origin.
    fn scale_non_uniform(&mut self, sx: f64, sy: f64) {
        self.apply_affine(&Affine::scale_non_uniform(sx, sy));
    }

    /// Scale the geometry by `sx` and `sy` around the point `(cx, cy)`.
    fn scale_around(&mut self, sx: f64, sy: f64, cx: f64, cy: f64) {
        let transform = Affine::translate((cx, cy))
            * Affine::scale_non_uniform(sx, sy)
            * Affine::translate((-cx, -cy));
        self.apply_affine(&transform);
    }

    /// Rotate the geometry by `theta` radians around the origin.
    fn rotate(&mut self, theta: f64) {
        self.apply_affine(&Affine::rotate(theta));
    }

    /// Rotate the geometry by `theta` radians around the point `(cx, cy)`.
    fn rotate_around(&mut self, theta: f64, cx: f64, cy: f64) {
        let transform =
            Affine::translate((cx, cy)) * Affine::rotate(theta) * Affine::translate((-cx, -cy));
        self.apply_affine(&transform);
    }

    /// Skew the geometry by `kx` and `ky` radians around the origin.
    fn skew(&mut self, kx: f64, ky: f64) {
        self.apply_affine(&skew_affine(kx, ky));
    }

    /// Skew the geometry by `kx` and `ky` radians around the point `(cx, cy)`.
    fn skew_around(&mut self, kx: f64, ky: f64, cx: f64, cy: f64) {
        let transform =
            Affine::translate((cx, cy)) * skew_affine(kx, ky) * Affine::translate((-cx, -cy));
        self.apply_affine(&transform);
    }
}

impl Transforms for BezPath {
    fn apply_affine(&mut self, affine: &Affine) {
        self.apply_affine(*affine);
    }
}

impl Transforms for Polyline {
    fn apply_affine(&mut self, affine: &Affine) {
        for point in self.iter_mut() {
            *point = *affine * *point;
        }
    }
}

impl<T: Transforms + PathType> Transforms for PathImpl<T> {
    fn apply_affine(&mut self, affine: &Affine) {
        self.data.apply_affine(affine);
    }
}

impl<T: Transforms + PathType> Transforms for LayerImpl<T> {
    fn apply_affine(&mut self, affine: &Affine) {
        self.paths.iter_mut().for_each(|path| {
            path.apply_affine(affine);
        });
    }
}

impl<T: Transforms + PathType> Transforms for DocumentImpl<T> {
    fn apply_affine(&mut self, affine: &Affine) {
        self.layers.iter_mut().for_each(|(_, layer)| {
            layer.apply_affine(affine);
        });
    }
}

// TODO: tests
