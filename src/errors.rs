use crate::fundamentals::Dimension;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ArshinError {
    #[error(
        "This dimensionalities are not compatible to perform this operation: {} and {}",
        a,
        b
    )]
    NotCompatibleDimensionalities { a: Dimension, b: Dimension },

    #[error("Error during parsing: {}", message)]
    PestParseError { message: String },

    #[error("OS Error: {}", message)]
    OSError { message: String },

    #[error("Incompatible units: expected {}, got {}", expected, got)]
    UnitsConversionError { expected: Dimension, got: Dimension },

    #[error("Unit {} already exists", name)]
    RegistryAlreadyContainsUnit { name: String },

    #[error("Registry does not contain unit {}", name)]
    RegistryDoesNotContainUnit { name: String },
}
