pub mod interval_coefficients;

use super::tools::{Merge, UID};
pub use interval_coefficients::IntervalCoefficients;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Deck {
    pub name: String,
    pub card_paths: Vec<String>,
    pub interval_coefficients: IntervalCoefficients,
}

impl Deck {
    pub fn new(
        name: &str,
        card_paths: Vec<&str>,
        interval_coefficients: IntervalCoefficients,
    ) -> Self {
        Self {
            name: name.to_string(),
            card_paths: card_paths.iter().map(|s| s.to_string()).collect(),
            interval_coefficients,
        }
    }

    pub fn with_interval_coefficients(self, interval_coefficients: IntervalCoefficients) -> Self {
        Self {
            interval_coefficients,
            ..self
        }
    }
}

impl UID for Deck {
    fn uid(&self) -> &str {
        &self.name[..]
    }
}

impl Merge<Deck> for Deck {
    fn merge(self, other: &Deck) -> Self {
        self.with_interval_coefficients(other.interval_coefficients.clone())
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    #[test]
    fn new() {
        let name = "cephelapoda";
        let card_paths = vec!["octopus", "squid", "cuttlefish", "nautilus"];
        let expected_card_paths = vec![
            String::from("octopus"),
            String::from("squid"),
            String::from("cuttlefish"),
            String::from("nautilus"),
        ];
        let interval_coefficients = IntervalCoefficients {
            pass_coef: 8.0,
            easy_coef: 9.0,
            fail_coef: 10.0,
        };
        let expected = Deck {
            name: name.to_string(),
            card_paths: expected_card_paths,
            interval_coefficients: interval_coefficients.clone(),
        };
        let actual = Deck::new(name, card_paths, interval_coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn with_interval_coefficients() {
        let name = "deck";
        let old_interval_coefficients = IntervalCoefficients::default();
        let new_interval_coefficients = IntervalCoefficients::new(8.0, 9.0, 10.0);
        let deck = Deck::new(name, vec!["a"], old_interval_coefficients);
        let mut expected = deck.clone();
        expected.interval_coefficients = new_interval_coefficients.clone();
        let actual = deck.with_interval_coefficients(new_interval_coefficients);
        assert_eq!(expected, actual);
    }

    #[test]
    fn uid() {
        let name = "The Deck";
        let deck = Deck::new(name, vec![], IntervalCoefficients::default());
        assert_eq!(name, deck.uid());
    }

    #[test]
    fn merge() {
        let a = Deck::new("a", vec![], IntervalCoefficients::default());
        let b = Deck::new("b", vec![], IntervalCoefficients::new(8.0, 9.0, 10.0));
        let mut expected = a.clone();
        expected.interval_coefficients = b.interval_coefficients.clone();
        assert_eq!(expected, a.merge(&b));
    }
}
