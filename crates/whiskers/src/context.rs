use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use rand_distr::{Distribution, WeightedAliasIndex};
use std::{fmt::Debug, ops::Range};
use vsvg::Point;

use crate::runner::InspectVariables;

/// Context passed to [`crate::App::update`].
pub struct Context<'a> {
    /// Random number generator pre-seeded by the UI.
    pub rng: rand_chacha::ChaCha8Rng,

    /// Time value controlled by the UI.
    pub time: f64,

    /// The loop time value controlled by the UI.
    pub loop_time: f64,

    /// Debug options instance for adding debug parameters
    pub inspect_variables: &'a mut InspectVariables,
}

impl<'a> Context<'a> {
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

    /// Helper function to generate a random boolean value with a given probability of being true
    pub fn rng_weighted_bool(&mut self, prob_true: f64) -> bool {
        self.rng.gen_bool(prob_true)
    }

    /// Helper function to generate a random boolean value with a probability of being true given by the `num`/`denom`
    /// ratio.
    ///
    /// This function always returns `false` if `denom` is 0.
    pub fn rng_ratio_bool(&mut self, num: u32, denom: u32) -> bool {
        if denom == 0 {
            false
        } else {
            self.rng.gen_ratio(num, denom)
        }
    }

    /// Helper function to return a random item from a slice
    ///
    /// # Panics
    ///
    /// Panics if the slice is empty.
    pub fn rng_choice<'b, T>(&mut self, choices: &'b impl AsRef<[T]>) -> &'b T {
        let index = self.rng_range(Range {
            start: 0usize,
            end: choices.as_ref().len(),
        });

        choices.as_ref().get(index).unwrap()
    }

    /// Helper function to return a random item from a slice
    /// of tuples with a probability weight and an item.
    ///
    /// # Panics
    ///
    /// Panics if the slice is empty
    pub fn rng_weighted_choice<'b, T>(&mut self, choices: &'b impl AsRef<[(f64, T)]>) -> &'b T {
        let weights: Vec<f64> = choices.as_ref().iter().map(|choice| choice.0).collect();
        let dist = WeightedAliasIndex::new(weights).unwrap();

        &choices.as_ref().get(dist.sample(&mut self.rng)).unwrap().1
    }

    /// Helper function to return a random vsvg Point
    pub fn rng_point(&mut self, x_range: Range<f64>, y_range: Range<f64>) -> Point {
        let x = self.rng_range(x_range);
        let y = self.rng_range(y_range);

        Point::new(x, y)
    }

    /// Helper function to display an inspect parameter in the inspect variables UI
    pub fn inspect(&mut self, key: impl AsRef<str>, value: impl Debug) {
        self.inspect_variables
            .add_parameter(&(key.as_ref().to_owned(), format!("{value:?}")));
    }
}
