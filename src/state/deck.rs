pub mod interval_coefficients;

pub use interval_coefficients::IntervalCoefficients;
use serde::{Deserialize, Serialize};
use super::tools::Identifiable;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Deck {
    pub name: String,
    pub card_paths: Vec<String>,
    pub interval_coefficients: IntervalCoefficients,
}

impl Deck {
    pub fn new(name: &str, card_paths: Vec<&str>, interval_coefficients: IntervalCoefficients) -> Self {
        Self {
            name: name.to_string(),
            card_paths: card_paths.iter().map(|s| s.to_string()).collect(),
            interval_coefficients,
        }
    }
}

impl<'a> Identifiable<'a> for Deck {
    fn uid(&'a self) -> &'a str {
        &self.name[..]
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

    fn uid() {
        let name = "The Deck";
        let deck = Deck::new(name, vec![], IntervalCoefficients::default());
        assert_eq!(name, deck.uid());
    }
}
