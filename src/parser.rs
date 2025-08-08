use crate::errors::ArshinError as Error;
use crate::fundamentals::base::*;
use crate::registry::UnitRegistry;
use crate::transformations::{DecibelTransformation, UnitTransformation};
use crate::units::Unit;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "units.pest"]
struct UnitsParser;

#[derive(Debug)]
struct UnitDefinition {
    pub name: String,
    pub dimension: DimensionExpression,
    pub transformation: Transformation,
    pub prefixes: Prefixes,
}

#[derive(Debug)]
struct DimensionExpression {
    pub terms: Vec<DimensionTerm>,
}

#[derive(Debug)]
struct DimensionTerm {
    pub fundamental: String,
    pub exponent: i64,
}

#[derive(Debug)]
enum Transformation {
    Identity,
    Linear { scale: f64, offset: Option<f64> },
    Decibel { p0: f64 },
}

#[derive(Debug, PartialEq, Eq)]
enum Prefixes {
    Standard,
    No,
}

const SI_PREFIXES: [(&str, &str, f64); 24] = [
    ("Quetta", "Q", 1e30),
    ("Ronna", "R", 1e27),
    ("Yotta", "Y", 1e24),
    ("Zetta", "Z", 1e21),
    ("Exa", "E", 1e18),
    ("Peta", "P", 1e15),
    ("Tera", "T", 1e12),
    ("Giga", "G", 1e9),
    ("Mega", "M", 1e6),
    ("kilo", "k", 1e3),
    ("hecto", "h", 1e2),
    ("deca", "da", 1e1),
    ("deci", "d", 1e-1),
    ("centi", "c", 1e-2),
    ("milli", "m", 1e-3),
    ("micro", "µ", 1e-6),
    ("nano", "n", 1e-9),
    ("pico", "p", 1e-12),
    ("femto", "f", 1e-15),
    ("atto", "a", 1e-18),
    ("zepto", "z", 1e-21),
    ("yocto", "y", 1e-24),
    ("ronto", "r", 1e-27),
    ("quecto", "q", 1e-30),
];

pub fn parse_units_file(file_content: &str) -> Result<UnitRegistry, Error> {
    let mut registry = UnitRegistry::new();
    let pairs =
        UnitsParser::parse(Rule::units_list, file_content).map_err(|e| Error::PestParseError {
            message: e.to_string(),
        })?;

    let mut units = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::unit_definition => {
                units.push(parse_unit_definition(pair));
            }
            _ => continue,
        }
    }

    for one in units {
        let mut dimension = COUNT;
        for term in one.dimension.terms {
            let another_dimension = match term.fundamental.as_str() {
                "length" => LENGTH,
                "mass" => MASS,
                "time" => TIME,
                "current" => CURRENT,
                "temperature" => TEMPERATURE,
                "amount of substance" => AMOUNT_OF_SUBSTANCE,
                "luminous intensity" => LUMINOUS_INTENSITY,
                "angle" => ANGLE,
                "bit" => BIT,
                "count" => COUNT,
                _ => unreachable!(),
            };
            dimension = dimension.mul(another_dimension.pow(term.exponent));
        }

        match one.transformation {
            Transformation::Identity => {
                registry
                    .register(Unit::new_base(one.name.as_str(), dimension))
                    .unwrap();
            }
            Transformation::Linear { scale, offset } => {
                registry
                    .register(Unit::new_linear(
                        one.name.as_str(),
                        dimension,
                        scale,
                        offset.unwrap_or(0.0),
                    ))
                    .unwrap();
            }
            Transformation::Decibel { p0 } => {
                registry
                    .register(Unit::new(
                        one.name.as_str(),
                        dimension,
                        UnitTransformation::Decibel(DecibelTransformation::new(p0)),
                    ))
                    .unwrap();
            }
        }

        if one.prefixes == Prefixes::Standard {
            match one.transformation {
                Transformation::Decibel { .. } => {
                    return Err(Error::PestParseError {
                        message: "Decibel transformation is not compatible with standard prefixes"
                            .into(),
                    });
                }
                Transformation::Linear { scale, offset } => {
                    if offset.unwrap_or(0.0) != 0.0 {
                        return Err(Error::PestParseError {
                            message: "Linear transformation with offset is not compatible with standard prefixes".into()
                        });
                    }
                    for (prefix, _, factor) in SI_PREFIXES.iter() {
                        registry
                            .register(Unit::new_linear(
                                format!("{}{}", prefix, one.name).as_str(),
                                dimension,
                                scale * factor,
                                0.0,
                            ))
                            .unwrap();
                    }
                }
                Transformation::Identity => {
                    for (prefix, _, factor) in SI_PREFIXES.iter() {
                        registry
                            .register(Unit::new_linear(
                                format!("{}{}", prefix, one.name).as_str(),
                                dimension,
                                *factor,
                                0.0,
                            ))
                            .unwrap();
                    }
                }
            }
        }
    }

    Ok(registry)
}

