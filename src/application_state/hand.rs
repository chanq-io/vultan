mod shuffle;

use super::card::{Card, Score};
use super::deck::{Deck, IntervalCoefficients};
use std::collections::VecDeque;

pub struct Hand<'hand> {
    queue: VecDeque<Card>,
    interval_coefficients: &'hand IntervalCoefficients
}

impl<'hand> Hand<'hand> {
    pub fn from(deck: &'hand Deck, cards: &'hand Vec<Card>) -> Hand<'hand> {
        let is_due_and_in_deck = |c: &Card| c.is_due() && c.in_deck(&deck.id);
        let due_deck_cards = cards
            .to_owned()
            .into_iter()
            .filter(is_due_and_in_deck)
            .collect();
        Self {
            queue: shuffle::shuffle_cards(due_deck_cards).into_iter().collect(),
            interval_coefficients: &deck.interval_coefficients
        }
    }

    // TODO test repeats failed cards
    pub fn revise_until_none_fail<ReadScoreCallback>(
        mut self,
        mut read_score: ReadScoreCallback,
    ) -> Vec<Card>
    where
        ReadScoreCallback: FnMut(&Card) -> Score
    {
        use Score::*;
        let mut revised = Vec::new();
        while self.queue.len() > 0 {
            let card = self.queue.pop_front().unwrap();
            match read_score(&card) {
                Easy => revised.push(card.transform(Easy, self.interval_coefficients)),
                Pass => revised.push(card.transform(Pass, self.interval_coefficients)),
                Hard => revised.push(card.transform(Hard, self.interval_coefficients)),
                Fail => self.queue.push_back(card.transform(Fail, self.interval_coefficients)),
            }
        }
        revised
    }
}

#[cfg(test)]
mod assertions {

    use super::*;
    use crate::application_state::card::assertions::assert_near as assert_cards_near;

    pub fn assert_near(a: &Vec<Card>, b: &Vec<Card>) {
        assert!(a.len() == b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_cards_near(x, y);
        }
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;
    use crate::application_state::{card::RevisionSettings, deck::IntervalCoefficients};
    use chrono::{DateTime, Duration, Utc};

    fn make_fake_card_from_path_and_deck(path: &str, deck: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![deck.to_string()],
            format!("{:?}?", path),
            format!("yes, {:?}", path),
            RevisionSettings::default(),
        )
    }

    fn make_fake_card_from_path_deck_and_revision_settings(
        path: &str,
        deck: &str,
        revision_settings: &RevisionSettings,
    ) -> Card {
        let mut card = make_fake_card_from_path_and_deck(path, deck);
        card.revision_settings = revision_settings.to_owned();
        card
    }

    fn make_fake_card_with_due_date(path: &str, deck: &str, due: DateTime<Utc>) -> Card {
        let mut card = make_fake_card_from_path_and_deck(path, deck);
        card.revision_settings.due = due;
        card
    }

    fn make_fake_deck(id: &str, card_paths: &Vec<&str>) -> Deck {
        Deck::new(id, card_paths.to_owned(), IntervalCoefficients::default())
    }

    fn make_fake_cards(deck_id: &str, card_paths: &Vec<&str>) -> Vec<Card> {
        card_paths
            .iter()
            .map(|p| make_fake_card_from_path_and_deck(p, deck_id))
            .collect()
    }

    fn make_fake_deck_and_cards(deck_id: &str, card_paths: Vec<&str>) -> (Deck, Vec<Card>) {
        (
            make_fake_deck(deck_id, &card_paths),
            make_fake_cards(deck_id, &card_paths),
        )
    }

    #[test]
    fn from_creates_shuffled_card_queue_from_deck_and_cards() {
        let input_card_paths = vec!["octopus", "squid", "cuttlefish", "nautilus"];
        let deck_id = "cephelapoda";
        let (deck, cards) = make_fake_deck_and_cards(deck_id, input_card_paths);
        let hand = Hand::from(&deck, &cards);
        let expected_card_paths = vec!["squid", "cuttlefish", "nautilus", "octopus"];
        let expected = make_fake_cards(deck_id, &expected_card_paths);
        let actual: Vec<Card> = hand.queue.into_iter().collect();
        assertions::assert_near(&expected, &actual);
    }

    #[test]
    fn from_creates_shuffled_card_queue_containing_due_cards_only() {
        let input_card_paths = vec!["squid", "cuttlefish", "nautilus"];
        let deck_id = "cephelapoda";
        let (deck, mut cards) = make_fake_deck_and_cards(deck_id, input_card_paths);
        let future_due_date = Utc::now() + Duration::days(4);
        let octopus_card = make_fake_card_with_due_date("octopus", deck_id, future_due_date);
        cards.push(octopus_card);
        let hand = Hand::from(&deck, &cards);
        let expected_card_paths = vec!["cuttlefish", "nautilus", "squid"];
        let expected = make_fake_cards(deck_id, &expected_card_paths);
        let actual: Vec<Card> = hand.queue.into_iter().collect();
        assertions::assert_near(&expected, &actual);
    }

