pub mod card;
pub mod deck;
pub mod hand;
mod tools;

use card::{parser::ParsingConfig, Card};
use deck::Deck;
use std::collections::HashMap;
use tools::Identifiable;

// TODO (de)serialise
#[derive(Debug, Default, PartialEq)]
struct State {
    card_parsing_config: ParsingConfig,
    cards: HashMap<String, Card>,
    decks: HashMap<String, Deck>,
}

impl State {
    fn new(card_parsing_config: ParsingConfig, cards: Vec<Card>, decks: Vec<Deck>) -> Self {
        Self {
            card_parsing_config,
            cards: cards
                .into_iter()
                .map(|c| (c.uid().to_string(), c))
                .collect(),
            decks: decks
                .into_iter()
                .map(|d| (d.uid().to_string(), d))
                .collect(),
        }
    }

    fn with_overriden_cards(mut self, cards: Vec<Card>) -> Self {
        self.cards
            .extend(cards.into_iter().map(|c| (c.path.clone(), c)));
        self
    }

    fn with_merged_cards(self, cards: Vec<Card>) -> Self {
        let overriding_cards: Vec<Card> = cards
            .into_iter()
            .map(|c| match self.cards.get(c.uid()) {
                Some(card) => c.with_revision_settings(card.revision_settings.clone()),
                None => c,
            })
            .collect();
        self.with_overriden_cards(overriding_cards)
    }

    fn with_decks(mut self, decks: Vec<Deck>) -> Self {
        self.decks
            .extend(decks.into_iter().map(|d| (d.name.clone(), d)));
        self
    }

    fn with_card_parsing_config(self, card_parsing_config: ParsingConfig) -> Self {
        Self {
            card_parsing_config,
            ..self
        }
    }

    // TODO
    fn get_all_cards_in_deck(deck_id: &str) -> Vec<&Card> {
        todo!()
    }

    // TODO
    fn get_deck(deck_id: &str) -> &Deck {
        todo!()
    }

    // TODO
    fn deal_hand(deck_id: &str) -> &Deck {
        todo!()
    }
}

#[cfg(test)]
mod unit_tests {

    use super::card::revision_settings::RevisionSettings;
    use super::*;
    use chrono::Utc;

    #[derive(Debug)]
    enum ExpectContains<T> {
        Yes(T),
        No(T),
    }

    fn fake_parsing_config_with_delimiter(delimiter: &str) -> ParsingConfig {
        let mut card_parsing_config = ParsingConfig::default();
        card_parsing_config.deck_delimiter = delimiter.to_string();
        card_parsing_config
    }

    fn fake_card_with_path_and_decks(path: &str, decks: Vec<&str>) -> Card {
        let mut card = Card::default();
        card.path = path.to_string();
        card.decks = decks.into_iter().map(|d| d.to_string()).collect();
        card
    }

    fn fake_deck_with_id(name: &str) -> Deck {
        let mut deck = Deck::default();
        deck.name = name.to_string();
        deck
    }

    fn fake_parsing_config_card_deck_and_state() -> (ParsingConfig, Card, Deck, State) {
        let deck_id = "a_deck";
        let card_parsing_config = fake_parsing_config_with_delimiter("///");
        let card = fake_card_with_path_and_decks("some/path", vec![deck_id]);
        let deck = fake_deck_with_id(deck_id);
        let state = State {
            card_parsing_config: card_parsing_config.clone(),
            cards: HashMap::from([(card.path.clone(), card.clone())]),
            decks: HashMap::from([(deck.name.clone(), deck.clone())]),
        };
        (card_parsing_config, card, deck, state)
    }

