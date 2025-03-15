mod shuffle;

use super::card::{Card, Score};
use super::deck::{Deck, IntervalCoefficients};
use anyhow::Result;
use custom_error::custom_error;
use std::collections::VecDeque;

use std::fs::File;
use std::io::prelude::*;

custom_error! {
    #[derive(PartialEq)]
    pub HandError
    EmptyDeck { name: String } = "Deck '{name}' contains no cards",
    NoDueCards { name: String } = "No due cards in Deck '{name}'",
    ReceivedExitApplicationSignal = ""
}

#[derive(Debug)]
pub struct Hand<'h> {
    queue: VecDeque<Card>,
    interval_coefficients: &'h IntervalCoefficients,
}

impl<'h> Hand<'h> {
    pub fn from(deck: &'h Deck, cards: Vec<&'h Card>) -> Result<Hand<'h>, HandError> {
        let deck_cards = Hand::filter_cards_in_deck(deck, cards);
        let n_cards_in_deck = deck_cards.len();
        let due_cards = Hand::filter_due_cards(deck_cards);
        let n_due_cards = due_cards.len();
        let hand_cards = shuffle::shuffle_cards(due_cards);
        let name = deck.name.to_owned();
        match (n_cards_in_deck, n_due_cards) {
            (0, _) => Err(HandError::EmptyDeck { name })?,
            (_, 0) => Err(HandError::NoDueCards { name })?,
            _ => Ok(Self {
                queue: hand_cards.into_iter().map(Clone::clone).collect(),
                interval_coefficients: &deck.interval_coefficients,
            }),
        }
    }

    pub fn revise_until_none_fail<ReadScoreCallback>(
        mut self,
        mut read_score: ReadScoreCallback,
    ) -> Result<Vec<Card>>
    where
        ReadScoreCallback: FnMut(&Card, usize) -> Result<Score>,
    {
        use Score::*;
        let mut output = Vec::new();
        while self.queue.len() > 0 {
            let n_remaining = self.queue.len();
            let card = self.queue.pop_front().unwrap();
            let transform = |card: Card, score| card.transform(score, self.interval_coefficients);
            match read_score(&card, n_remaining) {
                Ok(Fail) => self.queue.push_back(transform(card, Fail)),
                Ok(any_other_score) => output.push(transform(card, any_other_score)),
                Err(e) => {
                    if matches!(
                        e.downcast_ref::<HandError>(),
                        Some(HandError::ReceivedExitApplicationSignal)
                    ) {
                        // TODO TEST
                        output.push(card);
                        output.extend(self.queue.iter().cloned());
                        return Ok(output);
                    }
                }
            }
        }
        Ok(output)
    }

    pub fn number_of_due_cards(&self) -> usize {
        self.queue.len()
    }

    fn filter_cards_in_deck(deck: &'h Deck, cards: Vec<&'h Card>) -> Vec<&'h Card> {
        cards
            .into_iter()
            .filter(|c| c.in_deck(&deck.name))
            .collect()
    }

    fn filter_due_cards(cards: Vec<&'h Card>) -> Vec<&'h Card> {
        cards.into_iter().filter(|c| c.is_due()).collect()
    }
}

#[cfg(test)]
pub mod assertions {

    use super::*;
    use crate::state::card::assertions::assert_cards_near;
    use crate::state::tools::test_tools::{assertions::assert_length_matches, Expect};

    pub fn assert_hands_near(a: &[Card], b: &[Card]) {
        assert!(a.len() == b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_cards_near(x, y);
        }
    }

