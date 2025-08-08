use std::any::Any;
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

pub trait MathOpsF64:
    Add<f64, Output = Self>
    + Sub<f64, Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + Copy
    + Clone
    + PartialEq
{
    fn log(&self, base: f64) -> Self;
    fn exp(&self, base: f64) -> Self;
    fn pow(&self, power: f64) -> Self;
    fn as_f64(&self) -> f64;
}

impl MathOpsF64 for f64 {
    fn log(&self, base: f64) -> Self {
        f64::log(*self, base)
    }

    fn exp(&self, base: f64) -> Self {
        base.powf(*self)
    }

    fn pow(&self, power: f64) -> Self {
        self.powf(power)
    }

    fn as_f64(&self) -> f64 {
        *self
    }
}

pub trait UnitTransformation<T: MathOpsF64>: Clone + Debug {
    fn to_base(&self, value: T) -> T;
    fn from_base(&self, value: T) -> T;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct LinearTransformation {
    pub scale: f64,  // scale factor
    pub offset: f64, // shift factor (bias)
}

impl LinearTransformation {
    pub fn new(scale: f64, offset: f64) -> Self {
        Self { scale, offset }
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }
}

impl<T: MathOpsF64> UnitTransformation<T> for LinearTransformation {
    fn to_base(&self, value: T) -> T {
        (value * self.scale + self.offset) as T
    }

    fn from_base(&self, value: T) -> T {
        ((value - self.offset) / self.scale) as T
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct IdentityTransformation;

impl IdentityTransformation {
    pub fn new() -> Self {
        Self {}
    }
}

impl<T: MathOpsF64> UnitTransformation<T> for IdentityTransformation {
    fn from_base(&self, value: T) -> T {
        value
    }

    fn to_base(&self, value: T) -> T {
        value
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct DecibelTransformation {
    pub p0: f64, // base value of the relation
}

impl DecibelTransformation {
    pub fn new(p0: f64) -> Self {
        Self { p0 }
    }

    pub fn p0(&self) -> f64 {
        self.p0
    }
}

impl<T: MathOpsF64> UnitTransformation<T> for DecibelTransformation {
    fn to_base(&self, value: T) -> T {
        (value / 10.0f64).exp(10.0f64) * self.p0
    }

    fn from_base(&self, value: T) -> T {
        (value / self.p0).log(10.0) * 10.0f64
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod transformations_tests {
    use super::*;

    #[test]
    fn test_identity_transformation() {
        let trans = IdentityTransformation::new();
        assert_eq!(trans.to_base(42.0), 42.0);
        assert_eq!(trans.from_base(42.0), 42.0);

        // Verify type through downcasting
        let any_trans = <IdentityTransformation as UnitTransformation<f64>>::as_any(&trans);
        assert!(any_trans.is::<IdentityTransformation>());
    }

    #[test]
    fn test_linear_transformation() {
        let trans = LinearTransformation::new(2.0, 5.0);
        assert_eq!(trans.scale(), 2.0);
        assert_eq!(trans.offset(), 5.0);

        // Test to_base: value * scale + offset
        assert_eq!(trans.to_base(9.0), 23.0); // 9 * 2 + 5 = 23
        assert_eq!(trans.to_base(5.0), 15.0); // 5 * 2 + 5 = 15

        // Test from_base: (value - offset) / scale
        assert_eq!(trans.from_base(3.0), -1.0); // (3 - 5) / 2 = -1
        assert_eq!(trans.from_base(9.0), 2.0); // (9 - 5) / 2 = 2

        // Verify type through downcasting
        let any_trans = <LinearTransformation as UnitTransformation<f64>>::as_any(&trans);
        assert!(any_trans.is::<LinearTransformation>());
    }

    #[test]
    fn test_decibel_transformation() {
        let trans = DecibelTransformation::new(1.0);
        assert_eq!(trans.p0(), 1.0);

        // Test to_base: 10 ^ (value / 10.0) * p0
        assert_eq!(trans.to_base(0.0), 1.0); // 10^(0/10) * 1 = 1
        assert_eq!(trans.to_base(10.0), 10.0); // (10/10)^10 * 1 = 10

        // Test from_base: 10.0 * log10(value / p0)
        assert_eq!(trans.from_base(1.0), 0.0); // log10(1/1) * 10 = 0
        assert_eq!(trans.from_base(10.0), 10.0); // log10(10/1) * 10 = 10

        // Verify type through downcasting
        let any_trans = <DecibelTransformation as UnitTransformation<f64>>::as_any(&trans);
        assert!(any_trans.is::<DecibelTransformation>());
    }

    #[test]
    fn test_math_ops_f64() {
        let value: f64 = 100.0;
        assert_eq!(value.log(10.0), 2.0); // log10(100) = 2
        assert_eq!(value.powf(2.0), 10000.0); // 100^2 = 10000

        // Test arithmetic operations
        assert_eq!(value + 50.0, 150.0);
        assert_eq!(value - 50.0, 50.0);
        assert_eq!(value * 2.0, 200.0);
        assert_eq!(value / 2.0, 50.0);
    }
}
