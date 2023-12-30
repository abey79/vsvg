use std::fmt::{Display, Formatter};

use num_traits::{AsPrimitive, Float};

use crate::Length;

/// Errors related to [`Unit`]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum UnitError {
    #[error("Unrecognised unit '{0}'")]
    UnrecognisedError(String),
}

/// A distance unit.
///
/// Combined with [`Length`], can be used to manipulate physical length and convert between units.
/// Conversion from/to strings is also supported.
///
/// Convert between units:
/// ```
/// # use vsvg::{Unit, Length};
/// let f = 2.54f32;
/// assert_eq!(Unit::In.convert_from(&Unit::Cm, f), 1.0f32);
/// assert_eq!(Unit::Cm.convert_to(&Unit::In, f), 1.0f32);
/// ```
///
/// Create a [`Length`] via multiplication:
/// ```
/// # use vsvg::{Unit, Length};
/// assert_eq!(30.0f32 * Unit::Cm, Length { value: 30.0, unit: Unit::Cm });
/// assert_eq!(Unit::Mm * 25.0f64, Length { value: 25.0, unit: Unit::Mm });
/// ```
///
/// Convert to/from strings:
/// ```
/// # use vsvg::{Unit, Length};
/// assert_eq!(Unit::Cm.to_str(), "cm");
/// assert_eq!(Unit::Yd.to_str(), "yd");
///
/// assert_eq!(Unit::try_from("m"), Ok(Unit::M));
/// assert_eq!(Unit::try_from("kilometre"), Ok(Unit::Km));
/// ```
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

/// List of all available units, useful for iteration.
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
    /// Convert the unit into the corresponding pixel factor.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_px<F: Float>(&self) -> F {
        match &self {
            Self::Px => F::one(),
            Self::In => F::from(96.0).unwrap(),
            Self::Ft => F::from(12.0 * 96.0).unwrap(),
            Self::Yd => F::from(36.0 * 96.0).unwrap(),
            Self::Mi => F::from(1760.0 * 36.0 * 96.0).unwrap(),
            Self::Mm => F::from(96.0 / 25.4).unwrap(),
            Self::Cm => F::from(96.0 / 2.54).unwrap(),
            Self::M => F::from(100.0 * 96.0 / 2.54).unwrap(),
            Self::Km => F::from(100_000.0 * 96.0 / 2.54).unwrap(),
            Self::Pc => F::from(16.0).unwrap(),
            Self::Pt => F::from(96.0 / 72.0).unwrap(),
        }
    }

    /// Convert a value from pixel unit.
    #[must_use]
    #[inline]
    pub fn convert<F: Float>(&self, pixel_value: F) -> F {
        self.convert_from(&Unit::Px, pixel_value)
    }

    /// Convert a value from `from_unit` to this [`Unit`].
    #[must_use]
    #[inline]
    pub fn convert_from<F: Float>(&self, from_unit: &Unit, value: F) -> F {
        value * from_unit.to_px() / self.to_px()
    }

    /// Convert a value from this [`Unit`] to `to_unit`.
    #[must_use]
    #[inline]
    pub fn convert_to<F: Float>(&self, to_unit: &Unit, value: F) -> F {
        value * self.to_px() / to_unit.to_px()
    }

    /// Convert the unit to its string representation.
    ///
    /// Note: The opposite operation is available via the [`TryFrom`] trait.
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
}

impl TryFrom<&str> for Unit {
    type Error = UnitError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "px" | "pixel" => Ok(Unit::Px),
            "in" | "inch" => Ok(Unit::In),
            "ft" | "feet" => Ok(Unit::Ft),
            "yd" | "yard" => Ok(Unit::Yd),
            "mi" | "mile" | "miles" => Ok(Unit::Mi),
            "mm" | "millimeter" | "millimetre" => Ok(Unit::Mm),
            "cm" | "centimeter" | "centimetre" => Ok(Unit::Cm),
            "m" | "meter" | "metre" => Ok(Unit::M),
            "km" | "kilometer" | "kilometre" => Ok(Unit::Km),
            "pc" | "pica" => Ok(Unit::Pc),
            "pt" | "point" | "points" => Ok(Unit::Pt),
            _ => Err(UnitError::UnrecognisedError(value.to_owned())),
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_str().fmt(f)
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Mul<F> for Unit {
    type Output = Length;

    fn mul(self, rhs: F) -> Self::Output {
        Self::Output::new(rhs, self)
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Mul<F> for &'_ Unit {
    type Output = Length;

    fn mul(self, rhs: F) -> Self::Output {
        Self::Output::new(rhs, *self)
    }
}

// Orphan rule requires us to unroll these.
macro_rules! unit_trait_impl {
    ($t:ty) => {
        impl From<Unit> for $t {
            fn from(value: Unit) -> $t {
                value.to_px()
            }
        }

        impl From<&'_ Unit> for $t {
            fn from(value: &'_ Unit) -> $t {
                value.to_px()
            }
        }

        impl std::ops::Mul<Unit> for $t {
            type Output = Length;

            fn mul(self, rhs: Unit) -> Self::Output {
                Self::Output::new(self, rhs)
            }
        }

        impl std::ops::Mul<&'_ Unit> for $t {
            type Output = Length;

            fn mul(self, rhs: &'_ Unit) -> Self::Output {
                Self::Output::new(self, *rhs)
            }
        }
    };
}

unit_trait_impl!(f32);
unit_trait_impl!(f64);
