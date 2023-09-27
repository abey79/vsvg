/// A length unit
///
/// Doc TODO:
/// - Mul
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Unit {
    Px,
    In,
    Ft,
    Yd,
    Mi,
    Mm,
    Cm,
    M,
    Km,
    Pc,
    Pt,
}

impl From<Unit> for f64 {
    fn from(unit: Unit) -> Self {
        unit.to_px()
    }
}

impl From<Unit> for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(unit: Unit) -> Self {
        unit.to_px() as f32
    }
}

pub const UNITS: [Unit; 11] = [
    Unit::Px,
    Unit::In,
    Unit::Ft,
    Unit::Yd,
    Unit::Mi,
    Unit::Mm,
    Unit::Cm,
    Unit::M,
    Unit::Km,
    Unit::Pc,
    Unit::Pt,
];

impl Unit {
    #[must_use]
    pub fn to_px(&self) -> f64 {
        match &self {
            Self::Px => 1.0,
            Self::In => 96.0,
            Self::Ft => 12.0 * 96.0,
            Self::Yd => 36.0 * 96.0,
            Self::Mi => 1760.0 * 36.0 * 96.0,
            Self::Mm => 96.0 / 25.4,
            Self::Cm => 96.0 / 2.54,
            Self::M => 100.0 * 96.0 / 2.54,
            Self::Km => 100_000.0 * 96.0 / 2.54,
            Self::Pc => 16.0,
            Self::Pt => 96.0 / 72.0,
        }
    }

    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match &self {
            Self::Px => "px",
            Self::In => "in",
            Self::Ft => "ft",
            Self::Yd => "yd",
            Self::Mi => "mi",
            Self::Mm => "mm",
            Self::Cm => "cm",
            Self::M => "m",
            Self::Km => "km",
            Self::Pc => "pc",
            Self::Pt => "pt",
        }
    }

    #[must_use]
    pub fn from(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "px" | "pixel" => Some(Unit::Px),
            "in" | "inch" => Some(Unit::In),
            "ft" | "feet" => Some(Unit::Ft),
            "yd" | "yard" => Some(Unit::Yd),
            "mi" | "mile" | "miles" => Some(Unit::Mi),
            "mm" | "millimeter" | "millimetre" => Some(Unit::Mm),
            "cm" | "centimeter" | "centimetre" => Some(Unit::Cm),
            "m" | "meter" | "metre" => Some(Unit::M),
            "km" | "kilometer" | "kilometre" => Some(Unit::Km),
            "pc" | "pica" => Some(Unit::Pc),
            "pt" | "point" | "points" => Some(Unit::Pt),
            _ => None,
        }
    }
}

impl std::ops::Mul<Unit> for f64 {
    type Output = f64;

    fn mul(self, rhs: Unit) -> f64 {
        self * rhs.to_px()
    }
}

impl std::ops::Mul<f64> for Unit {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.to_px() * rhs
    }
}

impl std::ops::Div<Unit> for f64 {
    type Output = f64;

    fn div(self, rhs: Unit) -> f64 {
        self / rhs.to_px()
    }
}

impl std::ops::Div<f64> for Unit {
    type Output = f64;

    fn div(self, rhs: f64) -> Self::Output {
        self.to_px() / rhs
    }
}
