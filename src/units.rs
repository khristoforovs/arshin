use crate::fundamentals::Dimension;
use crate::transformations::{LinearTransformation, MathOpsF64, UnitTransformation};
use std::fmt;
use std::ops::{Div, Mul};

#[derive(Debug, PartialEq, Clone)]
pub struct Unit {
    pub name: String,
    pub dimensionality: Dimension,
    pub transformation: UnitTransformation,
}

impl<'a> fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.name, self.dimensionality)
    }
}

impl Unit {
    pub fn new(
        name: impl Into<String>,
        dimension: Dimension,
        transformation: UnitTransformation,
    ) -> Self {
        Self {
            name: name.into(),
            dimensionality: dimension,
            transformation,
        }
    }

    pub fn new_base(name: impl Into<String>, dimension: Dimension) -> Self {
        Self::new(name.into(), dimension, UnitTransformation::Identity)
    }

    pub fn new_linear(
        name: impl Into<String>,
        dimension: Dimension,
        scale: f64,
        offset: f64,
    ) -> Self {
        Self::new(
            name.into(),
            dimension,
            UnitTransformation::Linear(LinearTransformation::new(scale, offset)),
        )
    }

    pub fn to_base<T: MathOpsF64>(&self, value: T) -> T {
        self.transformation.to_base(value)
    }

    pub fn from_base<T: MathOpsF64>(&self, value: T) -> T {
        self.transformation.from_base(value)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dimensionality(&self) -> &Dimension {
        &self.dimensionality
    }

    pub fn transformation(&self) -> &UnitTransformation {
        &self.transformation
    }

    pub fn compatible(&self, other: &Unit) -> bool {
        self.dimensionality == other.dimensionality
    }
}

impl Mul<Unit> for Unit {
    type Output = Unit;

    fn mul(self, rhs: Unit) -> Self::Output {
        use UnitTransformation::*;

        // Check for biased and non-linear transformations
        match (self.transformation, rhs.transformation) {
            (Linear(t1), _) if t1.offset != 0.0 => {
                panic!(
                    "Multiplication not permitted for unit '{}' with biased transformation",
                    self.name
                )
            }
            (_, Linear(t2)) if t2.offset != 0.0 => {
                panic!(
                    "Multiplication not permitted for unit '{}' with biased transformation",
                    rhs.name
                )
            }
            (Decibel(_), _) | (_, Decibel(_)) => {
                panic!("Multiplication not supported for decibel transformations")
            }
            _ => {}
        }

        // Combine names and dimensionalities
        let new_name = format!("({} * {})", self.name, rhs.name);
        let new_dimension = self.dimensionality * rhs.dimensionality;

        // Combine transformations
        let scale = match self.transformation {
            Identity => 1.0,
            Linear(LinearTransformation { scale, .. }) => scale,
            _ => unreachable!(),
        };
        let rhs_scale = match rhs.transformation {
            Identity => 1.0,
            Linear(LinearTransformation { scale, .. }) => scale,
            _ => unreachable!(),
        };

        Unit::new(
            Box::leak(new_name.into_boxed_str()),
            new_dimension,
            Linear(LinearTransformation {
                scale: scale * rhs_scale,
                offset: 0.0,
            }),
        )
    }
}

impl Div<Unit> for Unit {
    type Output = Unit;

