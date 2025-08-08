use crate::errors::ArshinError as Error;
use crate::parser::parse_units_file;
use crate::units::Unit;
use std::collections::HashMap;
use std::fs;

pub struct UnitRegistry {
    pub units: HashMap<String, Unit>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
        }
    }

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

    pub fn get(&self, name: &str) -> Option<&Unit> {
        self.units.get(name)
    }
}

#[macro_export]
macro_rules! u {
    ($registry:ident, $unit_name:literal) => {
        $registry
            .get($unit_name)
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
