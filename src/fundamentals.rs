use std::fmt;
use std::ops::{Mul, Div};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Fundamentals {
    Mass,
    Length,
    Time,
    Current,
    Temperature,
    AmountOfSubstance,
    LuminousIntensity,
    Angle,
    Bit,
    Count,
}

impl fmt::Display for Fundamentals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Fundamentals::Length => "length",
                Fundamentals::Mass => "mass",
                Fundamentals::Time => "time",
                Fundamentals::Current => "current",
                Fundamentals::Temperature => "temperature",
                Fundamentals::AmountOfSubstance => "amount of substance",
                Fundamentals::LuminousIntensity => "luminous intensity",
                Fundamentals::Angle => "angle",
                Fundamentals::Bit => "bit",
                Fundamentals::Count => "count",
            }
        )
    }
}

impl Fundamentals {
    #[inline]
    pub const fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn from_index(n: usize) -> Result<Self, String> {
        match n {
            0 => Ok(Fundamentals::Mass),
            1 => Ok(Fundamentals::Length),
            2 => Ok(Fundamentals::Time),
            3 => Ok(Fundamentals::Current),
            4 => Ok(Fundamentals::Temperature),
            5 => Ok(Fundamentals::AmountOfSubstance),
            6 => Ok(Fundamentals::LuminousIntensity),
            7 => Ok(Fundamentals::Angle),
            8 => Ok(Fundamentals::Bit),
            9 => Ok(Fundamentals::Count),
            _ => Err("Invalid index of fundamental dimensionalities".to_string()),
        }
    }

    pub fn iter() -> impl Iterator<Item = Fundamentals> {
        use Fundamentals::*;
        [
            Mass,
            Length,
            Time,
            Current,
            Temperature,
            AmountOfSubstance,
            LuminousIntensity,
            Angle,
            Bit,
            Count,
        ]
        .iter()
        .copied()
    }
}

pub const FUNDAMENTALS_NUMBER: usize = 10;
pub type FundamentalsPowersType = i32;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Dimension([FundamentalsPowersType; FUNDAMENTALS_NUMBER]);

impl Dimension {
    pub fn new(powers: [FundamentalsPowersType; FUNDAMENTALS_NUMBER]) -> Dimension {
        let mut result = [0; FUNDAMENTALS_NUMBER];
        if powers[..FUNDAMENTALS_NUMBER - 1] == [0; FUNDAMENTALS_NUMBER - 1] {
            result[FUNDAMENTALS_NUMBER - 1..].copy_from_slice(&[1]);
            Dimension(result)
        } else {
            result[..FUNDAMENTALS_NUMBER - 1].copy_from_slice(&powers[..FUNDAMENTALS_NUMBER - 1]);
            result[FUNDAMENTALS_NUMBER - 1..].copy_from_slice(&[0]);
            Dimension(result)
        }
    }

    const fn new_from_fundamental(fundamental: Fundamentals) -> Dimension {
        let mut powers = [0; 10];
        powers[fundamental.to_index()] = 1;

        Dimension(powers)
    }

    pub fn mul(self, rhs: Dimension) -> Dimension {
        let mut powers = self.0;
        powers.iter_mut().zip(rhs.0.iter()).for_each(|(x, y)| {
            *x += y;
        });
        Dimension::new(powers)
    }

    pub fn div(self, rhs: Dimension) -> Dimension {
        let mut powers = self.0;
        powers.iter_mut().zip(rhs.0.iter()).for_each(|(x, y)| {
            *x -= y;
        });
        Dimension::new(powers)
    }

    pub fn pow(self, power: i64) -> Dimension {
        let mut powers = self.0;
        powers.iter_mut().for_each(|x| {
            *x *= power as FundamentalsPowersType;
        });
        Dimension::new(powers)
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut displayed = vec![];
        self.0.iter().enumerate().for_each(|(i, power)| {
            if *power == 1 {
                displayed.push(format!("{}", Fundamentals::from_index(i).unwrap()));
            } else if *power != 0 {
                displayed.push(format!(
                    "[{}]^{}",
                    Fundamentals::from_index(i).unwrap(),
                    power
                ));
            }
        });

        write!(f, "{}", displayed.join(" * "))
    }
}

impl Mul<Dimension> for Dimension {
    type Output = Dimension;

    fn mul(self, rhs: Dimension) -> Self::Output {
        self.mul(rhs)
    }
}

