mod shuffle;

use super::card::{Card, Score};
use super::deck::{Deck, IntervalCoefficients};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Hand<'h> {
    queue: VecDeque<Card>,
    interval_coefficients: &'h IntervalCoefficients,
}

impl<'h> Hand<'h> {
    pub fn from(deck: &'h Deck, cards: Vec<&'h Card>) -> Result<Hand<'h>, String> {
        let hand_cards = shuffle::shuffle_cards(Hand::filter_due_cards_in_deck(deck, cards));
        match hand_cards.len() {
            0 => Err(format!("Deck({}) contains no cards", deck.name)),
            _ => Ok(Self {
                queue: hand_cards.into_iter().collect(),
                interval_coefficients: &deck.interval_coefficients,
            }),
        }
    }

    pub fn revise_until_none_fail<ReadScoreCallback>(
        mut self,
        mut read_score: ReadScoreCallback,
    ) -> Vec<Card>
    where
        ReadScoreCallback: FnMut(&Card) -> Score,
    {
        use Score::*;
        let mut output = Vec::new();
        while self.queue.len() > 0 {
            let card = self.queue.pop_front().unwrap();
            let transform = |card: Card, score| card.transform(score, self.interval_coefficients);
            match read_score(&card) {
                Fail => self.queue.push_back(transform(card, Fail)),
                any_other_score => output.push(transform(card, any_other_score)),
            }
        }
        output
    }

    fn filter_due_cards_in_deck(deck: &'h Deck, cards: Vec<&'h Card>) -> Vec<Card> {
        cards
            .into_iter()
            .filter(|c| c.is_due() && c.in_deck(&deck.name))
            .map(|c| c.clone())
            .collect()
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
    use rstest::*;

    const FAKE_DECK_ID: &str = "cephelapoda";

    fn make_card(path: &str, deck: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![deck.to_string()],
            format!("{:?}?", path),
            format!("yes, {:?}", path),
            RevisionSettings::default(),
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
        let mut card = make_card(path, FAKE_DECK_ID);
        card.revision_settings.due = Utc::now() + Duration::days(4);
        card
    }

    fn fake_cards(paths: Vec<&str>) -> Vec<Card> {
        make_cards(FAKE_DECK_ID, &paths)
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
    #[case::returns_error_if_no_cards_exist_for_deck(vec![make_card("clam", "bivalvia")], Err(FAKE_DECK_ID))]
    fn from(#[case] cards: Vec<Card>, #[case] expected: Result<Vec<&str>, &str>) {
        let card_paths: Vec<&str> = cards.iter().map(|c| c.path.as_str()).collect();
        let deck = make_deck(FAKE_DECK_ID, &card_paths);
        let hand = Hand::from(&deck, cards.iter().collect());
        match hand {
            Ok(hand) => {
                let expected = make_cards(FAKE_DECK_ID, &expected.expect("BAD TEST"));
                let actual: Vec<Card> = hand.queue.into_iter().collect();
                assertions::assert_hands_near(&expected, &actual);
            }
            Err(err) => {
                assert!(err.contains(FAKE_DECK_ID));
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
        let expected: Vec<Card> = Vec::new();
        let actual = hand.revise_until_none_fail(|card| Score::Easy);
        assert_eq!(expected, actual);
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

        let actual = hand.revise_until_none_fail(|card| match &card.path[..] {
            "hard" => Score::Hard,
            "pass" => Score::Pass,
            "easy" => Score::Easy,
            _ => panic!("IMPOSSIBLE"),
        });

        assertions::assert_hands_near(&expected, &actual);
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
        let actual = hand.revise_until_none_fail(|card| match &card.path[..] {
            "fail" => {
                let number_of_cycles_so_far = total_number_of_cycles;
                if number_of_cycles_so_far < 5 {
                    total_number_of_cycles += 1;
                    Score::Fail
                } else {
                    Score::Pass
                }
            }
            _ => panic!("IMPOSSIBLE"),
        });

        assert_eq!(total_number_of_cycles, 5);
        assertions::assert_hands_near(&expected, &actual);
    }
}
