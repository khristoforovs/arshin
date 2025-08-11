use crate::errors::ArshinError as Error;
use crate::fundamentals::Dimension;
use crate::registry::DEFAULT_REGISTRY;
use crate::registry::UnitRegistry;
use crate::transformations::{LinearTransformation, MathOpsF64, UnitTransformation};
use crate::units::Unit;
use std::ops::{Add, Div, Mul, Sub};

/// Represents a physical quantity: magnitude in base units + unit.
///
/// Stores magnitude in base for easy ops; converts on demand.
#[derive(Debug, Clone)]
pub struct Quantity<T>
where
    T: MathOpsF64 + 'static,
{
    magnitude: T,
    unit: Unit,
}

impl<T> Quantity<T>
where
    T: MathOpsF64 + 'static,
{
    /// Creates a quantity from magnitude and unit (converts to base internally).
    pub fn new(magnitude: T, unit: Unit) -> Self {
        let base_magnitude = unit.to_base(magnitude);
        Self {
            magnitude: base_magnitude,
            unit: unit,
        }
    }

    /// Creates from registry by unit name.
    ///
    /// # Errors
    /// If unit not found.
    pub fn new_from_registry(
        registry: &UnitRegistry,
        magnitude: T,
        unit_name: &str,
    ) -> Result<Self, Error> {
        let unit = registry
            .get(unit_name)
            .ok_or(Error::RegistryDoesNotContainUnit {
                name: unit_name.into(),
            })?;

        let base_magnitude = unit.to_base(magnitude);
        Ok(Self {
            magnitude: base_magnitude,
            unit: unit.clone(),
        })
    }

    /// Gets magnitude in a target unit.
    ///
    /// # Errors
    /// If dimensions incompatible.
    pub fn magnitude_as(&self, unit: &Unit) -> Result<T, Error> {
        if self.dimensionality() != unit.dimensionality() {
            Err(Error::UnitsConversionError {
                expected: *self.dimensionality(),
                got: *unit.dimensionality(),
            })
        } else {
            Ok(unit.from_base(self.magnitude))
        }
    }

    /// Shorthand for `magnitude_as`.
    pub fn m_as(&self, unit: &Unit) -> Result<T, Error> {
        self.magnitude_as(unit)
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn dimensionality(&self) -> &Dimension {
        self.unit.dimensionality()
    }

    pub fn base_magnitude(&self) -> T {
        self.magnitude
    }

    /// Raises the quantity to a power (updates dimension and magnitude).
    pub fn pow(&self, power: i64) -> Self {
        match self.unit().transformation() {
            UnitTransformation::Decibel(_) => panic!("Cannot raise a decibel quantity to a power"),
            UnitTransformation::Linear(LinearTransformation { scale: _, offset }) => {
                if *offset != 0.0 {
                    panic!("Cannot raise a biased quantity to a power");
                }
            }
            _ => {}
        }

        Self::new(self.magnitude.pow(power as f64), self.unit().pow(power))
    }
}

// Multiplication by scalar a (f64)
impl<T> Mul<f64> for Quantity<T>
where
    T: MathOpsF64,
{
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Self {
            magnitude: self.magnitude * scalar,
            unit: self.unit,
        }
    }
}

// Division by a scalar (f64)
impl<T> Div<f64> for Quantity<T>
where
    T: MathOpsF64,
{
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        Self {
            magnitude: self.magnitude / scalar,
            unit: self.unit,
        }
    }
}

// Addition of two quantities
impl<T> Add<Quantity<T>> for Quantity<T>
where
    T: MathOpsF64 + Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Quantity<T>) -> Self::Output {
        if self.dimensionality() != other.dimensionality() {
            let error = Error::UnitsConversionError {
                expected: *self.dimensionality(),
                got: *other.dimensionality(),
            };
            panic!("{}", error);
        }
        Self {
            magnitude: self.magnitude + other.magnitude,
            unit: self.unit,
        }
    }
}

// Subtraction of two quantities
impl<T> Sub<Quantity<T>> for Quantity<T>
where
    T: MathOpsF64 + Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Quantity<T>) -> Self::Output {
        if self.dimensionality() != other.dimensionality() {
            let error = Error::UnitsConversionError {
                expected: *self.dimensionality(),
                got: *other.dimensionality(),
            };
            panic!("{}", error);
        }
        Self {
            magnitude: self.magnitude - other.magnitude,
            unit: self.unit,
        }
    }
}

// Multiplication of two quantities
impl<T> Mul<Quantity<T>> for Quantity<T>
where
    T: MathOpsF64 + Mul<Output = T> + 'static,
{
    type Output = Self;

    fn mul(self, other: Quantity<T>) -> Self::Output {
        let new_unit = self.unit.clone() * other.unit.clone();
        let new_magnitude = self.magnitude * other.magnitude;
        Self {
            magnitude: new_magnitude,
            unit: new_unit,
        }
    }
}

// Division of two quantities
impl<T> Div<Quantity<T>> for Quantity<T>
where
    T: MathOpsF64 + Div<Output = T> + 'static,
{
    type Output = Self;

    fn div(self, other: Quantity<T>) -> Self::Output {
        let new_unit = self.unit.clone() / other.unit.clone();
        let new_magnitude = self.magnitude / other.magnitude;
        Self {
            magnitude: new_magnitude,
            unit: new_unit,
        }
    }
}

