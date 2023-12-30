use crate::Unit;
use std::fmt;

pub const PAGE_SIZES: [PageSize; 22] = [
    PageSize::A6V,
    PageSize::A6H,
    PageSize::A5V,
    PageSize::A5H,
    PageSize::A4V,
    PageSize::A4H,
    PageSize::A3V,
    PageSize::A3H,
    PageSize::A2V,
    PageSize::A2H,
    PageSize::A1V,
    PageSize::A1H,
    PageSize::A0V,
    PageSize::A0H,
    PageSize::LetterV,
    PageSize::LetterH,
    PageSize::LegalV,
    PageSize::LegalH,
    PageSize::ExecutiveV,
    PageSize::ExecutiveH,
    PageSize::TabloidV,
    PageSize::TabloidH,
];

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum PageSize {
    A6V,
    A6H,
    A5V,
    A5H,
    A4V,
    A4H,
    A3V,
    A3H,
    A2V,
    A2H,
    A1V,
    A1H,
    A0V,
    A0H,
    LetterV,
    LetterH,
    LegalV,
    LegalH,
    ExecutiveV,
    ExecutiveH,
    TabloidV,
    TabloidH,
    Custom(f64, f64, Unit),
}

// macro to convert a float literal from mm to pixels
macro_rules! mm {
    ($x:expr) => {
        ($x) * 96.0 / 25.4
    };
}

impl PageSize {
    const A6_SIZE: (f64, f64) = (mm!(105.0), mm!(148.0));
    const A5_SIZE: (f64, f64) = (mm!(148.0), mm!(210.0));
    const A4_SIZE: (f64, f64) = (mm!(210.0), mm!(297.0));
    const A3_SIZE: (f64, f64) = (mm!(297.0), mm!(420.0));
    const A2_SIZE: (f64, f64) = (mm!(420.0), mm!(594.0));
    const A1_SIZE: (f64, f64) = (mm!(594.0), mm!(841.0));
    const A0_SIZE: (f64, f64) = (mm!(841.0), mm!(1189.0));
    const LETTER_SIZE: (f64, f64) = (mm!(215.9), mm!(279.4));
    const LEGAL_SIZE: (f64, f64) = (mm!(215.9), mm!(355.6));
    const EXECUTIVE_SIZE: (f64, f64) = (mm!(185.15), mm!(266.7));
    const TABLOID_SIZE: (f64, f64) = (mm!(279.4), mm!(431.8));

    /// Create new [`PageSize`] from width and height in pixels.
    ///
    /// This function attempts to match the given width and height to a standard page size and
    /// defaults to a [`PageSize::Custom`] if no match is found.
    #[must_use]
    pub fn new(mut w: f64, mut h: f64) -> Self {
        let flip = if w > h {
            std::mem::swap(&mut w, &mut h);
            true
        } else {
            false
        };

        let page_size = if (w, h) == Self::A6_SIZE {
            Self::A6V
        } else if (w, h) == Self::A5_SIZE {
            Self::A5V
        } else if (w, h) == Self::A4_SIZE {
            Self::A4V
        } else if (w, h) == Self::A3_SIZE {
            Self::A3V
        } else if (w, h) == Self::A2_SIZE {
            Self::A2V
        } else if (w, h) == Self::A1_SIZE {
            Self::A1V
        } else if (w, h) == Self::A0_SIZE {
            Self::A0V
        } else if (w, h) == Self::LETTER_SIZE {
            Self::LetterV
        } else if (w, h) == Self::LEGAL_SIZE {
            Self::LegalV
        } else if (w, h) == Self::EXECUTIVE_SIZE {
            Self::ExecutiveV
        } else if (w, h) == Self::TABLOID_SIZE {
            Self::TabloidV
        } else {
            Self::Custom(w, h, Unit::Px)
        };

        if flip {
            page_size.flip()
        } else {
            page_size
        }
    }

    /// Create a [`PageSize::Custom`] from width and height in the given [`Unit`].
    #[must_use]
    pub const fn custom(w: f64, h: f64, unit: Unit) -> Self {
        Self::Custom(w, h, unit)
    }