impl Div<Dimension> for Dimension {
    type Output = Dimension;

    fn div(self, rhs: Dimension) -> Self::Output {
        self.div(rhs)
    }
}

pub mod base {
    use super::*;
    use Fundamentals::*;

    pub const MASS: Dimension = Dimension::new_from_fundamental(Mass);
    pub const LENGTH: Dimension = Dimension::new_from_fundamental(Length);
    pub const TIME: Dimension = Dimension::new_from_fundamental(Time);
    pub const CURRENT: Dimension = Dimension::new_from_fundamental(Current);
    pub const TEMPERATURE: Dimension = Dimension::new_from_fundamental(Temperature);
    pub const AMOUNT_OF_SUBSTANCE: Dimension = Dimension::new_from_fundamental(AmountOfSubstance);
    pub const LUMINOUS_INTENSITY: Dimension = Dimension::new_from_fundamental(LuminousIntensity);
    pub const ANGLE: Dimension = Dimension::new_from_fundamental(Angle);
    pub const BIT: Dimension = Dimension::new_from_fundamental(Bit);
    pub const COUNT: Dimension = Dimension::new_from_fundamental(Count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dimensionality() {
        let powers = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let dimensionality = Dimension::new(powers);
        assert_eq!(dimensionality.0, powers);

        let powers = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let dimensionality = Dimension::new_from_fundamental(Fundamentals::Length);
        assert_eq!(dimensionality.0, powers);
    }

    #[test]
    fn test_multiplication_of_dimensionalities() {
        use Fundamentals::*;

        let powers = [1, 1, -2, 0, 0, 0, 0, 0, 0, 0];
        let length = Dimension::new_from_fundamental(Length);
        let mass = Dimension::new_from_fundamental(Mass);
        let time = Dimension::new_from_fundamental(Time);
        let force = length * mass * time.pow(-2);
        println!("{}", force);

        assert_eq!(force.0, powers);
    }

    #[test]
    fn test_multiplication_of_dimensionalities_2() {
        use Fundamentals::*;

        let powers = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let length = Dimension::new_from_fundamental(Length);
        let mass = Dimension::new_from_fundamental(Mass);

        let collapsed = Dimension::new(powers) / (length * mass);
        println!("{}", collapsed);
        assert_eq!(collapsed, Dimension::new_from_fundamental(Count));

        let uncollapsed = collapsed * length;
        assert_eq!(uncollapsed, Dimension::new_from_fundamental(Length));

        assert_eq!(length * Dimension::new_from_fundamental(Count), length);
    }

    #[test]
    fn test_fundamentals_display() {
        assert_eq!(Fundamentals::Length.to_string(), "length");
        assert_eq!(Fundamentals::Mass.to_string(), "mass");
        assert_eq!(Fundamentals::Time.to_string(), "time");
        assert_eq!(Fundamentals::Current.to_string(), "current");
        assert_eq!(Fundamentals::Temperature.to_string(), "temperature");
        assert_eq!(
            Fundamentals::AmountOfSubstance.to_string(),
            "amount of substance"
        );
        assert_eq!(
            Fundamentals::LuminousIntensity.to_string(),
            "luminous intensity"
        );
        assert_eq!(Fundamentals::Angle.to_string(), "angle");
        assert_eq!(Fundamentals::Bit.to_string(), "bit");
        assert_eq!(Fundamentals::Count.to_string(), "count");
    }

    #[test]
    fn test_fundamentals_to_index() {
        assert_eq!(Fundamentals::Mass.to_index(), 0);
        assert_eq!(Fundamentals::Length.to_index(), 1);
        assert_eq!(Fundamentals::Time.to_index(), 2);
        assert_eq!(Fundamentals::Current.to_index(), 3);
        assert_eq!(Fundamentals::Temperature.to_index(), 4);
        assert_eq!(Fundamentals::AmountOfSubstance.to_index(), 5);
        assert_eq!(Fundamentals::LuminousIntensity.to_index(), 6);
        assert_eq!(Fundamentals::Angle.to_index(), 7);
        assert_eq!(Fundamentals::Bit.to_index(), 8);
        assert_eq!(Fundamentals::Count.to_index(), 9);
    }

    #[test]
    fn test_fundamentals_from_index() {
        assert_eq!(Fundamentals::from_index(0), Ok(Fundamentals::Mass));
        assert_eq!(Fundamentals::from_index(1), Ok(Fundamentals::Length));
        assert_eq!(Fundamentals::from_index(2), Ok(Fundamentals::Time));
        assert_eq!(Fundamentals::from_index(3), Ok(Fundamentals::Current));
        assert_eq!(Fundamentals::from_index(4), Ok(Fundamentals::Temperature));
        assert_eq!(
            Fundamentals::from_index(5),
            Ok(Fundamentals::AmountOfSubstance)
        );
        assert_eq!(
            Fundamentals::from_index(6),
            Ok(Fundamentals::LuminousIntensity)
        );
        assert_eq!(Fundamentals::from_index(7), Ok(Fundamentals::Angle));
        assert_eq!(Fundamentals::from_index(8), Ok(Fundamentals::Bit));
        assert_eq!(Fundamentals::from_index(9), Ok(Fundamentals::Count));
        assert!(Fundamentals::from_index(10).is_err());
    }

    #[test]
    fn test_fundamentals_iter() {
        let fundamentals: Vec<Fundamentals> = Fundamentals::iter().collect();
        assert_eq!(fundamentals.len(), FUNDAMENTALS_NUMBER);
        assert_eq!(fundamentals[0], Fundamentals::Mass);
        assert_eq!(fundamentals[1], Fundamentals::Length);
        assert_eq!(fundamentals[9], Fundamentals::Count);
    }

    #[test]
    fn test_dimensionality_new() {
        let powers = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let dim = Dimension::new(powers);
        assert_eq!(dim.0, powers);

        let powers_with_count = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let dim = Dimension::new([0; FUNDAMENTALS_NUMBER]);
        assert_eq!(dim.0, powers_with_count);
    }

    #[test]
    fn test_dimensionality_new_from_fundamental() {
        let dim = Dimension::new_from_fundamental(Fundamentals::Mass);
        assert_eq!(dim.0, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let dim = Dimension::new_from_fundamental(Fundamentals::Length);
        assert_eq!(dim.0, [0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);

        let dim = Dimension::new_from_fundamental(Fundamentals::Count);
        assert_eq!(dim.0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_dimensionality_mul() {
        let length = Dimension::new_from_fundamental(Fundamentals::Length);
        let mass = Dimension::new_from_fundamental(Fundamentals::Mass);
        let result = length.mul(mass);
        assert_eq!(result.0, [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);

        let time = Dimension::new_from_fundamental(Fundamentals::Time);
        let result = length * time;
        assert_eq!(result.0, [0, 1, 1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_dimensionality_div() {
        let length = Dimension::new_from_fundamental(Fundamentals::Length);
        let time = Dimension::new_from_fundamental(Fundamentals::Time);
        let result = length.div(time);
        assert_eq!(result.0, [0, 1, -1, 0, 0, 0, 0, 0, 0, 0]);

        let mass = Dimension::new_from_fundamental(Fundamentals::Mass);
        let result = mass.div(mass);
        assert_eq!(result.0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_dimensionality_pow() {
        let length = Dimension::new_from_fundamental(Fundamentals::Length);
        let result = length.pow(2);
        assert_eq!(result.0, [0, 2, 0, 0, 0, 0, 0, 0, 0, 0]);

        let result = length.pow(-1);
        assert_eq!(result.0, [0, -1, 0, 0, 0, 0, 0, 0, 0, 0]);

        let result = length.pow(0);
        assert_eq!(result.0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

        let count = Dimension::new_from_fundamental(Fundamentals::Count);
        let result = count.pow(2);
        assert_eq!(result.0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_dimensionality_display() {
        let dim = Dimension::new([1, 2, -2, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(dim.to_string(), "mass * [length]^2 * [time]^-2");

        let dim = Dimension::new_from_fundamental(Fundamentals::Count);
        assert_eq!(dim.to_string(), "count");

        let dim = Dimension::new([0; FUNDAMENTALS_NUMBER]);
        assert_eq!(dim.to_string(), "count");
    }

    #[test]
    fn test_dimensionality_ops() {
        let length = Dimension::new_from_fundamental(Fundamentals::Length);
        let mass = Dimension::new_from_fundamental(Fundamentals::Mass);
        let time = Dimension::new_from_fundamental(Fundamentals::Time);

        let result = length * mass / time;
        assert_eq!(result.0, [1, 1, -1, 0, 0, 0, 0, 0, 0, 0]);

        let result = (length * mass) / (length * mass);
        assert_eq!(result.0, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }
}
