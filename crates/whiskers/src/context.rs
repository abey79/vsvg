use rand::Rng;
use std::ops::Range;

/// Context passed to [`crate::App::update`].
pub struct Context {
    /// Random number generator pre-seeded by the UI.
    pub rng: rand_chacha::ChaCha8Rng,

    /// Time value controlled by the UI.
    pub time: f64,

    /// The loop time value controlled by the UI.
    pub loop_time: f64,
}

impl Context {
    /// Time parameter, normalized by the loop time.
    ///
    /// Always returns 0.0 if the loop time is set to 0.0.
    #[must_use]
    pub fn normalized_time(&self) -> f64 {
        if self.loop_time == 0.0 {
            0.0
        } else {
            self.time / self.loop_time
        }
    }

    /// Helper function to generate a random number in a given range. This function accepts an empty
    /// range, in which case it will always return the start value.
    pub fn rng_range(&mut self, range: Range<f64>) -> f64 {
        if range.is_empty() {
            return range.start;
        }
        self.rng.gen_range(range)
    }
}
