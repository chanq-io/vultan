mod shuffle;

use super::card::{Card, Score};
use super::deck::Deck;
use std::collections::VecDeque;

pub struct Hand {
    queue: VecDeque<Card>,
}

impl Hand {
    pub fn from(deck: &Deck, cards: &Vec<Card>) -> Hand {
        let is_due_and_in_deck = |c: &Card| c.is_due() && c.in_deck(&deck.id);
        let due_deck_cards = cards
            .to_owned()
            .into_iter()
            .filter(is_due_and_in_deck)
            .collect();
        Self {
            queue: shuffle::shuffle_cards(due_deck_cards).into_iter().collect(),
        }
    }

    // TODO test returns empty list of cards for empty queue
    // TODO test returns cards transformed based on their score
    // TODO test repeats failed cards
    pub fn revise_until_all_pass<ReadScoreCallback>(
        self,
        read_score: ReadScoreCallback,
    ) -> Vec<Card>
    where
        ReadScoreCallback: FnMut(&Card) -> Score, // maybe doesn't need mut
    {
        let revised = Vec::new();
        revised
    }

    /*
    pub fn cycle_until_revised<Callback>(mut self, mut callback: Callback) -> Vec<Card>
    where
        Callback: FnMut(RevisableCard) -> RevisableCard, // maybe doesn't need mut
    {
        use RevisableCard::*;
        let mut revised = Vec::new();
        while self.queue.len() > 0 {
            let revisable_card = self.queue.pop_front().unwrap();
            match callback(revisable_card) {
                HasBeenRevised(card) => {
                    revised.push(card.to_owned());
                }
                ToBeRevised(card) => {
                    self.queue.push_back(ToBeRevised(card));
                }
            }
        }
        revised
    }
    */
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

    fn make_fake_card(path: &str, deck: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![deck.to_string()],
            format!("{:?}?", path),
            format!("yes, {:?}", path),
            RevisionSettings::default(),
        )
    }

    fn make_fake_card_with_due_date(path: &str, deck: &str, due: DateTime<Utc>) -> Card {
        let mut card = make_fake_card(path, deck);
        card.revision_settings.due = due;
        card
    }

    fn make_fake_deck(id: &str, card_paths: &Vec<&str>) -> Deck {
        Deck::new(id, card_paths.to_owned(), IntervalCoefficients::default())
    }

    fn make_fake_cards(deck_id: &str, card_paths: &Vec<&str>) -> Vec<Card> {
        card_paths.iter().map(|p| make_fake_card(p, deck_id)).collect()
    }

    fn make_fake_deck_and_cards(deck_id: &str, card_paths: Vec<&str>) -> (Deck, Vec<Card>) {
        (make_fake_deck(deck_id, &card_paths), make_fake_cards(deck_id, &card_paths))
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
        let clam_card = make_fake_card("clam", "bivalvia");
        cards.push(clam_card);
        let hand = Hand::from(&deck, &cards);
        let expected_card_paths = vec!["squid", "cuttlefish", "nautilus", "octopus"];
        let expected = make_fake_cards(deck_id, &expected_card_paths);
        let actual: Vec<Card> = hand.queue.into_iter().collect();
        assertions::assert_near(&expected, &actual);
    }

    /*
    #[test]
    fn sandbox() {
        let octopus_card = make_fake_card("octopus");
        let squid_card = make_fake_card("squid");
        let cuttlefish_card = make_fake_card("cuttlefish");
        let nautilus_card = make_fake_card("nautilus");
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