    /// Flip the page size from portrait to landscape or vice versa.
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            PageSize::A6V => PageSize::A6H,
            PageSize::A6H => PageSize::A6V,
            PageSize::A5V => PageSize::A5H,
            PageSize::A5H => PageSize::A5V,
            PageSize::A4V => PageSize::A4H,
            PageSize::A4H => PageSize::A4V,
            PageSize::A3V => PageSize::A3H,
            PageSize::A3H => PageSize::A3V,
            PageSize::A2V => PageSize::A2H,
            PageSize::A2H => PageSize::A2V,
            PageSize::A1V => PageSize::A1H,
            PageSize::A1H => PageSize::A1V,
            PageSize::A0V => PageSize::A0H,
            PageSize::A0H => PageSize::A0V,
            PageSize::LetterV => PageSize::LetterH,
            PageSize::LetterH => PageSize::LetterV,
            PageSize::LegalV => PageSize::LegalH,
            PageSize::LegalH => PageSize::LegalV,
            PageSize::ExecutiveV => PageSize::ExecutiveH,
            PageSize::ExecutiveH => PageSize::ExecutiveV,
            PageSize::TabloidV => PageSize::TabloidH,
            PageSize::TabloidH => PageSize::TabloidV,
            PageSize::Custom(w, h, unit) => PageSize::Custom(h, w, unit),
        }
    }

    #[must_use]
    pub fn to_pixels(&self) -> (f64, f64) {
        match self {
            // portrait
            Self::A6V => Self::A6_SIZE,
            Self::A5V => Self::A5_SIZE,
            Self::A4V => Self::A4_SIZE,
            Self::A3V => Self::A3_SIZE,
            Self::A2V => Self::A2_SIZE,
            Self::A1V => Self::A1_SIZE,
            Self::A0V => Self::A0_SIZE,
            Self::LetterV => Self::LETTER_SIZE,
            Self::LegalV => Self::LEGAL_SIZE,
            Self::ExecutiveV => Self::EXECUTIVE_SIZE,
            Self::TabloidV => Self::TABLOID_SIZE,

            // landscape
            Self::A6H => (Self::A6_SIZE.1, Self::A6_SIZE.0),
            Self::A5H => (Self::A5_SIZE.1, Self::A5_SIZE.0),
            Self::A4H => (Self::A4_SIZE.1, Self::A4_SIZE.0),
            Self::A3H => (Self::A3_SIZE.1, Self::A3_SIZE.0),
            Self::A2H => (Self::A2_SIZE.1, Self::A2_SIZE.0),
            Self::A1H => (Self::A1_SIZE.1, Self::A1_SIZE.0),
            Self::A0H => (Self::A0_SIZE.1, Self::A0_SIZE.0),
            Self::LetterH => (Self::LETTER_SIZE.1, Self::LETTER_SIZE.0),
            Self::LegalH => (Self::LEGAL_SIZE.1, Self::LEGAL_SIZE.0),
            Self::ExecutiveH => (Self::EXECUTIVE_SIZE.1, Self::EXECUTIVE_SIZE.0),
            Self::TabloidH => (Self::TABLOID_SIZE.1, Self::TABLOID_SIZE.0),

            Self::Custom(w, h, unit) => ((*w * unit).into(), (*h * unit).into()),
        }
    }

    #[must_use]
    pub fn w(&self) -> f64 {
        self.to_pixels().0
    }

    #[must_use]
    pub fn h(&self) -> f64 {
        self.to_pixels().1
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "a6" | "a6 (v)" => Some(Self::A6V),
            "a5" | "a5 (v)" => Some(Self::A5V),
            "a4" | "a4 (v)" => Some(Self::A4V),
            "a3" | "a3 (v)" => Some(Self::A3V),
            "a2" | "a2 (v)" => Some(Self::A2V),
            "a1" | "a1 (v)" => Some(Self::A1V),
            "a0" | "a0 (v)" => Some(Self::A0V),
            "letter" | "letter (v)" => Some(Self::LetterV),
            "legal" | "legal (v)" => Some(Self::LegalV),
            "executive" | "executive (v)" => Some(Self::ExecutiveV),
            "tabloid" | "tabloid (v)" => Some(Self::TabloidV),
            "a6 (h)" => Some(Self::A6H),
            "a5 (h)" => Some(Self::A5H),
            "a4 (h)" => Some(Self::A4H),
            "a3 (h)" => Some(Self::A3H),
            "a2 (h)" => Some(Self::A2H),
            "a1 (h)" => Some(Self::A1H),
            "a0 (h)" => Some(Self::A0H),
            "letter (h)" => Some(Self::LetterH),
            "legal (h)" => Some(Self::LegalH),
            "executive (h)" => Some(Self::ExecutiveH),
            "tabloid (h)" => Some(Self::TabloidH),
            _ => None, //TODO: implement WWxHHunit
        }
    }

    #[must_use]
    pub fn to_format(&self) -> Option<&'static str> {
        match self {
            Self::A6V => Some("A6 (V)"),
            Self::A5V => Some("A5 (V)"),
            Self::A4V => Some("A4 (V)"),
            Self::A3V => Some("A3 (V)"),
            Self::A2V => Some("A2 (V)"),
            Self::A1V => Some("A1 (V)"),
            Self::A0V => Some("A0 (V)"),
            Self::LetterV => Some("Letter (V)"),
            Self::LegalV => Some("Legal (V)"),
            Self::ExecutiveV => Some("Executive (V)"),
            Self::TabloidV => Some("Tabloid (V)"),
            Self::A6H => Some("A6 (H)"),
            Self::A5H => Some("A5 (H)"),
            Self::A4H => Some("A4 (H)"),
            Self::A3H => Some("A3 (H)"),
            Self::A2H => Some("A2 (H)"),
            Self::A1H => Some("A1 (H)"),
            Self::A0H => Some("A0 (H)"),
            Self::LetterH => Some("Letter (H)"),
            Self::LegalH => Some("Legal (H)"),
            Self::ExecutiveH => Some("Executive (H)"),
            Self::TabloidH => Some("Tabloid (H)"),
            Self::Custom(_, _, _) => None,
        }
    }
}

