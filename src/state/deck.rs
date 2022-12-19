pub mod interval_coefficients;

use super::card::Card;
use super::tools::{Merge, Near, UID};
pub use interval_coefficients::IntervalCoefficients;
use itertools::Itertools;
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

impl Near<Deck> for Deck {
    fn is_near(&self, other: &Deck) -> bool {
        self == other
    }
}

pub fn many_from_cards(cards: &[Card]) -> Vec<Deck> {
    let deck_name_to_paths = cards.into_iter().fold(
        std::collections::HashMap::new(),
        |mut deck_name_to_paths, card| {
            card.decks.iter().for_each(|deck_name| {
                deck_name_to_paths
                    .entry(deck_name)
                    .or_insert_with(|| vec![])
                    .push(card.path.as_str())
            });

            deck_name_to_paths
        },
    );

    deck_name_to_paths
        .into_iter()
        .map(|(deck_name, card_paths)| {
            Deck::new(&deck_name, card_paths, IntervalCoefficients::default())
        })
        .collect_vec()
}

pub mod fake {
    use super::*;
    pub fn deck(name: &str, paths: &[&str]) -> Deck {
        Deck::new(
            name,
            paths.iter().map(|s| s.to_owned()).collect_vec(),
            IntervalCoefficients::default(),
        )
    }
}

pub mod assertions {
    use super::*;

    pub fn assert_decks_eq(mut expected: Vec<Deck>, mut actual: Vec<Deck>) {
        assert!(expected.len() > 0 && actual.len() > 0 && expected.len() == actual.len());
        let comparator = |a: &Deck, b: &Deck| a.name.cmp(&b.name);
        expected.sort_by(comparator);
        actual.sort_by(comparator);
        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;
    use crate::state::card::fake::card as fake_card;
    use crate::state::tools::test_tools::ignore;

    #[test]
    fn many_decks_from_cards() {
        let card_a = fake_card("1.md", vec!["a", "b"], ignore(), ignore(), ignore());
        let card_b = fake_card("2.md", vec!["b", "c"], ignore(), ignore(), ignore());
        let card_c = fake_card("3.md", vec!["c", "d"], ignore(), ignore(), ignore());
        let cards = vec![card_a, card_b, card_c];
        let expected = vec![
            fake::deck("a", &vec!["1.md"]),
            fake::deck("b", &vec!["1.md", "2.md"]),
            fake::deck("c", &vec!["2.md", "3.md"]),
            fake::deck("d", &vec!["3.md"]),
        ];
        let actual = many_from_cards(&cards);
        assertions::assert_decks_eq(expected, actual);
    }

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
