# Arshin: A Rust Library for Units and Quantities

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Arshin is a lightweight, type-safe Rust library for managing physical units, dimensions, and quantities. Inspired by libraries like Pint (Python) and Boost.Units (C++), it provides dimensional analysis to prevent unit mismatches, supports conversions (including linear offsets like Celsius-Kelvin and decibel scales), and allows arithmetic operations on quantities. Units can be defined programmatically or parsed from a custom file format, with support for SI prefixes.

Key principles:

- **Dimensional Safety**: Operations check compatibility at runtime.
- **Base-Unit Storage**: Quantities store magnitudes in base units for efficient computations.
- **Extensibility**: Load units from files using a Pest-based parser.
- **Macros for Convenience**: `u!` for units and `q!` for quantities.

Arshin is suitable for scientific computing, engineering simulations, or any application requiring precise unit handling.

## Features

- **Fundamental Dimensions**: 10 base dimensions (mass, length, time, current, temperature, amount of substance, luminous intensity, angle, bit, count).
- **Units**: Base units (e.g., meter), scaled (e.g., kilometer), biased (e.g., Celsius), and decibel-based.
- **Transformations**: Identity, linear (scale + offset), decibel (logarithmic).
- **Quantities**: Values with units; support add/sub/mul/div, scalar ops, powering.
- **Registry**: HashMap-based storage; load from files with prefixes.
- **Parser**: Define units in a simple DSL (e.g., `unit meter {dimension: length, transformation: identity, prefixes: standard }`).
- **Errors**: Custom enum for parsing, registry, and conversion issues.
- **Macros**: `u!("meter")` to get units, `q!(5.0, "meter")` for quantities (using default or custom registry).

Limitations:

- No nonlinear transformations beyond decibel.
- Multiplication/division panics on biased or decibel units.
- No automatic unit simplification (e.g., m/s remains as is).

## Installation

Add Arshin to your `Cargo.toml`:

```toml
[dependencies]
arshin = "0.1.0"  # Replace with the actual version
```

If not published on crates.io, clone the repository and add as a path dependency:

```toml
[dependencies]
arshin = { path = "/path/to/arshin" }
```

Build requirements: Rust 1.60+ (uses `lazy_static`, `pest`, `thiserror`).

## Usage

### Quick Start

1. **Load Default Registry**: Arshin includes a default registry loaded from `units.txt` (base SI units, common derivations).

2. **Create Quantities and Convert**:

```rust
use arshin::{q, u};  // Macros for quantity and unit

fn main() -> Result<(), arshin::errors::ArshinError> {
    // Create quantity: 5 kilometers
    let distance = q!(5.0, "kilometer")?;

    // Convert to meters
    let meter = u!("meter")?;
    println!("5 km = {} m", distance.m_as(&meter)?);  // Output: 5 km = 5000 m

    Ok(())
}
```

3. **Arithmetic Operations**:

```rust
use arshin::{q, u};

fn main() -> Result<(), arshin::errors::ArshinError> {
    let mass = q!(2.0, "kilogram")?;
    let accel = q!(9.81, "meter / second^2")?;
    let force = mass * accel;

    let newton = u!("newton")?;
    println!("Force = {} N", force.m_as(&newton)?);  // Output: Force = 19.62 N

    Ok(())
}
```

4. **Temperature (Biased Units)**:

```rust
use arshin::q;

fn main() -> Result<(), arshin::errors::ArshinError> {
    let temp_c = q!(25.0, "degree_celsius")?;
    let kelvin = q!(298.15, "kelvin")?;
    assert_eq!(temp_c.m_as(&kelvin.unit())?, 298.15);

    // Note: Cannot multiply biased units (panics)
    Ok(())
}
```

5. **Derived Units**:

```rust
use arshin::{Unit, fundamentals::base::{LENGTH, TIME}};

let meter = Unit::new_base("meter", LENGTH);
let second = Unit::new_base("second", TIME);
let speed_unit = meter / second;  // Name: "(meter / second)", dim: length / time
```

### Advanced Usage

#### Custom Registry from File

Define units in a file (e.g., `custom_units.txt`):

```
unit meter { 
    dimension: length
    transformation: identity
    prefixes: standard
}
unit second { 
    dimension: time
    transformation: identity
    prefixes: standard
}
unit newton { 
    dimension: mass * length / time^2
    transformation: identity
    prefixes: standard
}
unit degree_celsius {
    dimension: temperature
    transformation: linear(scale: 1, offset: 273.15)
    prefixes: no
}
unit decibel {
    dimension: count
    transformation: decibel(p0: 1)
    prefixes: no
}
```

Load it:

```rust
use arshin::registry::UnitRegistry;

let registry = UnitRegistry::new_from_file("custom_units.txt")?;
let meter = registry.get("meter").unwrap();
```

#### Manual Unit Registration

```rust
use arshin::{UnitRegistry, Unit, fundamentals::base::LENGTH};

let mut registry = UnitRegistry::new();
let kilometer = Unit::new_linear("kilometer", LENGTH, 1000.0, 0.0);
registry.register(kilometer)?;
```

#### Powering Quantities

```rust
let area = q!(registry, 10.0, "meter")?.pow(2);
assert_eq!(area.unit().dimensionality().to_string(), "[length]^2");
```

### Error Handling

Operations like incompatible units return `ArshinError`:

- `UnitsConversionError`: Dimension mismatch.
- `RegistryDoesNotContainUnit`: Unit not found.
- Panics: Invalid ops (e.g., mul on biased units).

Wrap in `Result` and handle accordingly.

## API Overview

- **fundamentals::Dimension**: Combines base dimensions (e.g., `MASS * LENGTH / TIME.pow(2)` for force).
- **units::Unit**: Core unit struct with `new_base`, `new_linear`, `to_base`, `from_base`, `compatible`.
- **transformations::UnitTransformation**: Enum for identity/linear/decibel conversions.
- **registry::UnitRegistry**: Stores units; `new_from_file`, `register`, `get`.
- **quantities::Quantity<T>**: Generic over `MathOpsF64` (default f64); `new`, `magnitude_as`, `pow`.
- **parser::parse_units_file**: Parses DSL to registry.
- **errors::ArshinError**: Error variants.
- **Macros**: `u!(name)` or `u!(registry, name)`; `q!(value, name)` or `q!(registry, value, name)`.

Full docs: Check source or `cargo doc`.

## Contributing

Contributions welcome! Fork, create a branch, add tests, and submit a PR.

- Run tests: `cargo test`
- Build docs: `cargo doc --open`
- Format: `cargo fmt`

Report issues on GitHub.

## License

MIT License. See [LICENSE](LICENSE) for details.

---
