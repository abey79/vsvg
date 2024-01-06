use vsvg::text::Font;
use whiskers::prelude::*;

#[sketch_app]
struct TextSketch {
    font_size: f64,
}

impl Default for TextSketch {
    fn default() -> Self {
        Self { font_size: 12.0 }
    }
}

fn origin_cross(sketch: &mut Sketch) {
    let size: Length = 0.5 * Unit::Mm;
    sketch.line(-size, 0., size, 0.);
    sketch.line(0., -size, 0., size);
}

impl App for TextSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED).stroke_width(0.15 * Unit::Mm);

        let font = vsvg::text::builtin::FontData::get("FUTURAL").expect("font not found");

        sketch.translate(6. * Unit::Cm, 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.add_paths(vsvg::text::text_line(
            "Hello World",
            &font,
            self.font_size,
            vsvg::text::TextAlign::Left,
            0.0,
        ));

        sketch.translate(0., 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.add_paths(vsvg::text::text_line(
            "Hello World",
            &font,
            self.font_size,
            vsvg::text::TextAlign::Center,
            0.0,
        ));

        sketch.translate(0., 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.add_paths(vsvg::text::text_line(
            "Hello World",
            &font,
            self.font_size,
            vsvg::text::TextAlign::Right,
            0.0,
        ));

        Ok(())
    }
}

fn main() -> Result {
    TextSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
