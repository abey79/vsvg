use crate::Transforms;

pub mod builtin;

#[derive(Clone, Debug)]
pub struct Glyph {
    lt: f64,
    rt: f64,
    path: kurbo::BezPath,
    c: char,
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

#[derive(Clone, Copy, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum ParagraphAlign {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Clone, Debug)]
enum Command {
    Advance(f64),
    AdvanceWord(f64),
    DrawGlyph {
        path: kurbo::BezPath,
        offset: f64,
        c: char,
    },
}

fn commands_from_chars<'a, F: Font>(
    chars: impl Iterator<Item = char> + 'a,
    font: &'a F,
) -> impl Iterator<Item = Command> + 'a {
    chars
        .flat_map(|c| {
            if let Some(Glyph { lt, rt, path, c }) = font.glyph(c) {
                [
                    Some(Command::DrawGlyph {
                        path,
                        offset: -lt,
                        c,
                    }),
                    if c == ' ' {
                        Some(Command::AdvanceWord(rt - lt))
                    } else {
                        Some(Command::Advance(rt - lt))
                    },
                ]
                .into_iter()
            } else {
                // TODO: a placeholder glyph would be better here
                [Some(Command::Advance(0.0)), None].into_iter()
            }
        })
        .filter_map(|x| x)
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

    let glyphs = commands_from_chars(text.chars(), font)
        .filter_map(|cmd| match cmd {
            Command::Advance(offset) => {
                x += offset + glyph_spacing;
                None
            }
            Command::AdvanceWord(offset) => {
                x += offset + word_spacing;
                None
            }
            Command::DrawGlyph {
                mut path,
                offset,
                c: _c,
            } => {
                path.translate(x + offset, 0.0);
                Some(path)
            }
        })
        .collect();

    (glyphs, x)
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

pub fn text_paragraph<F: Font>(
    text: &str,
    font: &F,
    size: f64,
    align: ParagraphAlign,
    extra_spacing: f64,
    width: impl Into<f64>,
) -> Vec<kurbo::BezPath> {
    let mut x = 0.0;
    let mut x_word = 0.0;
    let mut y = 0.0;

    let line_spacing = 1.0 * font.height();
    let scale = size / font.height();
    let width = width.into() / scale;

    let mut cur_word = vec![];
    let commands = commands_from_chars(text.chars(), font);

    let mut line = vec![];
    let mut paragraph = vec![];

    let mut flush_line =
        |line: &mut Vec<Vec<kurbo::BezPath>>, x: f64, offset: f64, last_line: bool| {
            if !last_line && matches!(align, ParagraphAlign::Justified) {
                let space = if line.len() >= 2 {
                    (width - x + offset + extra_spacing) / (line.len() - 1) as f64
                } else {
                    0.0
                };

                line.drain(..).enumerate().for_each(|(i, mut word)| {
                    word.translate(space * i as f64, 0.0);
                    paragraph.extend(word.drain(..));
                });
            } else {
                let align_offset = match align {
                    ParagraphAlign::Left | ParagraphAlign::Justified => 0.0,
                    ParagraphAlign::Center => (width - x + offset + extra_spacing) / 2.0,
                    ParagraphAlign::Right => width - x + offset + extra_spacing,
                };

                line.translate(align_offset, 0.0);
                line.drain(..)
                    .for_each(|mut word: Vec<kurbo::BezPath>| paragraph.extend(word.drain(..)));
            }
        };

    // Always remember the last offset after a word, so we can remove it when computing line alignment
    let mut last_advance_word_offset = None;

    // Note: we prepend an `AdvanceWord`command to make sure we flush `cur_word` for the last word.
    for command in commands
        .into_iter()
        .chain(std::iter::once(Command::AdvanceWord(0.0)))
    {
        match command {
            Command::Advance(offset) => {
                x_word += offset + extra_spacing;
            }
            Command::AdvanceWord(offset) => {
                if x + x_word > width {
                    // flush the current line into the paragraph
                    flush_line(
                        &mut line,
                        x,
                        last_advance_word_offset.unwrap_or_default(),
                        false,
                    );

                    // reset coordinates
                    x = 0.0;
                    y += line_spacing;
                } else {
                }

                cur_word.translate(x, y);

                line.push(std::mem::take(&mut cur_word));

                x += x_word + offset;
                x_word = 0.0;
                last_advance_word_offset = Some(offset);
            }
            Command::DrawGlyph {
                mut path,
                offset,
                c: _c,
            } => {
                path.translate(x_word + offset, 0.0);
                cur_word.push(path);
            }
        }
    }

    // flush the last line
    flush_line(
        &mut line,
        x,
        last_advance_word_offset.unwrap_or_default(),
        true,
    );

    paragraph.scale(scale);
    paragraph
}
