use vsvg::text::Font;
use whiskers::prelude::*;

#[sketch_app]
struct TextSketch {
    font_size: f64,
    glyph_spacing: f64,
    text_width: Length,
    paragraph_text: String,
}

impl Default for TextSketch {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            glyph_spacing: 0.0,
            text_width: 5.0 * Unit::Cm,
            paragraph_text: "Lorem ipsum dolor sit amet. Honi soit qui mal y pense. \
                Pierre qui roule n'amasse pas mousse."
                .to_owned(),
        }
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
            self.glyph_spacing,
        ));

        sketch.translate(0., 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.add_paths(vsvg::text::text_line(
            "Hello World",
            &font,
            self.font_size,
            vsvg::text::TextAlign::Center,
            self.glyph_spacing,
        ));

        sketch.translate(0., 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.add_paths(vsvg::text::text_line(
            "Hello World",
            &font,
            self.font_size,
            vsvg::text::TextAlign::Right,
            self.glyph_spacing,
        ));

        sketch.translate(0., 2. * Unit::Cm);
        origin_cross(sketch);
        sketch.push_matrix_and(|sketch| {
            sketch.translate(self.text_width, 0.);
            origin_cross(sketch);
        });
        sketch.add_paths(vsvg::text::text_paragraph(
            self.paragraph_text.as_str(),
            &font,
            self.font_size,
            vsvg::text::ParagraphAlign::Left,
            self.glyph_spacing,
            self.text_width,
        ));

        sketch.translate(0., 6. * Unit::Cm);
        origin_cross(sketch);
        sketch.push_matrix_and(|sketch| {
            sketch.translate(self.text_width, 0.);
            origin_cross(sketch);
        });
        sketch.add_paths(vsvg::text::text_paragraph(
            self.paragraph_text.as_str(),
            &font,
            self.font_size,
            vsvg::text::ParagraphAlign::Center,
            self.glyph_spacing,
            self.text_width,
        ));

        sketch.translate(0., 6. * Unit::Cm);
        origin_cross(sketch);
        sketch.push_matrix_and(|sketch| {
            sketch.translate(self.text_width, 0.);
            origin_cross(sketch);
        });
        sketch.add_paths(vsvg::text::text_paragraph(
            self.paragraph_text.as_str(),
            &font,
            self.font_size,
            vsvg::text::ParagraphAlign::Right,
            self.glyph_spacing,
            self.text_width,
        ));

        sketch.translate(0., 6. * Unit::Cm);
        origin_cross(sketch);
        sketch.push_matrix_and(|sketch| {
            sketch.translate(self.text_width, 0.);
            origin_cross(sketch);
        });
        sketch.add_paths(vsvg::text::text_paragraph(
            self.paragraph_text.as_str(),
            &font,
            self.font_size,
            vsvg::text::ParagraphAlign::Justified,
            self.glyph_spacing,
            self.text_width,
        ));

        Ok(())
    }
}

fn main() -> Result {
    TextSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
