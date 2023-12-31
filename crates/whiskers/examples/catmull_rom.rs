use whiskers::prelude::*;

#[sketch_app]
struct CatmullRomSketch {
    #[param(slider, min = 3, max = 1500)]
    num_points: usize,

    #[param(logarithmic, min = 0.01, max = 100.0)]
    tension: f64,

    negative_tension: bool,
}

impl Default for CatmullRomSketch {
    fn default() -> Self {
        Self {
            num_points: 10,
            tension: 1.0,
            negative_tension: false,
        }
    }
}

impl App for CatmullRomSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED);

        let points = (0..self.num_points)
            .map(|_| ctx.rng_point(50.0..sketch.width() - 50.0, 50.0..sketch.height() - 50.0))
            .collect::<Vec<_>>();

        for pts in &points {
            sketch.circle(pts.x(), pts.y(), 1.);
        }

        sketch.catmull_rom(
            points,
            self.tension * if self.negative_tension { -1. } else { 1. },
        );

        Ok(())
    }
}

fn main() -> Result {
    CatmullRomSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::Center)
        .run()
}
