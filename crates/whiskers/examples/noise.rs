//! Example sketch to demonstrate the use of the `noise-rs` crate.

use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};
use whiskers::prelude::*;

#[derive(Sketch)]
struct NoiseSketch {
    #[param(slider, min = 0.0, max = 500.0)]
    margin: f64,
    line_count: usize,
    points_per_line: usize,

    gain: f64,

    #[param(logarithmic, min = 0.1, max = 10.0)]
    x_noise_range: f64,
    #[param(logarithmic, min = 0.1, max = 10.0)]
    y_noise_range: f64,

    #[param(logarithmic, min = 0.01, max = 100.0)]
    stroke_width: f64,

    color: Color,

    use_catmull_rom: bool,
    #[param(logarithmic, min = 0.01, max = 10.0)]
    tension: f64,
}

impl Default for NoiseSketch {
    fn default() -> Self {
        Self {
            margin: 50.0,
            line_count: 400,
            points_per_line: 100,
            gain: 22.0,
            x_noise_range: 2.3,
            y_noise_range: 2.6,

            stroke_width: 1.0,
            color: Color::DARK_RED.with_opacity(0.8),

            use_catmull_rom: false,
            tension: 1.0,
        }
    }
}

impl App for NoiseSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(self.color).stroke_width(self.stroke_width);

        let dx = (sketch.width() - 2.0 * self.margin) / (self.points_per_line - 1) as f64;
        let dy = (sketch.height() - 2.0 * self.margin) / (self.line_count - 1) as f64;

        sketch.translate(self.margin, self.margin);

        let fbm = Fbm::<Perlin>::default();

        let noise_map: NoiseMap = PlaneMapBuilder::<_, 4>::new(fbm)
            .set_size(self.points_per_line, self.line_count)
            .set_x_bounds(ctx.time - self.x_noise_range, ctx.time + self.x_noise_range)
            .set_y_bounds(-self.y_noise_range, self.y_noise_range)
            .build();

        for j in 0..self.line_count {
            sketch.translate(0.0, dy);

            let points = (0..self.points_per_line)
                .map(|i| (i as f64 * dx, self.gain * dy * noise_map.get_value(i, j)));

            if self.use_catmull_rom {
                sketch.catmull_rom(points, self.tension);
            } else {
                sketch.polyline(points, false);
            }
        }

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(NoiseSketch::default())
        .with_page_size_options(PageSize::A5H)
        .run()
}
