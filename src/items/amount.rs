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

    #[inline]
    fn mutate<T>(&mut self, f: impl FnOnce(&mut NotNan<f64>) -> T) -> T {
        let ret = f(&mut self.0);

        debug_assert!(
            !self.0.is_sign_negative(),
            "Amount produced with invalid value!"
        );

        ret
    }

    fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
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
    fn add_assign(&mut self, rhs: Amount) { self.mutate(|f| *f += rhs.0) }
}

impl SubAssign<Amount> for Amount {
    fn sub_assign(&mut self, rhs: Amount) { *self = self.checked_sub(rhs).unwrap(); }
}

impl MulAssign<Amount> for Amount {
    fn mul_assign(&mut self, rhs: Amount) { self.mutate(|f| *f *= rhs.0) }
}

impl Add<Amount> for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self { self.with(|s| *s += rhs) }
}

impl Sub<Amount> for Amount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self { self.with(|s| *s -= rhs) }
}

impl Mul<Amount> for Amount {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self { self.with(|s| *s *= rhs) }
}

macro_rules! auto_impls {
    ($ty:ty, $op:ident, $fn:ident, $op_asn:ident, $fn_asn:ident) => {
        impl $op<&$ty> for $ty {
            type Output = $ty;

            #[inline]
            fn $fn(self, rhs: &$ty) -> $ty { self.$fn(*rhs) }
        }

        impl<'a> $op<$ty> for &'a $ty {
            type Output = $ty;

            #[inline]
            fn $fn(self, rhs: $ty) -> $ty { (*self).$fn(rhs) }
        }

        impl<'a> $op<&$ty> for &'a $ty {
            type Output = $ty;

            #[inline]
            fn $fn(self, rhs: &$ty) -> $ty { (*self).$fn(*rhs) }
        }

        impl $op_asn<&$ty> for $ty {
            #[inline]
            fn $fn_asn(&mut self, rhs: &$ty) { self.$fn_asn(*rhs) }
        }
    };
}

auto_impls!(Amount, Add, add, AddAssign, add_assign);
auto_impls!(Amount, Sub, sub, SubAssign, sub_assign);
auto_impls!(Amount, Mul, mul, MulAssign, mul_assign);
