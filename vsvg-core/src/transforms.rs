use kurbo::Affine;

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
    fn transform(&mut self, affine: &Affine);

    /// Translate the geometry by `dx` and `dy`.
    fn translate(&mut self, dx: f64, dy: f64) {
        self.transform(&Affine::translate((dx, dy)));
    }

    /// Scale the geometry by `s` around the origin.
    fn scale(&mut self, s: f64) {
        self.transform(&Affine::scale(s));
    }

    /// Scale the geometry by `sx` and `sy` around the origin.
    fn scale_non_uniform(&mut self, sx: f64, sy: f64) {
        self.transform(&Affine::scale_non_uniform(sx, sy));
    }

    /// Scale the geometry by `sx` and `sy` around the point `(cx, cy)`.
    fn scale_around(&mut self, sx: f64, sy: f64, cx: f64, cy: f64) {
        let transform = Affine::translate((cx, cy))
            * Affine::scale_non_uniform(sx, sy)
            * Affine::translate((-cx, -cy));
        self.transform(&transform);
    }

    /// Rotate the geometry by `theta` radians around the origin.
    fn rotate(&mut self, theta: f64) {
        self.transform(&Affine::rotate(theta));
    }

    /// Rotate the geometry by `theta` radians around the point `(cx, cy)`.
    fn rotate_around(&mut self, theta: f64, cx: f64, cy: f64) {
        let transform =
            Affine::translate((cx, cy)) * Affine::rotate(theta) * Affine::translate((-cx, -cy));
        self.transform(&transform);
    }

    /// Skew the geometry by `kx` and `ky` radians around the origin.
    fn skew(&mut self, kx: f64, ky: f64) {
        self.transform(&skew_affine(kx, ky));
    }

    /// Skew the geometry by `kx` and `ky` radians around the point `(cx, cy)`.
    fn skew_around(&mut self, kx: f64, ky: f64, cx: f64, cy: f64) {
        let transform =
            Affine::translate((cx, cy)) * skew_affine(kx, ky) * Affine::translate((-cx, -cy));
        self.transform(&transform);
    }
}

// TODO: tests
