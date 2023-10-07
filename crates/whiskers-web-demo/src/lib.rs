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

    #[param(slider, min = 0., max = 10.)]
    stroke_width: f64,
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
            stroke_width: 1.0,
        }
    }
}

impl App for WhiskersDemoSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.scale(Unit::Cm).stroke_width(self.stroke_width);

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

wasm_sketch!(Runner::new(WhiskersDemoSketch::default())
    .with_layout_options(LayoutOptions::centered())
    .with_info_options(
        InfoOptions::default()
            .description(
                "This sketch is a recreation of the classic \"Schotter\" series by Georg Nees \
            (1968-1970).\n\nGeorg Nees (born 1926, Nuremberg) is considered one of the founders \
            of computer art and graphics. He was also one of the first people to exhibit his \
            computer graphics, at the studio gallery of the Technische Hochschule in Stuttgart in \
            February 1965. In 1969, he received his doctorate on the subject of Generative \
            Computer Graphics."
            )
            .author("Antoine Beyeler")
            .author_url("https://bylr.info/")
            .source_url(
                "https://github.com/abey79/vsvg/blob/master/crates/whiskers-web-demo/src/lib.rs"
            )
    ));
