/// A length unit
///
/// Doc TODO:
/// - Mul
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit(f64, &'static str);

impl From<Unit> for f64 {
    fn from(unit: Unit) -> Self {
        unit.0
    }
}

impl From<Unit> for f32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(unit: Unit) -> Self {
        unit.0 as f32
    }
}

pub const UNITS: [Unit; 11] = [
    Unit::PX,
    Unit::IN,
    Unit::FT,
    Unit::YD,
    Unit::MI,
    Unit::MM,
    Unit::CM,
    Unit::M,
    Unit::KM,
    Unit::PC,
    Unit::PT,
];

impl Unit {
    pub const PX: Unit = Unit(1.0, "px");
    pub const IN: Unit = Unit(96.0, "in");
    pub const FT: Unit = Unit(12.0 * 96.0, "ft");
    pub const YD: Unit = Unit(36.0 * 96.0, "yd");
    pub const MI: Unit = Unit(1760.0 * 36.0 * 96.0, "mi");
    pub const MM: Unit = Unit(96.0 / 25.4, "mm");
    pub const CM: Unit = Unit(96.0 / 2.54, "cm");
    pub const M: Unit = Unit(100.0 * 96.0 / 2.54, "m");
    pub const KM: Unit = Unit(100_000.0 * 96.0 / 2.54, "km");
    pub const PC: Unit = Unit(16.0, "pc");
    pub const PT: Unit = Unit(96.0 / 72.0, "pt");

    #[must_use]
    pub fn to_px(&self) -> f64 {
        self.0
    }

    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        self.1
    }

    #[must_use]
    pub fn from(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "px" | "pixel" => Some(Unit::PX),
            "in" | "inch" => Some(Unit::IN),
            "ft" | "feet" => Some(Unit::FT),
            "yd" | "yard" => Some(Unit::YD),
            "mi" | "mile" | "miles" => Some(Unit::MI),
            "mm" | "millimeter" | "millimetre" => Some(Unit::MM),
            "cm" | "centimeter" | "centimetre" => Some(Unit::CM),
            "m" | "meter" | "metre" => Some(Unit::M),
            "km" | "kilometer" | "kilometre" => Some(Unit::KM),
            "pc" | "pica" => Some(Unit::PC),
            "pt" | "point" | "points" => Some(Unit::PT),
            _ => None,
        }
    }
}

impl std::ops::Mul<Unit> for f64 {
    type Output = Unit;

    fn mul(self, rhs: Unit) -> Self::Output {
        Unit(self * rhs.0, rhs.1)
    }
}

impl std::ops::Mul<f64> for Unit {
    type Output = Unit;

    fn mul(self, rhs: f64) -> Self::Output {
        Unit(self.0 * rhs, self.1)
    }
}
