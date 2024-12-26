//! Built-in fonts based on the fond data extracted from <https://github.com/fogleman/axi/blob/master/axi/hershey_fonts.py>.

mod font_data;

use crate::text::Glyph;
use crate::PathDataTrait;
pub use font_data::*;

const FONT_NAMES: &[&str] = &[
    "ASTROLOGY",
    "CURSIVE",
    "CYRILC_1",
    "CYRILLIC",
    "FUTURAL",
    "FUTURAM",
    "GOTHGBT",
    "GOTHGRT",
    "GOTHICENG",
    "GOTHICGER",
    "GOTHICITA",
    "GOTHITT",
    "GREEK",
    "GREEKC",
    "GREEKS",
    "JAPANESE",
    "MARKERS",
    "MATHLOW",
    "MATHUPP",
    "METEOROLOGY",
    "MUSIC",
    "ROWMAND",
    "ROWMANS",
    "ROWMANT",
    "SCRIPTC",
    "SCRIPTS",
    "SYMBOLIC",
    "TIMESG",
    "TIMESI",
    "TIMESIB",
    "TIMESR",
    "TIMESRB",
];

const FONTS: &[&[(i8, i8, &[&[(i8, i8)]])]] = &[
    ASTROLOGY,
    CURSIVE,
    CYRILC_1,
    CYRILLIC,
    FUTURAL,
    FUTURAM,
    GOTHGBT,
    GOTHGRT,
    GOTHICENG,
    GOTHICGER,
    GOTHICITA,
    GOTHITT,
    GREEK,
    GREEKC,
    GREEKS,
    JAPANESE,
    MARKERS,
    MATHLOW,
    MATHUPP,
    METEOROLOGY,
    MUSIC,
    ROWMAND,
    ROWMANS,
    ROWMANT,
    SCRIPTC,
    SCRIPTS,
    SYMBOLIC,
    TIMESG,
    TIMESI,
    TIMESIB,
    TIMESR,
    TIMESRB,
];

struct GlyphData {
    lt: f64,
    rt: f64,
    path: kurbo::BezPath,
}

impl GlyphData {
    fn new(data: &'static (i8, i8, &'static [&'static [(i8, i8)]])) -> Option<Self> {
        fn to_point(point: &(i8, i8)) -> kurbo::Point {
            kurbo::Point {
                x: point.0 as f64,
                y: point.1 as f64,
            }
        }

        let path: kurbo::BezPath = data
            .2
            .iter()
            .filter(|line| line.len() > 1)
            .flat_map(|line| {
                let first = to_point(&line[0]);
                std::iter::once(kurbo::PathEl::MoveTo(first))
                    .chain(line[1..].iter().map(|p| kurbo::PathEl::LineTo(to_point(p))))
            })
            .collect();

        Some(Self {
            lt: data.0 as f64,
            rt: data.1 as f64,
            path,
        })
    }
}

pub struct FontData {
    glyphs: Vec<Option<GlyphData>>,
    height: f64,
}

impl FontData {
    fn new(data: &'static [(i8, i8, &'static [&'static [(i8, i8)]])]) -> Self {
        let glyphs: Vec<_> = data.iter().map(GlyphData::new).collect();

        let mut min = 0.0f64;
        let mut max = 0.0f64;

        glyphs.iter().filter_map(|g| g.as_ref()).for_each(|g| {
            let bounds = g.path.bounds();
            min = min.min(bounds.min_y());
            max = max.max(bounds.max_y());
        });

        Self {
            glyphs,
            height: max - min,
        }
    }
}

impl super::Font for FontData {
    fn get(path: &str) -> Option<Self> {
        let font_index = FONT_NAMES
            .iter()
            .enumerate()
            .find(|(_, name)| **name == path)
            .map(|(idx, _)| idx)?;

        Some(Self::new(FONTS[font_index]))
    }

    fn glyph(&self, c: char) -> Option<Glyph> {
        let glyph_index = c as usize - 32;
        let glyph_data = self.glyphs.get(glyph_index)?.as_ref()?;

        Some(Glyph {
            lt: glyph_data.lt,
            rt: glyph_data.rt,
            path: glyph_data.path.clone(),
            c,
        })
    }

    fn height(&self) -> f64 {
        self.height
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fonts_and_fonts_name_have_matching_length() {
        assert_eq!(FONT_NAMES.len(), FONTS.len());
    }

    #[test]
    fn test_playground() {
        let font = FontData::new(FONTS[0]);
    }
}