    #[test]
    fn from_creates_shuffled_card_queue_containing_cards_in_deck_only() {
        let input_card_paths = vec!["octopus", "squid", "cuttlefish", "nautilus"];
        let deck_id = "cephelapoda";
        let (deck, mut cards) = make_fake_deck_and_cards(deck_id, input_card_paths);
        let clam_card = make_fake_card_from_path_and_deck("clam", "bivalvia");
        cards.push(clam_card);
        let hand = Hand::from(&deck, &cards);
        let expected_card_paths = vec!["squid", "cuttlefish", "nautilus", "octopus"];
        let expected = make_fake_cards(deck_id, &expected_card_paths);
        let actual: Vec<Card> = hand.queue.into_iter().collect();
        assertions::assert_near(&expected, &actual);
    }

    #[test]
    fn revise_until_none_fail_with_empty_queue() {
        let input_card_paths = vec![];
        let deck_id = "some_deck";
        let (deck, cards) = make_fake_deck_and_cards(deck_id, input_card_paths);
        let hand = Hand::from(&deck, &cards);
        let expected: Vec<Card> = Vec::new();
        let actual = hand.revise_until_none_fail(|card| Score::Easy);
        assert_eq!(expected, actual);
    }

    // TODO remove duplication
    fn make_expected_transformed_revision_settings(
        original_due_date: &DateTime<Utc>,
        interval: f64,
        factor: f64,
    ) -> RevisionSettings {
        RevisionSettings::new(
            original_due_date.to_owned() + Duration::seconds((86400.0 * interval) as i64),
            interval,
            factor,
        )
    }

    #[test]
    fn revise_until_none_fail_transforms_cards_based_on_their_score() {
        let deck_id = "some_deck";
        let original_due_date = Utc::now() - Duration::days(4);
        let input_revision_settings = RevisionSettings::new(original_due_date, 1.0, 2000.0);
        let input_card_paths = vec!["hard", "pass", "easy"];
        let cards = input_card_paths
            .iter()
            .map(|path| {
                make_fake_card_from_path_deck_and_revision_settings(
                    path,
                    deck_id,
                    &input_revision_settings,
                )
            })
            .collect();
        let interval_coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let deck = Deck::new(deck_id, input_card_paths.to_owned(), interval_coefficients);
        let hand = Hand::from(&deck, &cards);
        let expected_hard_revision_settings =
            make_expected_transformed_revision_settings(&original_due_date, 2.4, 1850.0);
        let expected_pass_revision_settings =
            make_expected_transformed_revision_settings(&original_due_date, 6.0, 2000.0);
        let expected_easy_revision_settings =
            make_expected_transformed_revision_settings(&original_due_date, 20.0, 2150.0);
        let expected_specs = vec![
            ("pass", expected_pass_revision_settings),
            ("easy", expected_easy_revision_settings),
            ("hard", expected_hard_revision_settings),
        ];
        let expected: Vec<Card> = expected_specs
            .into_iter()
            .map(|(p, rs)| make_fake_card_from_path_deck_and_revision_settings(p, deck_id, &rs))
            .collect();
        let actual = hand.revise_until_none_fail(|card| match &card.path[..] {
            "hard" => Score::Hard,
            "pass" => Score::Pass,
            "easy" => Score::Easy,
            _ => panic!("IMPOSSIBLE"),
        });
        assertions::assert_near(&expected, &actual);
    }

    /*
    #[test]
    fn sandbox() {
        let octopus_card = make_fake_card_from_path_and_deck("octopus");
        let squid_card = make_fake_card_from_path_and_deck("squid");
        let cuttlefish_card = make_fake_card_from_path_and_deck("cuttlefish");
        let nautilus_card = make_fake_card_from_path_and_deck("nautilus");
        let hand = Hand::new(&vec![
            octopus_card,
            squid_card,
            cuttlefish_card,
            nautilus_card,
        ]);
        let mut x = 0;
        let actual = hand.cycle_until_revised(|revisable_card| {
            let c = revisable_card.clone();
            if let RevisableCard::ToBeRevised(card) = c {
                if x != 0 {
                    x = (x + 1) % 3;
                    return RevisableCard::HasBeenRevised(card);
                }
            }
            x = (x + 1) % 3;
            revisable_card.to_owned()
        });

        println!("{:?}", actual);
        assert!(false);
    }
    */
}
