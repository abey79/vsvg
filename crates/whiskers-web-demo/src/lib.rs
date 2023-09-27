//! Recreation of Georg Nees' ["Schotter" (1968-1970)](https://collections.vam.ac.uk/item/O221321/schotter-print-nees-georg/)
//! using whiskers.

use itertools::iproduct;
use whiskers::prelude::*;

#[derive(Sketch)]
pub struct WhiskersDemoSketch {
    col_count: u32,
    row_count: u32,

    #[param(slider, min = 0., max = 10.)]
    offset_cm: f64,

    #[param(slider, min = 0., max = 10.)]
    box_size_cm: f64,

    #[param(slider, min = 0., max = 90.)]
    rand_angle_deg: f64,

    #[param(slider, min = 0., max = 3.)]
    rand_offset_cm: f64,
}

impl Default for WhiskersDemoSketch {
    fn default() -> Self {
        Self {
            col_count: 12,
            row_count: 24,
            offset_cm: 1.,
            box_size_cm: 1.,
            rand_angle_deg: 45.,
            rand_offset_cm: 0.3,
        }
    }
}

impl App for WhiskersDemoSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.scale(Unit::Cm);

        for (i, j) in iproduct!(0..self.col_count, 0..self.row_count) {
            sketch.push_matrix_and(|sketch| {
                sketch.translate(i as f64 * self.offset_cm, j as f64 * self.offset_cm);

                let max_angle = self.rand_angle_deg * (j as f64 / self.row_count as f64);
                let max_offset = self.rand_offset_cm * (j as f64 / self.row_count as f64);

                sketch
                    .rotate_deg(ctx.rng_range(-max_angle..max_angle))
                    .translate(
                        ctx.rng_range(-max_offset..max_offset),
                        ctx.rng_range(-max_offset..max_offset),
                    )
                    .rect(0., 0., self.box_size_cm, self.box_size_cm);
            });
        }

        Ok(())
    }
}

wasm_sketch!(
    Runner::new(WhiskersDemoSketch::default()).with_layout_option(LayoutOptions::centered())
);
