//! A simple introductory sketch demonstrating basic whiskers usage.

use whiskers::prelude::*;
use whiskers::Runner;

#[sketch_app]
pub struct HelloWorldSketch {
    width: f64,
    height: f64,
}

impl Default for HelloWorldSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
        }
    }
}

impl App for HelloWorldSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED).stroke_width(3.0);

        sketch
            .translate(sketch.width() / 2.0, sketch.height() / 2.0)
            .rect(0., 0., self.width, self.height);

        Ok(())
    }
}

/// Create a configured runner for this sketch.
pub fn runner() -> Runner<'static, HelloWorldSketch> {
    HelloWorldSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .with_layout_options(LayoutOptions::centered())
        .with_info_options(
            InfoOptions::default()
                .description("A simple introductory sketch demonstrating basic whiskers usage.")
                .author("Antoine Beyeler")
                .author_url("https://bylr.info/")
                .source_url(
                    "https://github.com/abey79/vsvg/blob/master/crates/whiskers-gallery/src/sketches/hello_world.rs",
                ),
        )
}