impl From<PageSize> for (f64, f64) {
    fn from(page_size: PageSize) -> Self {
        page_size.to_pixels()
    }
}

impl fmt::Display for PageSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A6V => write!(f, "A6 (V)")?,
            Self::A5V => write!(f, "A5 (V)")?,
            Self::A4V => write!(f, "A4 (V)")?,
            Self::A3V => write!(f, "A3 (V)")?,
            Self::A2V => write!(f, "A2 (V)")?,
            Self::A1V => write!(f, "A1 (V)")?,
            Self::A0V => write!(f, "A0 (V)")?,
            Self::LetterV => write!(f, "Letter (V)")?,
            Self::LegalV => write!(f, "Legal (V)")?,
            Self::ExecutiveV => write!(f, "Executive (V)")?,
            Self::TabloidV => write!(f, "Tabloid (V)")?,
            Self::A6H => write!(f, "A6 (H)")?,
            Self::A5H => write!(f, "A5 (H)")?,
            Self::A4H => write!(f, "A4 (H)")?,
            Self::A3H => write!(f, "A3 (H)")?,
            Self::A2H => write!(f, "A2 (H)")?,
            Self::A1H => write!(f, "A1 (H)")?,
            Self::A0H => write!(f, "A0 (H)")?,
            Self::LetterH => write!(f, "Letter (H)")?,
            Self::LegalH => write!(f, "Legal (H)")?,
            Self::ExecutiveH => write!(f, "Executive (H)")?,
            Self::TabloidH => write!(f, "Tabloid (H)")?,
            Self::Custom(w, h, unit) => write!(f, "{:.1}x{:.1}{}", w, h, unit.to_str())?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_page_size_display() {
        assert_eq!(format!("{}", PageSize::A6V), "A6 (V)");
        assert_eq!(format!("{}", PageSize::LegalV), "Legal (V)");
        assert_eq!(
            format!("{}", PageSize::Custom(100.0, 200.0, Unit::Px)),
            "100.0x200.0px"
        );
    }

    #[test]
    fn test_page_size_parse() {
        assert_eq!(PageSize::parse("A6"), Some(PageSize::A6V));
        assert_eq!(PageSize::parse("A5 (h)"), Some(PageSize::A5H));
        assert_eq!(PageSize::parse("A4"), Some(PageSize::A4V));
        assert_eq!(PageSize::parse("A3 (v)"), Some(PageSize::A3V));
        assert_eq!(PageSize::parse("A2"), Some(PageSize::A2V));
        assert_eq!(PageSize::parse("A1"), Some(PageSize::A1V));
        assert_eq!(PageSize::parse("A0"), Some(PageSize::A0V));
        assert_eq!(PageSize::parse("Letter"), Some(PageSize::LetterV));
        assert_eq!(PageSize::parse("Legal"), Some(PageSize::LegalV));
        assert_eq!(PageSize::parse("Executive"), Some(PageSize::ExecutiveV));
        assert_eq!(PageSize::parse("Tabloid"), Some(PageSize::TabloidV));

        //TODO: this should work
        assert_eq!(PageSize::parse("100x200px"), None);
    }

    #[test]
    fn test_page_size_flip() {
        for page_size in PAGE_SIZES {
            assert_eq!(page_size.flip().flip(), page_size);
        }

        assert_eq!(PageSize::A6V.flip(), PageSize::A6H);
        assert_eq!(PageSize::A5V.flip(), PageSize::A5H);
        assert_eq!(PageSize::A4V.flip(), PageSize::A4H);
        assert_eq!(PageSize::A3V.flip(), PageSize::A3H);
        assert_eq!(PageSize::A2V.flip(), PageSize::A2H);
        assert_eq!(PageSize::A1V.flip(), PageSize::A1H);
        assert_eq!(PageSize::A0V.flip(), PageSize::A0H);
        assert_eq!(PageSize::LetterV.flip(), PageSize::LetterH);
        assert_eq!(PageSize::LegalV.flip(), PageSize::LegalH);
        assert_eq!(PageSize::ExecutiveV.flip(), PageSize::ExecutiveH);
        assert_eq!(PageSize::TabloidV.flip(), PageSize::TabloidH);

        assert_eq!(
            PageSize::custom(100.0, 200.0, Unit::Px).flip(),
            PageSize::custom(200.0, 100.0, Unit::Px)
        );
    }
}
