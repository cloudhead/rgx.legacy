//! Numerical traits.
use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait Zero: PartialEq + Sized {
    const ZERO: Self;

    fn is_zero(&self) -> bool;
}

impl Zero for f32 {
    const ZERO: f32 = 0.;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for f64 {
    const ZERO: f64 = 0.;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for usize {
    const ZERO: usize = 0;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for i32 {
    const ZERO: i32 = 0;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for u32 {
    const ZERO: u32 = 0;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for u64 {
    const ZERO: u64 = 0;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

impl Zero for i64 {
    const ZERO: i64 = 0;

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait One: Sized {
    const ONE: Self;

    fn is_one(&self) -> bool;
}

impl One for f32 {
    const ONE: f32 = 1.;

    fn is_one(&self) -> bool {
        self == &Self::ONE
    }
}

impl One for f64 {
    const ONE: f64 = 1.;

    fn is_one(&self) -> bool {
        self == &Self::ONE
    }
}

impl One for usize {
    const ONE: usize = 1;

    fn is_one(&self) -> bool {
        self == &Self::ONE
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Two {
    const TWO: Self;
}

impl Two for f32 {
    const TWO: f32 = 2.;
}

impl Two for f64 {
    const TWO: f64 = 2.;
}

impl Two for i32 {
    const TWO: i32 = 2;
}

////////////////////////////////////////////////////////////////////////////////

/// Floating point numbers.
pub trait Float:
    Copy
    + PartialOrd
    + Zero
    + One
    + Two
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + Mul<Output = Self>
{
    fn sqrt(self) -> Self;
}

impl Float for f32 {
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
}

impl Float for f64 {
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
}

/// Adds convenience methods to `f32` and `f64`.
pub trait FloatExt<T> {
    /// Rounds to the nearest integer away from zero,
    /// unless the provided value is already an integer.
    ///
    /// It is to `ceil` what `trunc` is to `floor`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rgx::math::traits::FloatExt;
    ///
    /// let f = 3.7_f64;
    /// let g = 3.0_f64;
    /// let h = -3.7_f64;
    /// let i = -5.1_f32;
    ///
    /// assert_eq!(f.expand(), 4.0);
    /// assert_eq!(g.expand(), 3.0);
    /// assert_eq!(h.expand(), -4.0);
    /// assert_eq!(i.expand(), -6.0);
    /// ```
    fn expand(&self) -> T;
}

impl FloatExt<f64> for f64 {
    #[inline]
    fn expand(&self) -> f64 {
        self.abs().ceil().copysign(*self)
    }
}

impl FloatExt<f32> for f32 {
    #[inline]
    fn expand(&self) -> f32 {
        self.abs().ceil().copysign(*self)
    }
}
