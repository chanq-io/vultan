pub mod card;
pub mod deck;
pub mod hand;
mod tools;

use card::{parser::ParsingConfig, Card};
use deck::Deck;
use hand::Hand;
use std::collections::HashMap;
use tools::{UID, Merge};

// TODO (de)serialise
#[derive(Debug, Default, PartialEq)]
struct State {
    card_parsing_config: ParsingConfig,
    cards: HashMap<String, Card>,
    decks: HashMap<String, Deck>,
}

impl State {
    pub fn new(card_parsing_config: ParsingConfig, cards: Vec<Card>, decks: Vec<Deck>) -> Self {
        Self {
            card_parsing_config,
            cards: HashMap::from_iter(Self::uid_value_pairs(cards).into_iter()),
            decks: HashMap::from_iter(Self::uid_value_pairs(decks).into_iter()),
        }
    }

    // TODO test
    // TODO impl
    pub fn read(notes_directory_path: &str) -> Self {
        // let state = Self::read_or_default(notes_dir)
        // let card_parser = Parser::from(&state.card_parsing_config);
        // let cards = Card::read_all_or_default(&card_parser, &args.notes_dir);
        // let decks = Deck::many_from_cards(cards);
        // state.with_merged_cards(cards).with_merged_decks(decks)
        todo!()
    }

    // TODO test
    // TODO impl
    pub fn write(notes_directory_path: &str) {
        todo!()
    }

    pub fn with_overriden_cards(self, cards: Vec<Card>) -> Self {
        Self {
            cards: Self::override_matching_values(self.cards, cards),
            ..self
        }
    }

    pub fn with_overriden_decks(self, decks: Vec<Deck>) -> Self {
        Self {
            decks: Self::override_matching_values(self.decks, decks),
            ..self
        }
    }

    pub fn with_card_parsing_config(self, card_parsing_config: ParsingConfig) -> Self {
        Self {
            card_parsing_config,
            ..self
        }
    }

    // TODO test
    // TODO impl
    pub fn deal(deck_name: &str) -> Hand {
        todo!()
    }

    // TODO test
    // TODO impl
    fn read_file_or_default(notes_dir_path: &str) {
        todo!()
    }

    fn with_merged_cards(self, cards: Vec<Card>) -> Self {
        Self {
            cards: Self::merge_matching_values(self.cards, cards),
            ..self
        }
    }

    fn with_merged_decks(self, decks: Vec<Deck>) -> Self {
        Self {
            decks: Self::merge_matching_values(self.decks, decks),
            ..self
        }
    }

    fn override_matching_values<T: UID>(
        map: HashMap<String, T>,
        items: Vec<T>,
    ) -> HashMap<String, T> {
        let mut m = map;
        m.extend(items.into_iter().map(|i| (i.uid().to_string(), i)));
        m
    }

    fn merge_matching_values<T: Merge<T> + UID>(
        map: HashMap<String, T>,
        items: Vec<T>,
    ) -> HashMap<String, T> {
        let overriding: Vec<T> = items
            .into_iter()
            .map(|i| match map.get(i.uid()) {
                Some(item) => i.merge(&item),
                None => i,
            })
            .collect();
        State::override_matching_values(map, overriding)
    }

    fn uid_value_pairs<T: UID>(items: Vec<T>) -> Vec<(String, T)> {
        items
            .into_iter()
            .map(|i| (i.uid().to_string(), i))
            .collect()
    }
}

#[cfg(test)]
mod unit_tests {

    use super::card::revision_settings::RevisionSettings;
    use super::deck::interval_coefficients::IntervalCoefficients;
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

    fn fake_deck_with_name(name: &str) -> Deck {
        let mut deck = Deck::default();
        deck.name = name.to_string();
        deck
    }

    fn fake_parsing_config_card_deck_and_state() -> (ParsingConfig, Card, Deck, State) {
        let deck_name = "a_deck";
        let card_parsing_config = fake_parsing_config_with_delimiter("///");
        let card = fake_card_with_path_and_decks("some/path", vec![deck_name]);
        let deck = fake_deck_with_name(deck_name);
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
        T: PartialEq + tools::UID,
    {
        state_map.contains_key(item.uid()) && *item == state_map[item.uid()]
    }

    fn assert_state_map_contains_all<'a, T>(
        state_map: &HashMap<String, T>,
        expected: &'a Vec<ExpectContains<T>>,
    ) where
        T: Default + std::fmt::Debug + PartialEq + tools::UID,
    {
        assert!(state_map_length_matches(&state_map, &expected));
        for comparator in expected.iter() {
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
    fn with_overriden_decks_when_new_deck_has_different_name_from_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let new_deck = fake_deck_with_name("a_new_deck_appears");
        let actual = state.with_overriden_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::Yes(old_deck), ExpectContains::Yes(new_deck)],
        );
    }

    #[test]
    fn with_overriden_decks_when_new_deck_has_same_name_as_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let mut new_deck = fake_deck_with_name(&old_deck.name[..]);
        new_deck.interval_coefficients.easy_coef = 9000.0;
        let actual = state.with_overriden_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::No(old_deck), ExpectContains::Yes(new_deck)],
        );
    }

    #[test]
    fn with_merged_decks_when_new_deck_has_different_name_from_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let new_deck = fake_deck_with_name("a_new_deck_appears");
        let actual = state.with_merged_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![ExpectContains::Yes(old_deck), ExpectContains::Yes(new_deck)],
        );
    }

    #[test]
    fn with_merged_decks_when_new_deck_has_same_name_as_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_parsing_config_card_deck_and_state();
        let mut expected_deck = old_deck.clone();
        expected_deck.card_paths = vec!["a/new/path".to_string(), "another/new/path".to_string()];
        let mut new_deck = expected_deck.clone();
        new_deck.interval_coefficients = IntervalCoefficients::new(31.0, 32.0, 33.0);
        let actual = state.with_merged_decks(vec![new_deck.clone()]);
        assert_state_eq(
            &actual,
            &parsing_config,
            vec![ExpectContains::Yes(card)],
            vec![
                ExpectContains::No(old_deck),
                ExpectContains::No(new_deck),
                ExpectContains::Yes(expected_deck),
            ],
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