fn parse_unit_definition(pair: Pair<Rule>) -> UnitDefinition {
    let mut name = String::new();
    let mut dimension = DimensionExpression { terms: Vec::new() };
    let mut transformation = Transformation::Identity;
    let mut prefixes = Prefixes::No;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => name = inner_pair.as_str().into(),
            Rule::dimension_property => dimension = parse_dimension_property(inner_pair),
            Rule::transformation_property => {
                transformation = parse_transformation_property(inner_pair)
            }
            Rule::prefixes_property => prefixes = parse_prefixes_property(inner_pair),
            _ => unreachable!(),
        }
    }

    UnitDefinition {
        name,
        dimension,
        transformation,
        prefixes,
    }
}

fn parse_dimension_property(pair: Pair<Rule>) -> DimensionExpression {
    let mut terms = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::dimension_expression {
            terms = parse_dimension_expression(inner_pair);
        }
    }

    DimensionExpression { terms }
}

fn parse_dimension_expression(pair: Pair<Rule>) -> Vec<DimensionTerm> {
    let mut terms = Vec::new();
    let mut current_operator = String::from("*");

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::dimension_term => {
                let mut term = parse_dimension_term(inner_pair);
                if current_operator == "/" {
                    term.exponent = -term.exponent;
                }
                terms.push(term);
            }
            Rule::operator => {
                current_operator = inner_pair.as_str().to_string();
            }
            _ => unreachable!(),
        }
    }

    terms
}

fn parse_dimension_term(pair: Pair<Rule>) -> DimensionTerm {
    let mut fundamental = String::new();
    let mut exponent = 1;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::fundamental => fundamental = inner_pair.as_str().to_string(),
            Rule::exponent => exponent = inner_pair.as_str().parse::<i64>().unwrap_or(1),
            _ => {}
        }
    }

    DimensionTerm {
        fundamental,
        exponent,
    }
}

fn parse_transformation_property(pair: Pair<Rule>) -> Transformation {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::transformation {
            return parse_transformation(inner_pair);
        }
    }

    Transformation::Identity
}

