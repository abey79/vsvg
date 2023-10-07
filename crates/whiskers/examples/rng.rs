use std::ops::Range;
use vsvg::COLORS;
use whiskers::prelude::*;

#[derive(Sketch)]
struct RngSketch {
    width: f64,
    height: f64,
}

impl Default for RngSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
        }
    }
}

impl App for RngSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(3.0);

        let should_generate_random_color = ctx.rng_bool();

        println!(
            "Was a random color generated? {}",
            if should_generate_random_color {
                "Yes"
            } else {
                "No"
            }
        );

        let colors = COLORS.to_vec();
        let chosen_color = if should_generate_random_color {
            ctx.rng_choice(&colors)
        } else {
            &Color::BLACK
        };

        println!("{}", chosen_color.to_rgb_string());

        sketch.color(*chosen_color);
        sketch
            .translate(sketch.width() / 2.0, sketch.height() / 2.0)
            .rect(0., 0., self.width, self.height);

        let x_range = Range {
            start: 0.0,
            end: sketch.width(),
        };
        let y_range = Range {
            start: 0.0,
            end: sketch.height(),
        };

        let some_point = ctx.rng_point(x_range, y_range);

        sketch.push_matrix_reset();
        sketch.circle(some_point.x(), some_point.y(), 20.0);

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(RngSketch::default())
        .with_random_seed()
        .with_page_size_options(PageSize::A5H)
        .run()
}
