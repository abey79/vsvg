//! Example sketch to demonstrate the use of [`Length`] in sketches.

use whiskers::prelude::*;

#[sketch_app]
struct LengthSketch {
    pen_width: Length,

    unit: Unit,
}

impl Default for LengthSketch {
    fn default() -> Self {
        Self {
            pen_width: 0.15 * Unit::Mm,
            unit: Unit::Cm,
        }
    }
}

impl App for LengthSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED).stroke_width(self.pen_width);

        let u = self.unit;

        // Most methods accept anything that converts to f64, which includes `Length` (constructed
        // from a float and a `Unit`)
        sketch.line(u, 2. * u, 7. * u, 4. * u);
        sketch.translate(0., 2. * u);
        sketch.rect(4. * u, 2. * u, 6. * u, 2. * u);

        // NOTE: this is equivalent to the arguably more idiomatic following code:
        // sketch.scale(self.unit);
        // sketch.line(1., 2., 7., 4.);
        // sketch.translate(0., 2.);
        // sketch.rect(4., 2., 6., 2.);

        Ok(())
    }
}

fn main() -> Result {
    LengthSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
