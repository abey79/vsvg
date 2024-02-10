use anyhow::Ok;
use whiskers::prelude::*;

#[sketch_app]
struct DebugSketch {
    #[param(slider, min = 20.0, max = 400.0)]
    width: f64,
    #[param(slider, min = 20.0, max = 400.0)]
    height: f64,
}

impl Default for DebugSketch {
    fn default() -> Self {
        Self {
            width: 130.0,
            height: 130.0,
        }
    }
}

impl App for DebugSketch {
    fn update(&mut self, sketch: &mut Sketch, ctx: &mut Context) -> anyhow::Result<()> {
        sketch.stroke_width(Unit::Mm * 4.0);
        sketch.color(Color::RED);
        sketch.rect(0.0, 0.0, self.width, self.height);

        ctx.debug("Square area", (self.width * self.height).round());

        Ok(())
    }
}

fn main() -> Result {
    DebugSketch::runner()
        .with_layout_options(LayoutOptions::Center)
        .with_debug_options(DebugOptions::default().label("Computed properties"))
        .run()
}
