use crate::{PathDataTrait, Transforms};

pub mod builtin;

#[derive(Clone, Debug)]
pub struct Glyph {
    lt: f64,
    rt: f64,
    path: kurbo::BezPath,
}

/// Trait for font data.
///
/// This trait enables multiple providers of font data.
pub trait Font {
    fn get(path: &str) -> Option<Self>
    where
        Self: Sized;

    fn glyph(&self, c: char) -> Option<Glyph>;

    fn height(&self) -> f64;
}

pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Fundamental text layout function.
///
/// Returns a list of glyphs and the total width of the line.
fn basic_text_line<F: Font>(
    text: &str,
    font: &F,
    glyph_spacing: f64,
    word_spacing: f64,
) -> (Vec<kurbo::BezPath>, f64) {
    let mut x = 0.0;

    let glyphs = text
        .chars()
        .filter_map(|ch| {
            if let Some(Glyph { lt, rt, mut path }) = font.glyph(ch) {
                path.translate(x - lt, 0.0);
                x += rt - lt + glyph_spacing;
                if ch == ' ' {
                    x += word_spacing;
                }

                Some(path)
            } else {
                x += glyph_spacing;

                None
            }
        })
        .collect();

    (glyphs, x)
}

fn glyphs_bounds<'a>(glyphs: impl Iterator<Item = &'a kurbo::BezPath>) -> Option<kurbo::Rect> {
    glyphs.fold(None, |acc, path| {
        let bounds = path.bounds();
        if let Some(acc) = acc {
            Some(acc.union(bounds))
        } else {
            Some(bounds)
        }
    })
}

pub fn text_line<F: Font>(
    text: &str,
    font: &F,
    size: f64,
    align: TextAlign,
    extra_spacing: f64,
) -> Vec<kurbo::BezPath> {
    let (mut glyphs, mut line_width) = basic_text_line(text, font, extra_spacing, 0.0);

    let scale = size / font.height();
    glyphs.scale(scale);
    line_width *= scale;

    match align {
        TextAlign::Left => {}
        TextAlign::Center => {
            glyphs.translate(-line_width / 2.0, 0.0);
        }
        TextAlign::Right => {
            glyphs.translate(-line_width, 0.0);
        }
    }

    glyphs
}