    fn state_map_length_matches<'a, T>(
        state_map: &HashMap<String, T>,
        expected: &'a Vec<ExpectContains<T>>,
    ) -> bool
    where
        T: Default,
    {
        let expected_length = expected
            .iter()
            .filter(|c| {
                std::mem::discriminant(*c)
                    == std::mem::discriminant(&ExpectContains::Yes(T::default()))
            })
            .count();
        state_map.len() == expected_length
    }

    fn state_map_contains<'a, T>(state_map: &HashMap<String, T>, item: &'a T) -> bool
    where
        T: PartialEq + tools::Identifiable<'a>,
    {
        state_map.contains_key(item.uid()) && *item == state_map[item.uid()]
    }

    fn assert_state_map_contains_all<'a, T>(
        state_map: &HashMap<String, T>,
        expected: &'a Vec<ExpectContains<T>>,
    ) where
        T: Default + std::fmt::Debug + PartialEq + tools::Identifiable<'a>,
    {
        assert!(state_map_length_matches(&state_map, &expected));
        for comparator in expected.iter() {
            println!("\n\n{:?}\n{:?}\n\n", state_map, comparator);
            match comparator {
                ExpectContains::Yes(item) => assert!(state_map_contains(state_map, item)),
                ExpectContains::No(item) => assert!(!state_map_contains(state_map, item)),
            }
        }
    }

    fn assert_state_eq(
        actual_state: &State,
        expected_parsing_config: &ParsingConfig,
        expected_cards: Vec<ExpectContains<Card>>,
        expected_decks: Vec<ExpectContains<Deck>>,
    ) {
        assert_eq!(*expected_parsing_config, actual_state.card_parsing_config);
        assert_state_map_contains_all(&actual_state.cards, &expected_cards);
        assert_state_map_contains_all(&actual_state.decks, &expected_decks);
    }

    #[test]
    fn default() {
        let expected = State {
            card_parsing_config: ParsingConfig::default(),
            cards: HashMap::new(),
            decks: HashMap::new(),
        };
        let actual = State::default();
        assert_eq!(expected, actual);
    }

    #[test]
    fn new() {
        let (card_parsing_config, card, deck, expected) = fake_parsing_config_card_deck_and_state();
        let cards = vec![card.clone()];
        let decks = vec![deck.clone()];
        let actual = State::new(card_parsing_config, cards, decks);

        assert_eq!(expected, actual);
    }

    #[test]
    fn with_overriden_cards_when_new_card_has_different_path_from_old_card() {
        let (parsing_config, old_card, deck, state) = fake_parsing_config_card_deck_and_state();
        let new_card = fake_card_with_path_and_decks("some/other/path", vec!["another_deck"]);
        let actual = state.with_overriden_cards(vec![new_card.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(old_card), ExpectContains::Yes(new_card)],
            vec![ExpectContains::Yes(deck)],
        );
    }

    #[test]
    fn with_overriden_cards_when_new_card_has_same_path_as_old_card() {
        let (parsing_config, old_card, deck, state) = fake_parsing_config_card_deck_and_state();
        let new_card = fake_card_with_path_and_decks(&old_card.path[..], vec!["another_deck"]);
        let actual = state.with_overriden_cards(vec![new_card.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::No(old_card), ExpectContains::Yes(new_card)],
            vec![ExpectContains::Yes(deck)],
        );
    }

    #[test]
    fn with_merged_cards_when_new_card_has_different_path_from_old_card() {
        let (parsing_config, old_card, deck, state) = fake_parsing_config_card_deck_and_state();
        let new_card = fake_card_with_path_and_decks("some/other/path", vec!["another_deck"]);
        let actual = state.with_merged_cards(vec![new_card.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(old_card), ExpectContains::Yes(new_card)],
            vec![ExpectContains::Yes(deck)],
        );
    }

    #[test]
    fn with_merged_cards_when_new_card_has_same_path_as_old_card() {
        let (parsing_config, old_card, deck, state) = fake_parsing_config_card_deck_and_state();
        let mut expected_card = fake_card_with_path_and_decks(old_card.uid(), vec!["another_deck"]);
        expected_card.revision_settings = old_card.revision_settings.clone();
        let mut new_card = expected_card.clone();
        new_card.revision_settings = RevisionSettings::new(Utc::now(), 9000.0, 1234567.5);
        let actual = state.with_merged_cards(vec![new_card.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![
                ExpectContains::No(old_card),
                ExpectContains::No(new_card),
                ExpectContains::Yes(expected_card),
            ],
            vec![ExpectContains::Yes(deck)],
        );
    }

    #[test]
    fn with_decks_when_new_deck_has_different_id_from_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let new_deck = fake_deck_with_id("a_new_deck_appears");
        let actual = state.with_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::Yes(old_deck), ExpectContains::Yes(new_deck)],
        );
    }

    #[test]
    fn with_decks_when_new_deck_has_same_id_as_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let mut new_deck = fake_deck_with_id(&old_deck.name[..]);
        new_deck.interval_coefficients.easy_coef = 9000.0;
        let actual = state.with_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::No(old_deck), ExpectContains::Yes(new_deck)],
        );
    }

    #[test]
    fn with_card_parsing_config() {
        let (_, card, deck, state) = fake_parsing_config_card_deck_and_state();
        let mut new_parsing_config = ParsingConfig::default();
        new_parsing_config.deck_delimiter = "?".to_string();
        let actual = state.with_card_parsing_config(new_parsing_config.clone());
        assert_state_eq(
            &actual,
            &new_parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::Yes(deck)],
        );
    }
}