    fn div(self, rhs: Unit) -> Self::Output {
        use UnitTransformation::*;

        // Check for biased and non-linear transformations
        match (self.transformation, rhs.transformation) {
            (Linear(t1), _) if t1.offset != 0.0 => {
                panic!(
                    "Multiplication not permitted for unit '{}' with biased transformation",
                    self.name
                )
            }
            (_, Linear(t2)) if t2.offset != 0.0 => {
                panic!(
                    "Multiplication not permitted for unit '{}' with biased transformation",
                    rhs.name
                )
            }
            (Decibel(_), _) | (_, Decibel(_)) => {
                panic!("Multiplication not supported for decibel transformations")
            }
            _ => {}
        }

        // Combine names and dimensionalities
        let new_name = format!("({} * {})", self.name, rhs.name);
        let new_dimension = self.dimensionality * rhs.dimensionality;

        // Combine transformations
        let scale = match self.transformation {
            Identity => 1.0,
            Linear(LinearTransformation { scale, .. }) => scale,
            _ => unreachable!(),
        };
        let rhs_scale = match rhs.transformation {
            Identity => 1.0,
            Linear(LinearTransformation { scale, .. }) => scale,
            _ => unreachable!(),
        };

        Unit::new(
            Box::leak(new_name.into_boxed_str()),
            new_dimension,
            Linear(LinearTransformation {
                scale: scale / rhs_scale,
                offset: 0.0,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::fundamentals::base::{LENGTH, MASS, TEMPERATURE, TIME};

    use super::*;

    #[test]
    fn test_unit_new() {
        let unit = Unit::new("meter", LENGTH, UnitTransformation::Identity);
        assert_eq!(unit.name(), "meter");
        assert_eq!(unit.dimensionality(), &LENGTH);
        assert_eq!(unit.to_base(10.0), 10.0);
        assert_eq!(unit.from_base(10.0), 10.0);
    }

    #[test]
    fn test_unit_new_base() {
        let unit = Unit::new_base("second", TIME);
        assert_eq!(unit.name(), "second");
        assert_eq!(unit.dimensionality(), &TIME);
        assert_eq!(unit.to_base(5.0), 5.0);
        assert_eq!(unit.from_base(5.0), 5.0);
    }

    #[test]
    fn test_unit_new_linear() {
        let unit = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
        assert_eq!(unit.name(), "kilometer");
        assert_eq!(unit.dimensionality(), &LENGTH);
        assert_eq!(unit.to_base(1.0), 1000.0); // 1 km = 1000 m
        assert_eq!(unit.from_base(1000.0), 1.0); // 1000 m = 1 km

        // Test with offset (e.g., Celsius to Kelvin)
        let unit = Unit::new_linear("celsius", TEMPERATURE, 1.0, 273.15);
        assert_eq!(unit.to_base(0.0), 273.15); // 0°C = 273.15 K
        assert_eq!(unit.from_base(273.15), 0.0); // 273.15 K = 0°C
    }

    #[test]
    fn test_unit_is_compatible_with() {
        let meter = Unit::new_base("meter", LENGTH);
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
        let second = Unit::new_base("second", TIME);

        assert!(meter.compatible(&kilometer));
        assert!(!meter.compatible(&second));
    }

    #[test]
    fn test_unit_transformation_access() {
        let unit = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
        let transformation = unit.transformation();
        if let UnitTransformation::Linear(transformation) = transformation {
            assert_eq!(transformation.scale(), 1000.0);
            assert_eq!(transformation.offset(), 0.0);
        }
    }

    #[test]
    fn test_unit_display() {
        let meter = Unit::new_base("meter", LENGTH);
        assert_eq!(format!("{}", meter), "meter [length]");

        let second = Unit::new_base("second", TIME);
        assert_eq!(format!("{}", second), "second [time]");

        let joule = Unit::new_base("joule", MASS * LENGTH.pow(2) / TIME.pow(2));
        assert_eq!(
            format!("{}", joule),
            "joule [mass * [length]^2 * [time]^-2]"
        );
    }

    #[test]
    fn test_units_operations() {
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
        let minute = Unit::new_linear("minute", TIME, 60.0, 0.0);
        let kilometer_per_minute = kilometer.clone() / minute.clone();
        assert_eq!(kilometer_per_minute.to_base(1.0), 1.0e3 / 60.0);

        let kilometer_minute = kilometer * minute;
        assert_eq!(kilometer_minute.to_base(1.0), 6.0e4);
    }
}
