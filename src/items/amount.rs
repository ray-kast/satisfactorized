use std::{
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use num_derive::{One, Zero};
use ordered_float::NotNan;
use serde::de;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Invalid item amount {0:?}")]
pub struct AmountError(f64);

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{0}")]
    Amount(#[from] AmountError),
    #[error("{0}")]
    Std(#[from] std::num::ParseFloatError),
}

// TODO: subject to change.  i'd like to not use IEEE754 values
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Zero, One)]
pub struct Amount(NotNan<f64>);

impl Amount {
    pub fn new(f: f64) -> Result<Self, AmountError> {
        Self::new_not_nan(NotNan::new(f).map_err(|_| AmountError(f))?)
    }

    pub fn new_not_nan(f: NotNan<f64>) -> Result<Self, AmountError> {
        if f == -0.0 {
            Ok(Self(-f))
        } else if f.is_sign_negative() {
            Err(AmountError(*f))
        } else {
            Ok(Self(f))
        }
    }

    pub fn checked_sub(self, rhs: Amount) -> Option<Self> { Self::new_not_nan(self.0 - rhs.0).ok() }

    #[inline(always)]
    fn debug_assert(&self) {
        debug_assert!(
            !self.0.is_sign_negative(),
            "Amount produced with invalid value!"
        );
    }
}

impl FromStr for Amount {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> { Self::new(s.parse()?).map_err(Into::into) }
}

impl TryFrom<f64> for Amount {
    type Error = AmountError;

    fn try_from(f: f64) -> Result<Self, AmountError> { Self::new(f) }
}

impl<'de> de::Deserialize<'de> for Amount {
    fn deserialize<D: de::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        f64::deserialize(de).and_then(|f| Self::new(f).map_err(<D::Error as de::Error>::custom))
    }
}

impl AddAssign<Amount> for Amount {
    fn add_assign(&mut self, rhs: Amount) {
        self.0 += rhs.0;
        self.debug_assert();
    }
}

impl SubAssign<Amount> for Amount {
    fn sub_assign(&mut self, rhs: Amount) { *self = self.checked_sub(rhs).unwrap(); }
}

impl MulAssign<Amount> for Amount {
    fn mul_assign(&mut self, rhs: Amount) {
        self.0 *= rhs.0;
        self.debug_assert();
    }
}

// TODO: macro all this below

impl Add<Amount> for Amount {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl Sub<Amount> for Amount {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl Mul<Amount> for Amount {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self *= rhs;
        self
    }
}
