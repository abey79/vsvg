use std::str::FromStr;

use num_traits::{AsPrimitive, Float};
use rand::Rng;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};

/// An angle in radians, with facilities for parsing and formatting.
///
/// Important: the [`FromStr`] implementation assumes degrees if no unit is provided, as it is used
/// for user input:
/// ```rust
/// # use vsvg::Angle;
/// assert_eq!("1.0rad".parse::<Angle>(), Ok(Angle::from_rad(1.0)));
/// assert_eq!("45deg".parse::<Angle>(), Ok(Angle::from_deg(45.0)));
/// assert_eq!("45".parse::<Angle>(), Ok(Angle::from_deg(45.0)));
/// ```
#[derive(
    Debug, Clone, Default, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct Angle(pub f64);

impl Angle {
    /// Create an angle from radians.
    #[must_use]
    pub fn from_rad(rad: f64) -> Self {
        Self(rad)
    }

    /// Create an angle from degrees.
    #[must_use]
    pub fn from_deg(deg: f64) -> Self {
        Self(deg.to_radians())
    }

    /// Radian value of this angle.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn rad<F: Float>(&self) -> F {
        F::from(self.0).unwrap()
    }

    /// Degree value of this angle.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn deg<F: Float>(&self) -> F {
        F::from(self.0.to_degrees()).unwrap()
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AngleError {
    #[error("Could not parse number: {0}")]
    FloatParseError(#[from] std::num::ParseFloatError),
    #[error("Could not parse unit: {0}")]
    UnitParseError(String),
}

impl FromStr for Angle {
    type Err = AngleError;

    /// Parse from user input. Assumes degrees if no unit is provided.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let number_str = s.trim_end_matches(|c: char| c.is_alphabetic() || c == '°');
        let unit_str = &s[number_str.len()..];

        // remove whitespace between number and unit
        let number_str = number_str.trim();

        let angle = number_str.parse::<f64>()?;

        match unit_str {
            "rad" | "r" => Ok(Self::from_rad(angle)),
            "deg" | "d" | "°" | "" => Ok(Self::from_deg(angle)),
            _ => Err(AngleError::UnitParseError(unit_str.to_string())),
        }
    }
}

impl std::ops::Add<Angle> for Angle {
    type Output = Self;

    fn add(self, rhs: Angle) -> Self::Output {
        Self::from_rad(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Angle> for Angle {
    type Output = Self;

    fn sub(self, rhs: Angle) -> Self::Output {
        Self::from_rad(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_rad(-self.0)
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Mul<F> for Angle {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        Self::from_rad(self.0 * rhs.as_())
    }
}

impl<F: Float + AsPrimitive<f64>> std::ops::Div<F> for Angle {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        Self::from_rad(self.0 / rhs.as_())
    }
}

// Orphan rule requires us to unroll these.
macro_rules! angle_trait_impl {
    ($t:ty) => {
        impl From<Angle> for $t {
            fn from(angle: Angle) -> Self {
                angle.rad()
            }
        }

        impl From<&'_ Angle> for $t {
            fn from(angle: &'_ Angle) -> Self {
                angle.rad()
            }
        }

        impl std::ops::Mul<Angle> for $t {
            type Output = Angle;

            fn mul(self, rhs: Angle) -> Self::Output {
                rhs.mul(self)
            }
        }
    };
}

angle_trait_impl!(f32);
angle_trait_impl!(f64);

// ==========================================
// `rand` support

pub struct AngleSampler(rand::distributions::uniform::UniformFloat<f64>);

impl UniformSampler for crate::AngleSampler {
    type X = Angle;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(rand::distributions::uniform::UniformFloat::new(
            low.borrow().0,
            high.borrow().0,
        ))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(rand::distributions::uniform::UniformFloat::new_inclusive(
            low.borrow().0,
            high.borrow().0,
        ))
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Self::X::from_rad(self.0.sample(rng))
    }
}

impl SampleUniform for Angle {
    type Sampler = crate::AngleSampler;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_from_str() {
        assert_eq!("0".parse::<Angle>().unwrap(), Angle(0.0));
        assert_eq!("0.0".parse::<Angle>().unwrap(), Angle(0.0));
        assert_eq!("1.0rad".parse::<Angle>().unwrap(), Angle(1.0));
        assert_eq!("1.0r".parse::<Angle>().unwrap(), Angle(1.0));

        assert_eq!("1.0deg".parse::<Angle>().unwrap(), Angle::from_deg(1.0));
        assert_eq!("1.0d".parse::<Angle>().unwrap(), Angle::from_deg(1.0));
        assert_eq!("1.0°".parse::<Angle>().unwrap(), Angle::from_deg(1.0));

        // !!!! FromStr assume user input, so defaults to degrees
        assert_eq!("1.0".parse::<Angle>().unwrap(), Angle::from_deg(1.0));

        assert_eq!(
            "1.0hello".parse::<Angle>(),
            Err(AngleError::UnitParseError("hello".to_string()))
        );
    }
}
