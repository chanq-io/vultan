use rand::seq::SliceRandom;

#[cfg(test)]
use rand::rngs::mock::StepRng;

#[cfg(not(test))]
use rand::thread_rng;

use crate::state::card::Card;

pub fn shuffle_cards(iterable: Vec<Card>) -> Vec<Card> {
    #[cfg(test)]
    let mut random_number_generator = StepRng::new(0, 0);
    #[cfg(not(test))]
    let mut random_number_generator = thread_rng();
    let mut output = iterable.to_owned();
    output.shuffle(&mut random_number_generator);
    output
}

#[cfg(test)]
mod unit_tests {

    use super::*;
    use crate::state::card::RevisionSettings;

    fn make_fake_card(path: &str) -> Card {
        Card::new(
            path.to_string(),
            vec![],
            "".to_string(),
            "".to_string(),
            RevisionSettings::default(),
        )
    }

    #[test]
    fn shuffling_cards() {
        let card_paths = vec!["octopus", "squid", "cuttlefish", "nautilus"];
        let deck_id = "cephelapoda";
        let cards: Vec<Card> = card_paths.iter().map(|p| make_fake_card(p)).collect();
        let expected_paths = vec!["squid", "cuttlefish", "nautilus", "octopus"];
        let actual_cards = shuffle_cards(cards);
        let actual_paths: Vec<&str> = actual_cards.iter().map(|c| &c.path[..]).collect();
        assert_eq!(expected_paths, actual_paths);
    }
}