    pub fn assert_hand_contains(
        hand: &Hand,
        expected_coefficients: &IntervalCoefficients,
        expected_queued_items: &[Expect<Card>],
    ) {
        assert_eq!(hand.interval_coefficients, expected_coefficients);
        assert_length_matches(&hand.queue, &expected_queued_items);
        for comparator in expected_queued_items.iter() {
            match comparator {
                Expect::DoesContain(item) => assert!(hand.queue.contains(&item)),
                Expect::DoesNotContain(item) => assert!(!hand.queue.contains(&item)),
                _ => panic!("BAD TEST"),
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;
    use crate::state::card::revision_settings::test_tools::make_expected_revision_settings;
    use crate::state::{card::RevisionSettings, deck::IntervalCoefficients};
    use chrono::{Duration, Utc};
    use itertools::*;
    use rstest::*;

    const FAKE_DECK_NAME: &str = "cephelapoda";

    fn make_card(path: &str, deck: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![deck.to_string()],
            format!("{:?}?", path),
            format!("yes, {:?}", path),
            RevisionSettings::default(),
        )
    }

    fn make_future_card(path: &str, deck: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![deck.to_string()],
            format!("{:?}?", path),
            format!("yes, {:?}", path),
            RevisionSettings::new(Utc::now() + chrono::Duration::days(10), 0.0, 1300.0),
        )
    }

    fn make_card_with_revision_settings(
        path: &str,
        deck: &str,
        revision_settings: &RevisionSettings,
    ) -> Card {
        let mut card = make_card(path, deck);
        card.revision_settings = revision_settings.to_owned();
        card
    }

    fn make_deck(name: &str, card_paths: &[&str]) -> Deck {
        Deck::new(name, card_paths.to_owned(), IntervalCoefficients::default())
    }

    fn make_cards(deck_id: &str, card_paths: &[&str]) -> Vec<Card> {
        card_paths.iter().map(|p| make_card(p, deck_id)).collect()
    }

    fn concat_cards(a: Vec<Card>, b: Vec<Card>) -> Vec<Card> {
        vec![a, b].concat()
    }

    fn fake_future_card(path: &str) -> Card {
        let mut card = make_card(path, FAKE_DECK_NAME);
        card.revision_settings.due = Utc::now() + Duration::days(4);
        card
    }

    fn fake_cards(paths: Vec<&str>) -> Vec<Card> {
        make_cards(FAKE_DECK_NAME, &paths)
    }

    #[rstest]
    #[case::creates_shuffled_card_queue_from_deck_and_cards(
        fake_cards(vec!["octopus", "squid", "cuttlefish", "nautilus"]),
        Ok(vec!["squid", "cuttlefish", "nautilus", "octopus"])
    )]
    #[case::creates_shuffled_card_queue_containing_due_cards_only(
        concat_cards(fake_cards(vec!["squid", "cuttlefish", "nautilus"]), vec![fake_future_card("octopus")]),
        Ok(vec!["cuttlefish", "nautilus", "squid"])
    )]
    #[case::creates_shuffled_card_queue_containing_cards_in_deck_only(
        concat_cards(fake_cards(vec!["octopus", "squid", "cuttlefish", "nautilus"]), vec![make_card("clam", "bivalvia")]),
        Ok(vec!["squid", "cuttlefish", "nautilus", "octopus"])
    )]
    #[case::returns_empty_deck_error_if_no_cards_exist_for_deck(vec![make_card("clam", "bivalvia")], Err(HandError::EmptyDeck{name: FAKE_DECK_NAME.to_owned()}))]
    #[case::returns_no_due_cards_error_if_no_cards_due_in_deck(vec![make_future_card("squid", FAKE_DECK_NAME)], Err(HandError::NoDueCards{name: FAKE_DECK_NAME.to_owned()}))]
    fn from(#[case] cards: Vec<Card>, #[case] expected: Result<Vec<&str>, HandError>) {
        let card_paths: Vec<&str> = cards.iter().map(|c| c.path.as_str()).collect();
        let deck = make_deck(FAKE_DECK_NAME, &card_paths);
        let hand = Hand::from(&deck, cards.iter().collect());
        match hand {
            Ok(hand) => {
                let expected = make_cards(FAKE_DECK_NAME, &expected.expect("BAD TEST. Expected"));
                let actual: Vec<Card> = hand.queue.into_iter().collect();
                assertions::assert_hands_near(&expected, &actual);
            }
            Err(err) => {
                assert_eq!(expected.unwrap_err(), err);
            }
        }
    }

    #[test]
    fn revise_until_none_fail_with_empty_queue() {
        let interval_coefficients = IntervalCoefficients::default();
        let hand = Hand {
            queue: VecDeque::new(),
            interval_coefficients: &&interval_coefficients,
        };
        let actual = hand.revise_until_none_fail(|card, n_remaining| Ok(Score::Easy));
        assert!(actual.expect("Expected empty vec").len() == 0);
    }

    #[test]
    fn revise_until_none_fail_transforms_cards_based_on_their_score() {
        let deck_id = "some_deck";
        let in_date = Utc::now() - Duration::days(4);
        let input_revision_settings = RevisionSettings::new(in_date, 1.0, 2000.0);
        let input_card_paths = vec!["hard", "pass", "easy"];
        let cards: Vec<Card> = input_card_paths
            .iter()
            .map(|path| make_card_with_revision_settings(path, deck_id, &input_revision_settings))
            .collect();
        let interval_coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let deck = Deck::new(deck_id, input_card_paths.to_owned(), interval_coefficients);
        let hand = Hand::from(&deck, cards.iter().collect()).unwrap();
        let expected_specs = vec![
            ("pass", 6.0, 2000.0),
            ("easy", 20.0, 2150.0),
            ("hard", 2.4, 1850.0),
        ];
        let expected: Vec<Card> = expected_specs
            .into_iter()
            .map(|(p, i, f)| {
                let revision_settings = make_expected_revision_settings(&in_date, i, f);
                make_card_with_revision_settings(p, deck_id, &revision_settings)
            })
            .collect();

        let mut expected_remaining = cards.len();
        let actual = hand
            .revise_until_none_fail(|card, n_remaining| {
                assert_eq!(expected_remaining, n_remaining);
                expected_remaining -= 1;
                match &card.path[..] {
                    "hard" => Ok(Score::Hard),
                    "pass" => Ok(Score::Pass),
                    "easy" => Ok(Score::Easy),
                    _ => panic!("IMPOSSIBLE"),
                }
            })
            .expect("Expected vec of cards");

        assertions::assert_hands_near(&expected, &actual);
    }

    #[test]
    fn number_of_due_cards() {
        let deck_id = "some_deck";
        let due_card_date = Utc::now() - Duration::days(4);
        let due_card_rs = RevisionSettings::new(due_card_date, 1.0, 2000.0);
        let not_due_card_date = Utc::now() + Duration::days(4);
        let not_due_card_rs = RevisionSettings::new(not_due_card_date, 1.0, 2000.0);
        let (path_1, path_2) = ("path_1", "path_2");
        let due_card = make_card_with_revision_settings(path_1, deck_id, &due_card_rs);
        let not_due_card = make_card_with_revision_settings(path_2, deck_id, &not_due_card_rs);
        let cards = vec![&due_card, &not_due_card];
        let interval_coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let deck = Deck::new(deck_id, vec![path_1, path_2], interval_coefficients);
        let hand = Hand::from(&deck, cards).unwrap();
        assert_eq!(1, hand.number_of_due_cards());
    }

    #[test]
    fn revise_until_none_fail_cycles_for_failed_cards() {
        let deck_id = "some_deck";
        let in_date = Utc::now() - Duration::days(4);
        let in_rs = RevisionSettings::new(in_date, 1.0, 2000.0);
        let path = "fail";
        let card = make_card_with_revision_settings(path, deck_id, &in_rs);
        let cards = vec![&card];
        let interval_coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let deck = Deck::new(deck_id, vec![path], interval_coefficients);
        let hand = Hand::from(&deck, cards).unwrap();
        let out_rs = make_expected_revision_settings(&in_date, 2.6, 1300.0);
        let expected = vec![make_card_with_revision_settings(path, deck_id, &out_rs)];

        let mut total_number_of_cycles = 0;
        let actual = hand
            .revise_until_none_fail(|card, n_remaining| match &card.path[..] {
                "fail" => {
                    let number_of_cycles_so_far = total_number_of_cycles;
                    if number_of_cycles_so_far < 5 {
                        total_number_of_cycles += 1;
                        Ok(Score::Fail)
                    } else {
                        Ok(Score::Pass)
                    }
                }
                _ => panic!("IMPOSSIBLE"),
            })
            .expect("Expected vec of cards");

        assert_eq!(total_number_of_cycles, 5);
        assertions::assert_hands_near(&expected, &actual);
    }
}
