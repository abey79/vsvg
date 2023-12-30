use crate::Unit;
use num_traits::{AsPrimitive, Float};
use std::fmt::{Display, Formatter};

/// A physical length, described by a value and a [`Unit`].
///
/// A [`Length`] can be created with [`Length::new`] or by multiplying a float with a [`Unit`]:
/// ```
/// # use vsvg::{Unit, Length};
/// assert_eq!(Length::new(0.0356, Unit::Cm), 0.0356 * Unit::Cm);
/// ```
///
/// All float conversion assume the default [`Unit`] of [`Unit::Px`]:
/// ```
/// # use vsvg::{Unit, Length};
/// assert_eq!(Length::from(96.0), Length::new(96., Unit::Px));
/// assert_eq!(f64::from(1.0 * Unit::In), 96.0);
/// ```
///
/// The usual arithmetic operations are supported.
///
/// **Note**: Floats are always considered as [`Unit::Px`]. When adding or subtracting two
/// [`Length`]s, the result will have the [`Unit`] of the left-hand side.
///
/// ```
/// # use vsvg::{Unit, Length};
/// // Negation is supported.
/// assert_eq!(-Length::new(1.0, Unit::In), -1.0 * Unit::In);
///
/// // The result has the unit of the left-hand side.
/// assert_eq!(1.0 * Unit::In + 2.54 * Unit::Cm, 2.0 * Unit::In);
/// assert_eq!(5.08 * Unit::Cm - 1.0 * Unit::In, 2.54 * Unit::Cm);
///
/// // Floats are considered pixels.
/// assert_eq!(1.0 * Unit::In + 96.0, 2.0 * Unit::In);
/// assert_eq!(96.0 + 1.0 * Unit::In , 2.0 * Unit::In);
/// assert_eq!(2.0 * Unit::In - 96.0, 1.0 * Unit::In);
/// assert_eq!(96.0 - 0.5 * Unit::In, 0.5 * Unit::In);
///
/// // Multiplication and division by floats is supported.
/// // Note: dividing by a `Length` is not supported.
/// assert_eq!((1.0 * Unit::In) * 2.0, 2.0 * Unit::In);
/// assert_eq!(2.0 * (1.0 * Unit::In), 2.0 * Unit::In);
/// assert_eq!((1.0 * Unit::In) / 2.0, 0.5 * Unit::In);
/// ```
///
/// [`Length`] implements [`From`] for [`Unit`], so you can use [`Unit`] as a shorthand:
/// ```
/// # use vsvg::{Unit, Length};
/// assert_eq!(Length::from(Unit::In), Length::new(1., Unit::In));
/// ```
///
/// A [`Length`] with a different [`Unit`] can be converted using [`Length::convert_to`]:
/// ```
/// # use vsvg::{Unit, Length};
/// let l = Length::new(2.54, Unit::Cm);
/// assert_eq!(l.convert_to(Unit::In), 1.0 * Unit::In);
/// ```
///
/// [`Length`] delegates [`Display`] to [`f64`], so it supports the standard float formatting
/// syntax:
/// ```
/// # use vsvg::Length;
/// let l = Length::new(0.0356, vsvg::Unit::Cm);
/// assert_eq!(format!("{l:.2}"), "0.04cm");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Length {
    pub value: f64,
    pub unit: Unit,
}

impl Default for Length {
    fn default() -> Self {
        Length {
            value: 0.0,
            unit: Unit::Px,
        }
    }
}

impl Display for Length {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)?;
        write!(f, "{}", self.unit)
    }
}

impl Length {
    #[must_use]
    pub fn new<F: Float + AsPrimitive<f64>>(value: F, unit: Unit) -> Self {
        Self {
            value: value.as_(),
            unit,
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_px<F: Float>(&self) -> F {
        F::from(self.value).unwrap() * self.unit.to_px::<F>()
    }

    #[must_use]
    pub fn convert_to(self, unit: Unit) -> Self {
        Self {
            value: self.unit.convert_to(&unit, self.value),
            unit,
        }
    }
}

impl From<Unit> for Length {
    fn from(value: Unit) -> Self {
        Self::new(1.0f64, value)
    }
}

impl From<&'_ Unit> for Length {
    fn from(value: &'_ Unit) -> Self {
        Self::new(1.0f64, *value)
    }
}

impl<F: Float + AsPrimitive<f64>> From<F> for Length {
    fn from(value: F) -> Self {
        Self::new(value, Unit::Px)
    }
}

impl std::ops::Add<Length> for Length {
    type Output = Self;

    fn add(self, rhs: Length) -> Self::Output {
        Self {
            value: self.value + rhs.convert_to(self.unit).value,
            unit: self.unit,
        }
    }
}

impl std::ops::Sub<Length> for Length {
    type Output = Self;

    fn sub(self, rhs: Length) -> Self::Output {
        Self {
            value: self.value - rhs.convert_to(self.unit).value,
            unit: self.unit,
        }
    }
}

impl std::ops::Neg for Length {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Add<F> for Length {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        Self {
            value: self.value + Unit::Px.convert_to(&self.unit, rhs.as_()),
            unit: self.unit,
        }
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Sub<F> for Length {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        Self {
            value: self.value - Unit::Px.convert_to(&self.unit, rhs.as_()),
            unit: self.unit,
        }
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Mul<F> for Length {
    type Output = Length;

    fn mul(self, rhs: F) -> Self::Output {
        Self {
            value: self.value * rhs.as_(),
            unit: self.unit,
        }
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Div<F> for Length {
    type Output = Length;

    fn div(self, rhs: F) -> Self::Output {
        Self {
            value: self.value / rhs.as_(),
            unit: self.unit,
        }
    }
}

// Orphan rule requires us to unroll these.
macro_rules! length_trait_impl {
    ($t:ty) => {
        impl From<Length> for $t {
            fn from(length: Length) -> Self {
                length.to_px()
            }
        }

        impl From<&'_ Length> for $t {
            fn from(length: &'_ Length) -> Self {
                length.to_px()
            }
        }

        impl std::ops::Add<Length> for $t {
            type Output = Length;

            fn add(self, rhs: Length) -> Self::Output {
                rhs.add(self)
            }
        }

        impl std::ops::Sub<Length> for $t {
            type Output = Length;

            fn sub(self, rhs: Length) -> Self::Output {
                -rhs.sub(self)
            }
        }

        impl std::ops::Mul<Length> for $t {
            type Output = Length;

            fn mul(self, rhs: Length) -> Self::Output {
                rhs.mul(self)
            }
        }
    };
}

length_trait_impl!(f32);
length_trait_impl!(f64);
