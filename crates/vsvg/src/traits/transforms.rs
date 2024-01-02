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
    fn transform(&mut self, affine: &Affine) -> &mut Self;

    /// Translate the geometry by `dx` and `dy`.
    #[inline]
    fn translate(&mut self, dx: impl Into<f64>, dy: impl Into<f64>) -> &mut Self {
        self.transform(&Affine::translate((dx.into(), dy.into())));
        self
    }

    /// Scale the geometry by `s` around the origin.
    #[inline]
    fn scale(&mut self, s: impl Into<f64>) -> &mut Self {
        self.transform(&Affine::scale(s.into()));
        self
    }

    /// Scale the geometry by `sx` and `sy` around the origin.
    #[inline]
    fn scale_non_uniform(&mut self, sx: impl Into<f64>, sy: impl Into<f64>) -> &mut Self {
        self.transform(&Affine::scale_non_uniform(sx.into(), sy.into()));
        self
    }

    /// Scale the geometry by `sx` and `sy` around the point `(cx, cy)`.
    #[inline]
    fn scale_around(
        &mut self,
        sx: impl Into<f64>,
        sy: impl Into<f64>,
        cx: impl Into<f64>,
        cy: impl Into<f64>,
    ) -> &mut Self {
        let (cx, cy) = (cx.into(), cy.into());
        let transform = Affine::translate((cx, cy))
            * Affine::scale_non_uniform(sx.into(), sy.into())
            * Affine::translate((-cx, -cy));
        self.transform(&transform);
        self
    }

    /// Rotate the geometry by `theta` radians around the origin.
    #[inline]
    fn rotate(&mut self, theta: f64) -> &mut Self {
        self.transform(&Affine::rotate(theta));
        self
    }

    /// Rotate the geometry by `theta` degrees around the origin.
    #[inline]
    fn rotate_deg(&mut self, theta: f64) -> &mut Self {
        self.rotate(theta.to_radians())
    }

    /// Rotate the geometry by `theta` radians around the point `(cx, cy)`.
    #[inline]
    fn rotate_around(&mut self, theta: f64, cx: impl Into<f64>, cy: impl Into<f64>) -> &mut Self {
        let (cx, cy) = (cx.into(), cy.into());
        let transform =
            Affine::translate((cx, cy)) * Affine::rotate(theta) * Affine::translate((-cx, -cy));
        self.transform(&transform);
        self
    }

    /// Rotate the geometry by `theta` degrees around the point `(cx, cy)`.
    #[inline]
    fn rotate_around_deg(
        &mut self,
        theta: f64,
        cx: impl Into<f64>,
        cy: impl Into<f64>,
    ) -> &mut Self {
        self.rotate_around(theta.to_radians(), cx.into(), cy.into())
    }

    /// Skew the geometry by `kx` and `ky` radians around the origin.
    #[inline]
    fn skew(&mut self, kx: impl Into<f64>, ky: impl Into<f64>) -> &mut Self {
        self.transform(&skew_affine(kx.into(), ky.into()));
        self
    }

    /// Skew the geometry by `kx` and `ky` radians around the point `(cx, cy)`.
    #[inline]
    fn skew_around(
        &mut self,
        kx: impl Into<f64>,
        ky: impl Into<f64>,
        cx: impl Into<f64>,
        cy: impl Into<f64>,
    ) -> &mut Self {
        let (cx, cy) = (cx.into(), cy.into());
        let transform = Affine::translate((cx, cy))
            * skew_affine(kx.into(), ky.into())
            * Affine::translate((-cx, -cy));
        self.transform(&transform);
        self
    }
}

// TODO: tests