fn parse_transformation(pair: Pair<Rule>) -> Transformation {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identity_transformation => return Transformation::Identity,
            Rule::linear_transformation => {
                let mut scale = 0.0;
                let mut offset = None;
                for lp in inner_pair.into_inner() {
                    match lp.as_rule() {
                        Rule::number => {
                            if scale == 0.0 {
                                scale = lp.as_str().parse::<f64>().unwrap_or(1.0);
                            } else {
                                offset = Some(lp.as_str().parse::<f64>().unwrap_or(0.0));
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                return Transformation::Linear { scale, offset };
            }
            Rule::decibel_transformation => {
                let mut p0 = 0.0;
                for lp in inner_pair.into_inner() {
                    if lp.as_rule() == Rule::number {
                        p0 = lp.as_str().parse::<f64>().unwrap_or(1.0);
                    }
                }
                return Transformation::Decibel { p0 };
            }
            _ => unreachable!(),
        }
    }

    Transformation::Identity
}

fn parse_prefixes_property(pair: Pair<Rule>) -> Prefixes {
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::prefixes_expression {
            match inner_pair.as_str() {
                "standard" => return Prefixes::Standard,
                "no" => return Prefixes::No,
                _ => unreachable!(),
            }
        }
    }

    Prefixes::No
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn parse_registry() -> UnitRegistry {
            let file_content = r#"
            unit meter { 
                dimension: length
                transformation: identity
                prefixes: standard
            }
            unit gram { 
                dimension: mass
                transformation: identity
                prefixes: standard
            }
            unit newton { 
                dimension: mass * length / time^2
                transformation: linear(scale: 1.0, offset: 0.0)
                prefixes: standard
            }
            unit light_year {
                dimension: length
                transformation: linear(scale: 9.4607304725808e15)
                prefixes: no
            }
            unit degree_celsius {
                dimension: temperature
                transformation: linear(scale: 1, offset: 273.15)
                prefixes: no
            }
            unit degree_kelvin {
                dimension: temperature
                transformation: identity
                prefixes: standard
            }
            unit decibel {
                dimension: count
                transformation: decibel(p0: 1)
                prefixes: no
            }
            "#;

            parse_units_file(file_content).unwrap()
        }
    }

    #[test]
    fn test_content() {
        let registry = fixtures::parse_registry();

        let expected_names = vec!["meter", "kilogram", "decibel"];
        for name in &expected_names {
            assert!(registry.get(name).is_some(), "Unit {} not found", name);
        }
    }

    #[test]
    fn test_meter_dimensionality() {
        let registry = fixtures::parse_registry();

        let meter = registry.get("meter").expect("Meter unit not found");
        assert_eq!(meter.name(), "meter", "Meter unit has incorrect name");

        assert_eq!(
            meter.dimensionality(),
            &LENGTH,
            "Meter unit has incorrect dimensionality"
        );
    }

    #[test]
    fn test_kilogram_dimensionality() {
        let registry = fixtures::parse_registry();

        let kilogram = registry.get("kilogram").expect("Kilogram unit not found");
        assert_eq!(
            kilogram.name(),
            "kilogram",
            "Kilogram unit has incorrect name"
        );

        assert_eq!(
            kilogram.dimensionality(),
            &MASS,
            "Kilogram unit has incorrect dimensionality"
        );
    }

    #[test]
    fn test_linear_transformation() {
        use UnitTransformation::*;

        let registry = fixtures::parse_registry();

        let kilogram = registry.get("kilogram").unwrap();
        if let Linear(transformation) = kilogram.transformation() {
            assert_eq!(
                transformation.scale, 1000.0,
                "Kilogram scale coefficient incorrect"
            );
            assert_eq!(
                transformation.offset, 0.0,
                "Kilogram offset coefficient incorrect"
            );
        } else {
            unreachable!();
        }

        assert_eq!(
            kilogram.to_base(1.0),
            1.0e3,
            "Kilogram transformation incorrect"
        );
    }

    #[test]
    fn test_decibel_dimensionality() {
        let registry = fixtures::parse_registry();

        let decibel = registry.get("decibel").expect("Decibel unit not found");
        assert_eq!(decibel.name(), "decibel", "Decibel unit has incorrect name");

        assert_eq!(
            decibel.dimensionality(),
            &COUNT,
            "Decibel unit has incorrect dimensionality"
        );
    }

    #[test]
    fn test_decibel_transformation() {
        use UnitTransformation::*;

        let registry = fixtures::parse_registry();

        let decibel = registry.get("decibel").expect("Decibel unit not found");
        if let Decibel(transformation) = decibel.transformation() {
            assert_eq!(transformation.p0, 1.0, "Decibel p0 coefficient incorrect");
        } else {
            unreachable!();
        }

        assert_eq!(
            decibel.to_base(10.0),
            10.0,
            "Decibel transformation incorrect"
        );
    }
}
