/// Context passed to [`SketchApp::update`].
pub struct Context {
    /// Random number generator pre-seeded by the UI.
    pub rng: rand_chacha::ChaCha8Rng,

    /// Time value controlled by the UI.
    pub time: f64,
}
