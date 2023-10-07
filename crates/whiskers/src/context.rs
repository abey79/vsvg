use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use std::ops::Range;
use vsvg::Point;

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
    pub fn rng_range<T: SampleUniform + PartialOrd>(&mut self, range: Range<T>) -> T {
        if range.is_empty() {
            return range.start;
        }
        self.rng.gen_range(range)
    }

    /// Helper function to generate a random boolean value
    pub fn rng_bool(&mut self) -> bool {
        self.rng.gen_bool(0.5)
    }

    /// Helper function to return a random item from a slice
    ///
    /// # Panics
    ///
    /// Panics if the slice is empty.
    pub fn rng_choice<'a, T>(&mut self, choices: &'a impl AsRef<[T]>) -> &'a T {
        let index = self.rng_range(Range {
            start: 0usize,
            end: choices.as_ref().len(),
        });

        choices.as_ref().get(index).unwrap()
    }

    /// Helper function to return a random vsvg Point
    pub fn rng_point(&mut self, x_range: Range<f64>, y_range: Range<f64>) -> Point {
        let x = self.rng_range(x_range);
        let y = self.rng_range(y_range);

        Point::new(x, y)
    }
}
