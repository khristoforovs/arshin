use crate::errors::ArshinError as Error;
use crate::parser::parse_units_file;
use crate::units::Unit;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;

lazy_static! {
    // Default non-mutable registry
    pub static ref DEFAULT_REGISTRY: UnitRegistry = UnitRegistry::default();
}

/// Registry for storing and retrieving units by name.
///
/// Can be populated manually or from a file via parser.
pub struct UnitRegistry {
    pub units: HashMap<String, Unit>,
}

impl Default for UnitRegistry {
    fn default() -> Self {
        Self::new_from_file("src/units.txt").unwrap()
    }
}

impl UnitRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
        }
    }

    /// Creates a registry from a units file.
    ///
    /// # Parameters
    /// - `file_name`: Path to the file.
    ///
    /// # Returns
    /// `Ok(UnitRegistry)` or `ArshinError`.
    ///
    /// # Errors
    /// - File read errors.
    /// - Parse errors.
    pub fn new_from_file(file_name: &str) -> Result<UnitRegistry, Error> {
        let file_content = fs::read_to_string(file_name).map_err(|e| Error::OSError {
            message: e.to_string(),
        })?;

        let registry = parse_units_file(&file_content).map_err(|e| Error::PestParseError {
            message: e.to_string(),
        })?;

        Ok(registry)
    }

    pub fn unit_names(&self) -> impl Iterator<Item = String> {
        self.units.keys().map(|s| s.clone())
    }

    /// Registers a unit.
    ///
    /// # Errors
    /// If the name already exists.
    pub fn register(&mut self, unit: Unit) -> Result<(), Error> {
        let name: String = unit.name.to_string();
        if self.contains(unit.name()) {
            return Err(Error::RegistryAlreadyContainsUnit { name: name });
        }

        self.units.insert(name, unit);
        Ok(())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.units.contains_key(name)
    }

    /// Gets a unit by name.
    pub fn get(&self, name: &str) -> Option<&Unit> {
        self.units.get(name)
    }
}

/// Macro to get a unit from a registry (or default).
///
/// # Examples
/// `u!("meter")` or `u!(registry, "meter")`.
#[macro_export]
macro_rules! u {
    ($registry:ident, $unit_name:expr) => {
        $registry.get($unit_name).map(|unit| unit.clone()).ok_or(
            Error::RegistryDoesNotContainUnit {
                name: $unit_name.into(),
            },
        )
    };

    ($unit_name:expr) => {
        DEFAULT_REGISTRY
            .get($unit_name)
            .map(|unit| unit.clone())
            .ok_or(Error::RegistryDoesNotContainUnit {
                name: $unit_name.into(),
            })
    };
}

#[cfg(test)]
mod tests {
    use crate::fundamentals::base::{LENGTH, TEMPERATURE};

    use super::*;

    #[test]
    fn test_registry_create() {
        let registry = UnitRegistry::new();
        assert!(registry.units.is_empty());
    }

    #[test]
    fn test_registry_create_from_file() {
        let registry = UnitRegistry::new_from_file("src/units.txt").unwrap();
        assert!(registry.get("meter").is_some());
        assert!(registry.get("kilometer").is_some());
        assert!(registry.get("degree_celsius").is_some());
        assert!(registry.get("decibel").is_some());
        assert!(registry.get("newton").is_some());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = UnitRegistry::new();

        let meter = Unit::new_base("meter", LENGTH);
        let kilometer = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
        let celsius = Unit::new_linear("celsius", TEMPERATURE, 1.0, 273.15);

        registry.register(meter).unwrap();
        registry.register(kilometer).unwrap();
        registry.register(celsius).unwrap();

        // Getting by name
        if let Some(unit) = registry.get("kilometer") {
            println!("Found: {}", unit.name());
            let base = unit.to_base(5.0); // 5 km → 5000 m
            println!("5 km = {} in base units", base);
        }
    }

    #[test]
    fn test_failed_double_register() {
        let mut registry = UnitRegistry::new();
        let meter = Unit::new_base("meter", LENGTH);
        assert!(registry.register(meter.clone()).is_ok());
        assert!(registry.register(meter.clone()).is_err());
    }

    #[test]
    fn test_create_default() {
        let registry = UnitRegistry::new_from_file("src/units.txt").unwrap();
        assert!(registry.get("meter").is_some());
        assert!(registry.get("kilometer").is_some());
        assert!(registry.get("degree_celsius").is_some());
        assert!(registry.get("decibel").is_some());
        assert!(registry.get("newton").is_some());
    }
}