/// Macro to create a quantity from value and unit name (using custom or default registry).
#[macro_export]
macro_rules! q {
    ($registry:ident, $value:expr, $unit_name:expr) => {
        Quantity::new_from_registry(&$registry, $value, $unit_name)
    };

    ($value:expr, $unit_name:expr) => {
        Quantity::new_from_registry(&DEFAULT_REGISTRY, $value, $unit_name)
    };
}

#[cfg(test)]
mod tests {
    use crate::fundamentals::base::*;
    use crate::registry::UnitRegistry;
    use crate::u;

    use super::*;

    #[test]
    fn test_create_quantity() {
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1.0e3, 0.0);
        let quantity = Quantity::new(1.0, kilometer.clone());
        assert_eq!(quantity.dimensionality(), &LENGTH);
        assert_eq!(quantity.unit().name(), "kilometer");
        assert_eq!(quantity.base_magnitude(), 1000.0);
    }

    #[test]
    fn test_magnitude_as() {
        let meter = Unit::new_base("meter", LENGTH);
        let quantity = Quantity::new(1000.0, meter.clone());
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1.0e3, 0.0);
        assert_eq!(quantity.magnitude_as(&kilometer), Ok(1.0));

        let quantity = Quantity::new(1.0, kilometer.clone());
        assert_eq!(quantity.m_as(&meter), Ok(1.0e3));
    }

    #[test]
    fn test_registry_magnitude_in() -> Result<(), Error> {
        let r = UnitRegistry::new_from_file("src/units.txt")?;

        let meter = r.get("meter").unwrap();
        let kilometer = r.get("kilometer").unwrap();
        let foot = r.get("foot").unwrap();

        let meters = Quantity::new_from_registry(&r, 5000.0, "meter")?;
        let kilometers = Quantity::new_from_registry(&r, 5.0, "kilometer")?;

        assert_eq!(kilometers.magnitude_as(meter).unwrap(), 5000.0);
        assert_eq!(meters.magnitude_as(kilometer).unwrap(), 5.0);

        assert_eq!(kilometers.magnitude_as(foot).unwrap().round(), 16404.0);
        assert_eq!(meters.magnitude_as(foot).unwrap().round(), 16404.0);

        let gram = r.get("gram").unwrap();
        assert_eq!(
            Quantity::new_from_registry(&r, 2.0, "tonne")?.m_as(gram),
            Ok(2.0e6)
        );

        assert_eq!(
            q!(2.0, "tonne").and_then(|q| q.m_as(&u!("gram")?)),
            Ok(2.0e6)
        );

        let quantity = q!(4.0, "kilogram")? / q!(2.0, "meter")?;
        println!("{}", quantity.magnitude);
        println!("{}", quantity.unit.name());
        let new_unit = u!("gram")?.div(u!("millimeter")?);

        println!("{}", quantity.m_as(&new_unit)?);

        Ok(())
    }

    #[test]
    fn scalar_operations() {
        let meter = Unit::new_base("meter", LENGTH);
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1.0e3, 0.0);
        let quantity = Quantity::new(1000.0, meter.clone());
        assert_eq!(
            (quantity.clone() * 2.0).magnitude_as(&kilometer).unwrap(),
            2.0
        );
        assert_eq!((quantity / 2.0).magnitude_as(&kilometer).unwrap(), 0.5);
    }

    #[test]
    fn test_arithmetic_operations() -> Result<(), Error> {
        let meter = Unit::new_base("meter", LENGTH);
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1.0e3, 0.0);
        let quantity1 = Quantity::new(1000.0, meter.clone());
        let quantity2 = Quantity::new(2.0, kilometer.clone());
        assert_eq!(
            (quantity1.clone() + quantity2.clone()).magnitude_as(&meter)?,
            3000.0
        );
        assert_eq!((quantity1 - quantity2).magnitude_as(&kilometer)?, -1.0);

        let centimeter = Unit::new_linear("centimeter", LENGTH, 1.0e-2, 0.0);
        let cube_meter = meter.pow(3);
        let cube_centimeter = centimeter.pow(3);
        assert!(
            ((Quantity::new(1.0, cube_meter) / Quantity::new(1.0, cube_centimeter)).magnitude
                - 1.0e6)
                .abs()
                < 1.0e-5
        );

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_bad_arithmetic_operations() {
        let meter = Unit::new_base("meter", LENGTH);
        let second = Unit::new_base("second", TIME);
        let _ = Quantity::new(1.0, meter) + Quantity::new(1.0, second);
    }

    #[test]
    fn test_multiplication_of_quantities() -> Result<(), Error> {
        let gram = Unit::new_linear("gram", MASS, 1e-3, 0.0);
        let meter = Unit::new_base("meter", LENGTH);
        let second = Unit::new_base("second", TIME);

        let joule = Unit::new_base("joule", MASS * LENGTH.pow(2) / TIME.pow(2));

        assert_eq!(
            (Quantity::new(1.0e3, gram) * Quantity::new(4.0, meter).pow(2)
                / Quantity::new(1.0, second).pow(2))
            .magnitude_as(&joule)?,
            16.0
        );

        assert_eq!(
            (q!(1.0e3, "gram")? * q!(4.0, "meter")?.pow(2) / q!(1.0, "second")?.pow(2))
                .magnitude_as(&joule)?,
            16.0
        );

        Ok(())
    }
}
