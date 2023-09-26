//! Asteroid design kindly contributed by @Wyth@mastodon.art for my
//! [Rusteroïds](https://github.com/abey79/rusteroids) game.

use whiskers::prelude::*;

#[derive(Sketch)]
pub struct WhiskersDemoSketch {
    col_count: u32,
    row_count: u32,

    #[param(slider, min = 0., max = 10.)]
    offset_cm: f64,

    #[param(slider, min = 0., max = 10.)]
    box_size_cm: f64,
    randomness: f64,
}

impl Default for WhiskersDemoSketch {
    fn default() -> Self {
        Self {
            col_count: 12,
            row_count: 24,
            offset_cm: 1.,
            box_size_cm: 0.8,
            randomness: 0.1,
        }
    }
}

impl App for WhiskersDemoSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.scale_unit(Unit::CM);

        //TODO: this is not ergonomic, mul/div with Unit should yield to float and scale_unit
        // should go.
        sketch.translate(
            (sketch.width() / Unit::CM.to_px() - self.col_count as f64 * self.offset_cm) / 2.0,
            (sketch.height() / Unit::CM.to_px() - self.row_count as f64 * self.offset_cm) / 2.0,
        );

        for j in 0..self.row_count {
            for i in 0..self.col_count {
                sketch.rect(
                    i as f64 * self.offset_cm,
                    j as f64 * self.offset_cm,
                    self.box_size_cm,
                    self.box_size_cm,
                );
            }
        }
        Ok(())
    }
}

wasm_sketch!(Runner::new(WhiskersDemoSketch::default()).with_time_enabled(false));
