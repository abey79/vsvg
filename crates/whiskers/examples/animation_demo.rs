//! This example demonstrates how animation can be made based on the time information provided by
//! the [`whiskers::Context`] structure and the related UI.

use std::f64::consts::TAU;
use whiskers::prelude::*;

#[derive(Sketch)]
struct AnimationDemoSketch {
    #[param(min = 3, max = 15)]
    ngon_sides: usize,
    ngon_radius: f64,
    bars_radius: f64,
    circles_radius: f64,
    circles_count: usize,
    outer_radius: f64,
}

impl Default for AnimationDemoSketch {
    fn default() -> Self {
        Self {
            ngon_sides: 5,
            ngon_radius: 30.0,
            bars_radius: 80.0,
            circles_radius: 110.0,
            circles_count: 16,
            outer_radius: 180.,
        }
    }
}

impl App for AnimationDemoSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.translate(sketch.width() / 2., sketch.height() / 2.);

        let angle = TAU * ctx.normalized_time();

        sketch.color(Color::KHAKI).stroke_width(1.0).polyline(
            (0..self.ngon_sides).map(|i| {
                let angle = angle + i as f64 * TAU / self.ngon_sides as f64;
                Point::new(
                    angle.cos() * self.ngon_radius,
                    angle.sin() * self.ngon_radius,
                )
            }),
            true,
        );

        // inner bars
        sketch.color(Color::BLACK);
        for i in 0..8 {
            let angle = -angle + i as f64 * TAU / 8.0;
            let x = angle.cos() * self.bars_radius;
            let y = angle.sin() * self.bars_radius;
            sketch.line(x / 2.0, y / 2.0, x, y);
        }

        for i in 0..self.circles_count {
            let angle = angle + i as f64 * TAU / self.circles_count as f64;
            let x = angle.cos() * self.circles_radius;
            let y = angle.sin() * self.circles_radius;
            sketch
                .color(Color::DARK_GREEN)
                .stroke_width(1.0)
                .circle(x, y, 5.0);
        }

        for i in 0..4 {
            let radius = self.outer_radius - i as f64 * 15.0;
            let angle = angle * if i % 2 == 0 { 1.0 } else { -1.0 };
            sketch
                .color(Color::DARK_RED)
                .stroke_width(3.0)
                .circle(0.0, 0.0, radius);
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            sketch
                .color(Color::DARK_BLUE)
                .stroke_width(1.0)
                .circle(x, y, 5.0);
        }

        Ok(())
    }
}

fn main() -> Result {
    AnimationDemoSketch::runner()
        .with_page_size_options(PageSize::Custom(600.0, 600.0, Unit::Px))
        // default to running the animation with the provided loop time
        .with_animation_options(AnimationOptions::looping(3.0).play())
        .run()
}
