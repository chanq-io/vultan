pub mod card;
pub mod deck;
pub mod file;
pub mod hand;
pub mod tools;

use anyhow::{Context, Result};
use card::{
    parser::{Parser, ParsingConfig},
    Card,
};
use custom_error::custom_error;
use deck::Deck;
use file::FileHandle;
use hand::Hand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tools::{Merge, IO, UID};

#[cfg(test)]
use mocks::to_string_pretty as serialise;
#[cfg(not(test))]
use ron::ser::to_string_pretty as serialise;

pub const STATE_FILENAME: &str = ".vultan.ron";

custom_error! { pub StateError
    MissingDeck { name: String } = "No deck named '{name}' exists",
    EmptyDeck { name: String } = "Deck '{name}' contains no cards",
    NoDueCards { name: String } = "No due cards in Deck '{name}'",
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct State {
    pub card_parsing_config: ParsingConfig,
    pub cards: HashMap<String, Card>,
    pub decks: HashMap<String, deck::Deck>,
}

impl State {
    pub fn new(
        card_parsing_config: ParsingConfig,
        cards: Vec<Card>,
        decks: Vec<deck::Deck>,
    ) -> Self {
        Self {
            card_parsing_config,
            cards: HashMap::from_iter(Self::uid_value_pairs(cards)),
            decks: HashMap::from_iter(Self::uid_value_pairs(decks)),
        }
    }

    pub fn read(notes_dirpath: std::path::PathBuf) -> Result<Self> {
        let file_handle = FileHandle::from(notes_dirpath.join(STATE_FILENAME));
        let state = Self::from_file(file_handle).unwrap_or_default();
        let parser = Parser::from(&state.card_parsing_config)?;
        let loaded_cards = card::try_load_many(notes_dirpath, &parser)?;
        let decks = deck::many_from_cards(&loaded_cards.succeeded);
        Ok(state
            .with_merged_cards(loaded_cards.succeeded)
            .with_merged_decks(decks))
    }

    pub fn write(&self, file_handle: impl IO) -> Result<()> {
        let file_path = file_handle.path();
        let content = serialise(self, ron::ser::PrettyConfig::default())
            .with_context(|| format!("Unable to serialise State to {}", file_path))?;
        file_handle
            .write(content)
            .with_context(|| format!("Unable to write State to {}", file_path))
    }

    pub fn with_overriden_cards(self, cards: Vec<Card>) -> Self {
        Self {
            cards: Self::override_matching_values(self.cards, cards),
            ..self
        }
    }

    pub fn with_overriden_decks(self, decks: Vec<deck::Deck>) -> Self {
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

    pub fn deal(&self, deck_name: &str) -> Result<Hand<'_>, StateError> {
        let deck = self.get_deck(deck_name)?;
        Hand::from(deck, self.cards.values().collect()).map_err(|e| match e {
            hand::HandError::EmptyDeck { name } => StateError::EmptyDeck { name },
            hand::HandError::NoDueCards { name } => StateError::NoDueCards { name },
            _ => unreachable!(),
        })
    }

    pub fn get_deck(&self, deck_name: &str) -> Result<&Deck, StateError> {
        self.decks.get(deck_name).ok_or(StateError::MissingDeck {
            name: deck_name.to_owned(),
        })
    }

    fn from_file(file_handle: impl IO) -> Result<Self> {
        let file_path = file_handle.path();
        let content = file_handle
            .read()
            .with_context(|| format!("Unable to read State from {}", file_path))?;
        ron::from_str(&content).with_context(|| format!("Unable to parse State from {}", file_path))
    }

    fn with_merged_cards(self, cards: Vec<Card>) -> Self {
        Self {
            cards: Self::merge_matching_values(self.cards, cards),
            ..self
        }
    }

    fn with_merged_decks(self, decks: Vec<deck::Deck>) -> Self {
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
                Some(item) => i.merge(item),
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
pub mod mocks {

    use super::*;

    pub const ERROR_ID: &str = "ERROR";

    pub fn to_string_pretty(state: &State, config: ron::ser::PrettyConfig) -> Result<String> {
        custom_error! { FakeRonError
            Booboo {msg: String} = "Oops an error {msg}",
        }
        if state.card_parsing_config.deck_delimiter == ERROR_ID {
            Err(FakeRonError::Booboo {
                msg: ERROR_ID.to_string(),
            })?
        } else {
            ron::ser::to_string_pretty(state, ron::ser::PrettyConfig::default()).context("whatever")
        }
    }
}

#[cfg(test)]
pub mod assertions {

    use super::tools::test_tools::{assertions::assert_uid_map_contains, Expect};
    use super::*;

    pub fn assert_state_eq(
        actual_state: &State,
        expected_parsing_config: &ParsingConfig,
        expected_cards: Vec<Expect<Card>>,
        expected_decks: Vec<Expect<deck::Deck>>,
    ) {
        assert_eq!(*expected_parsing_config, actual_state.card_parsing_config);
        assert_uid_map_contains(&actual_state.cards, &expected_cards);
        assert_uid_map_contains(&actual_state.decks, &expected_decks);
    }
}

#[cfg(test)]
mod unit_tests {

    use super::card::fake::markdown_card_with_default_format as fake_markdown_card;
    use super::card::revision_settings::RevisionSettings;
    use super::deck::interval_coefficients::IntervalCoefficients;
    use super::hand::assertions::assert_hand_contains;
    use super::tools::test_tools::{Expect, MockIO};
    use super::*;
    use assert_fs::prelude::*;
    use chrono::{DateTime, Duration, Utc};
    use itertools::Itertools;

    fn fake_parsing_config_with_delimiter(delimiter: &str) -> ParsingConfig {
        ParsingConfig {
            deck_delimiter: delimiter.to_string(),
            ..Default::default()
        }
    }

    fn fake_card_with_path_and_decks(path: &str, decks: Vec<&str>) -> Card {
        Card {
            path: path.to_string(),
            decks: decks.into_iter().map(|d| d.to_string()).collect(),
            ..Default::default()
        }
    }

    fn fake_card_with_path_decks_and_due_date(
        path: &str,
        decks: Vec<&str>,
        due: DateTime<Utc>,
    ) -> Card {
        let mut card = fake_card_with_path_and_decks(path, decks);
        card.revision_settings.due = due;
        card
    }

    fn fake_deck_with_name(name: &str) -> deck::Deck {
        deck::Deck {
            name: name.to_string(),
            ..Default::default()
        }
    }

    fn fake_deck_with_name_and_card_paths(name: &str, card_paths: &[&str]) -> deck::Deck {
        deck::Deck {
            name: name.to_string(),
            card_paths: card_paths.iter().map(ToString::to_string).collect_vec(),
            ..Default::default()
        }
    }

    fn fake_state_with_single_card_and_deck() -> (ParsingConfig, Card, deck::Deck, State) {
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
        let (card_parsing_config, card, deck, expected) = fake_state_with_single_card_and_deck();
        let cards = vec![card.clone()];
        let decks = vec![deck.clone()];
        let actual = State::new(card_parsing_config, cards, decks);

        assert_eq!(expected, actual);
    }

    #[test]
    fn with_overriden_cards_when_new_card_has_different_path_from_old_card() {
        let (parsing_config, old_card, deck, state) = fake_state_with_single_card_and_deck();
        let new_card = fake_card_with_path_and_decks("some/other/path", vec!["another_deck"]);
        let actual = state.with_overriden_cards(vec![new_card.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(old_card), Expect::DoesContain(new_card)],
            vec![Expect::DoesContain(deck)],
        );
    }

    #[test]
    fn with_overriden_cards_when_new_card_has_same_path_as_old_card() {
        let (parsing_config, old_card, deck, state) = fake_state_with_single_card_and_deck();
        let new_card = fake_card_with_path_and_decks(&old_card.path[..], vec!["another_deck"]);
        let actual = state.with_overriden_cards(vec![new_card.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![
                Expect::DoesNotContain(old_card),
                Expect::DoesContain(new_card),
            ],
            vec![Expect::DoesContain(deck)],
        );
    }

    #[test]
    fn with_merged_cards_when_new_card_has_different_path_from_old_card() {
        let (parsing_config, old_card, deck, state) = fake_state_with_single_card_and_deck();
        let new_card = fake_card_with_path_and_decks("some/other/path", vec!["another_deck"]);
        let actual = state.with_merged_cards(vec![new_card.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(old_card), Expect::DoesContain(new_card)],
            vec![Expect::DoesContain(deck)],
        );
    }

    #[test]
    fn with_merged_cards_when_new_card_has_same_path_as_old_card() {
        let (parsing_config, old_card, deck, state) = fake_state_with_single_card_and_deck();
        let mut expected_card = fake_card_with_path_and_decks(old_card.uid(), vec!["another_deck"]);
        expected_card.revision_settings = old_card.revision_settings.clone();
        let mut new_card = expected_card.clone();
        new_card.revision_settings = RevisionSettings::new(Utc::now(), 9000.0, 1234567.5);
        let actual = state.with_merged_cards(vec![new_card.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![
                Expect::DoesNotContain(old_card),
                Expect::DoesNotContain(new_card),
                Expect::DoesContain(expected_card),
            ],
            vec![Expect::DoesContain(deck)],
        );
    }

    #[test]
    fn with_overriden_decks_when_new_deck_has_different_name_from_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_state_with_single_card_and_deck();
        let new_deck = fake_deck_with_name("a_new_deck_appears");
        let actual = state.with_overriden_decks(vec![new_deck.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(card)],
            vec![Expect::DoesContain(old_deck), Expect::DoesContain(new_deck)],
        );
    }

    #[test]
    fn with_overriden_decks_when_new_deck_has_same_name_as_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_state_with_single_card_and_deck();
        let mut new_deck = fake_deck_with_name(&old_deck.name[..]);
        new_deck.interval_coefficients.easy_coef = 9000.0;
        let actual = state.with_overriden_decks(vec![new_deck.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(card)],
            vec![
                Expect::DoesNotContain(old_deck),
                Expect::DoesContain(new_deck),
            ],
        );
    }

    #[test]
    fn with_merged_decks_when_new_deck_has_different_name_from_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_state_with_single_card_and_deck();
        let new_deck = fake_deck_with_name("a_new_deck_appears");
        let actual = state.with_merged_decks(vec![new_deck.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(card)],
            vec![Expect::DoesContain(old_deck), Expect::DoesContain(new_deck)],
        );
    }

    #[test]
    fn with_merged_decks_when_new_deck_has_same_name_as_old_deck() {
        let (parsing_config, card, old_deck, state) = fake_state_with_single_card_and_deck();
        let mut expected_deck = old_deck.clone();
        expected_deck.card_paths = vec!["a/new/path".to_string(), "another/new/path".to_string()];
        let mut new_deck = expected_deck.clone();
        new_deck.interval_coefficients = IntervalCoefficients::new(31.0, 32.0, 33.0);
        let actual = state.with_merged_decks(vec![new_deck.clone()]);
        assertions::assert_state_eq(
            &actual,
            &parsing_config,
            vec![Expect::DoesContain(card)],
            vec![
                Expect::DoesNotContain(old_deck),
                Expect::DoesNotContain(new_deck),
                Expect::DoesContain(expected_deck),
            ],
        );
    }

    #[test]
    fn with_card_parsing_config() {
        let (_, card, deck, state) = fake_state_with_single_card_and_deck();
        let new_parsing_config = ParsingConfig {
            deck_delimiter: "?".to_string(),
            ..Default::default()
        };
        let actual = state.with_card_parsing_config(new_parsing_config.clone());
        assertions::assert_state_eq(
            &actual,
            &new_parsing_config,
            vec![Expect::DoesContain(card)],
            vec![Expect::DoesContain(deck)],
        );
    }

    #[test]
    fn deal_when_deck_does_not_exist() {
        let state = State::default();
        let deck_name = "Does not exist";
        let actual = state.deal(deck_name);
        assert!(actual.is_err());
        assert!(format!("{:#?}", actual.unwrap_err()).contains(deck_name));
    }

    #[test]
    fn get_deck_when_deck_does_not_exist() {
        let state = State::default();
        let deck_name = "Does not exist";
        let actual = state.get_deck(deck_name);
        assert!(actual.is_err());
        assert!(format!("{:#?}", actual.unwrap_err()).contains(deck_name));
    }

    #[test]
    fn get_deck() {
        let (deck_name_a, deck_name_b) = ("a", "b");
        let card_parsing_config = ParsingConfig::default();
        let past = Utc::now() - Duration::days(10);
        let future = Utc::now() + Duration::days(10);
        let deck_a_due_card =
            fake_card_with_path_decks_and_due_date("a/some", vec![deck_name_a], past);
        let deck_a_other_card =
            fake_card_with_path_decks_and_due_date("a/other", vec![deck_name_a], future);
        let deck_b_due_card =
            fake_card_with_path_decks_and_due_date("b/some", vec![deck_name_b], past);
        let deck_b_other_card =
            fake_card_with_path_decks_and_due_date("b/other", vec![deck_name_b], future);
        let (deck_a, deck_b) = (
            fake_deck_with_name(deck_name_a),
            fake_deck_with_name(deck_name_b),
        );
        let state = State {
            card_parsing_config: card_parsing_config.clone(),
            cards: HashMap::from([
                (deck_a_due_card.path.clone(), deck_a_due_card.clone()),
                (deck_a_other_card.path.clone(), deck_a_other_card.clone()),
                (deck_b_due_card.path.clone(), deck_b_due_card.clone()),
                (deck_b_other_card.path.clone(), deck_b_other_card.clone()),
            ]),
            decks: HashMap::from([
                (deck_a.name.clone(), deck_a.clone()),
                (deck_b.name.clone(), deck_b.clone()),
            ]),
        };
        let actual = state.get_deck(deck_name_b).unwrap();
        assert_eq!(&deck_b, actual);
    }

    #[test]
    fn deal() {
        let (deck_name_a, deck_name_b) = ("a", "b");
        let card_parsing_config = ParsingConfig::default();
        let past = Utc::now() - Duration::days(10);
        let future = Utc::now() + Duration::days(10);
        let deck_a_due_card =
            fake_card_with_path_decks_and_due_date("a/some", vec![deck_name_a], past);
        let deck_a_other_card =
            fake_card_with_path_decks_and_due_date("a/other", vec![deck_name_a], future);
        let deck_b_due_card =
            fake_card_with_path_decks_and_due_date("b/some", vec![deck_name_b], past);
        let deck_b_other_card =
            fake_card_with_path_decks_and_due_date("b/other", vec![deck_name_b], future);
        let (deck_a, deck_b) = (
            fake_deck_with_name(deck_name_a),
            fake_deck_with_name(deck_name_b),
        );
        let state = State {
            card_parsing_config: card_parsing_config.clone(),
            cards: HashMap::from([
                (deck_a_due_card.path.clone(), deck_a_due_card.clone()),
                (deck_a_other_card.path.clone(), deck_a_other_card.clone()),
                (deck_b_due_card.path.clone(), deck_b_due_card.clone()),
                (deck_b_other_card.path.clone(), deck_b_other_card.clone()),
            ]),
            decks: HashMap::from([
                (deck_a.name.clone(), deck_a.clone()),
                (deck_b.name.clone(), deck_b.clone()),
            ]),
        };
        let expected_queued_items = vec![Expect::DoesContain(deck_b_due_card)];
        let actual = state.deal(deck_name_b).unwrap();
        assert_hand_contains(
            &actual,
            &deck_b.interval_coefficients,
            &expected_queued_items,
        );
    }

    fn write_fake_file(s: &str, temp_dir: &assert_fs::TempDir, filename: &str) {
        temp_dir
            .child(filename)
            .write_str(s)
            .expect("Dump fake temp file.");
    }

    #[test]
    fn read() {
        let expected_due_date = Utc::now();
        let (path_a, path_b) = ("a_path.md", "b_path.md");
        let (deck_name_a, deck_name_b) = ("a", "b");
        let mut card_a = fake_card_with_path_and_decks(path_a, vec![deck_name_a]);
        let mut card_b = fake_card_with_path_and_decks(path_b, vec![deck_name_b]);
        let (question_a, question_b) = (card_a.question.clone(), card_b.question.clone());
        let (answer_a, answer_b) = (card_a.answer.clone(), card_b.answer.clone());
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let fake_notes_dirpath = temp_dir.path().to_path_buf();
        let state_str = ron::to_string(&State::default()).expect("Serialize State failed");
        let md_a = fake_markdown_card(&[deck_name_a], question_a.as_str(), answer_b.as_str());
        let md_b = fake_markdown_card(&[deck_name_b], question_b.as_str(), answer_b.as_str());
        write_fake_file(&state_str, &temp_dir, STATE_FILENAME);
        write_fake_file(&md_a, &temp_dir, path_a);
        write_fake_file(&md_b, &temp_dir, path_b);
        let expected_card_parsing_config = ParsingConfig::default();
        let path_a = String::from(fake_notes_dirpath.join(path_a).to_string_lossy());
        let path_b = String::from(fake_notes_dirpath.join(path_b).to_string_lossy());
        card_a.path = path_a.to_owned();
        card_b.path = path_b.to_owned();
        let exp_cards = vec![
            Expect::DoesContainNear(card_a),
            Expect::DoesContainNear(card_b),
        ];
        let (deck_a, deck_b) = (
            fake_deck_with_name_and_card_paths(deck_name_a, &[&path_a]),
            fake_deck_with_name_and_card_paths(deck_name_b, &[&path_b]),
        );
        let exp_decks = vec![Expect::DoesContain(deck_a), Expect::DoesContain(deck_b)];
        let actual = State::read(fake_notes_dirpath).unwrap();
        assertions::assert_state_eq(&actual, &expected_card_parsing_config, exp_cards, exp_decks);
    }

    #[test]
    fn from_file() {
        let expected_due_date = Utc::now();
        let expected_card_path = "a_card";
        let expected_deck_name = "a";
        let expected_card = fake_card_with_path_decks_and_due_date(
            expected_card_path,
            vec![expected_deck_name],
            expected_due_date,
        );
        let expected_deck = fake_deck_with_name(expected_deck_name);
        let expected_card_parsing_config = ParsingConfig::default();
        let expected_cards = vec![Expect::DoesContain(expected_card)];
        let expected_decks = vec![Expect::DoesContain(expected_deck)];
        let state_str = format!(
            "(card_parsing_config:(decks_pattern:TaggedLine(tag:\"tags:\"),deck_delimiter:\":\",question_pattern:WrappedMultiLine(opening_tag:\"# Question\",closing_tag:\"# Answer\"),answer_pattern:WrappedMultiLine(opening_tag:\"# Answer\",closing_tag:\"----\n\")),cards:{{\"{}\":(path:\"{}\",decks:[\"{}\"],question:\"\",answer:\"\",revision_settings:(due:\"{}\",interval:0.0,memorisation_factor:1300.0)),}},decks:{{\"{}\":(name:\"{}\",card_paths:[],interval_coefficients:(pass_coef:1.0,easy_coef:1.3,fail_coef:0.0))}})",
            expected_card_path,
            expected_card_path,
            expected_deck_name,
            expected_due_date,
            expected_deck_name,
            expected_deck_name,
        );
        let mut mock_file_handle = MockIO::new();
        mock_file_handle
            .expect_read()
            .returning(move || Ok(state_str.clone()));
        mock_file_handle
            .expect_path()
            .return_const("some_path".to_string());
        mock_file_handle.expect_write().never();
        let actual = State::from_file(mock_file_handle).unwrap();
        assertions::assert_state_eq(
            &actual,
            &expected_card_parsing_config,
            expected_cards,
            expected_decks,
        );
    }

    #[test]
    fn from_file_when_file_handle_read_fails() {
        let state_str = "oh dear";
        let mut mock_file_handle = MockIO::new();
        mock_file_handle
            .expect_read()
            .returning(move || Err(std::io::Error::from(std::io::ErrorKind::NotFound)));
        mock_file_handle
            .expect_path()
            .return_const(state_str.to_string());
        let actual = State::from_file(mock_file_handle);
        assert!(actual.is_err());
        assert!(actual
            .unwrap_err()
            .to_string()
            .contains(&format!("Unable to read State from {}", state_str)));
    }

    #[test]
    fn from_file_when_ron_fails() {
        let state_str = "G.a|R,B$4:g'3";
        let state_path = state_str;
        let state_content = state_str.to_string();
        let mut mock_file_handle = MockIO::new();
        mock_file_handle
            .expect_read()
            .returning(move || Ok(state_content.clone()));
        mock_file_handle
            .expect_path()
            .return_const(state_path.to_string());
        let actual = State::from_file(mock_file_handle);
        assert!(actual.is_err());
        assert!(actual
            .unwrap_err()
            .to_string()
            .contains(&format!("Unable to parse State from {}", state_str)));
    }

    #[test]
    fn write() {
        let due_date = Utc::now();
        let card_path = "a_card";
        let deck_name = "a";
        let card = fake_card_with_path_decks_and_due_date(card_path, vec![deck_name], due_date);
        let deck = fake_deck_with_name(deck_name);
        let card_parsing_config = ParsingConfig::default();
        let state = State::new(card_parsing_config, vec![card], vec![deck]);
        let expected =
            ron::ser::to_string_pretty(&state, ron::ser::PrettyConfig::default()).unwrap();
        let mut mock_file_handle = MockIO::new();
        mock_file_handle.expect_read().never();
        mock_file_handle.expect_path().return_const("".to_string());
        mock_file_handle
            .expect_write()
            .with(mockall::predicate::eq(expected))
            .returning(move |_| Ok(()));
        assert!(state.write(mock_file_handle).is_ok());
    }

    #[test]
    fn write_when_file_handle_write_fails() {
        let due_date = Utc::now();
        let card_path = "a_card";
        let deck_name = "a";
        let state_path = "stateful";
        let card = fake_card_with_path_decks_and_due_date(card_path, vec![deck_name], due_date);
        let deck = fake_deck_with_name(deck_name);
        let card_parsing_config = ParsingConfig::default();
        let state = State::new(card_parsing_config, vec![card], vec![deck]);
        let mut mock_file_handle = MockIO::new();
        mock_file_handle.expect_read().never();
        mock_file_handle
            .expect_write()
            .returning(move |_| Err(std::io::Error::from(std::io::ErrorKind::NotFound)));
        mock_file_handle
            .expect_path()
            .return_const(state_path.to_string());
        let actual = state.write(mock_file_handle);
        assert!(actual.is_err());
        assert!(actual
            .unwrap_err()
            .to_string()
            .contains(&format!("Unable to write State to {}", state_path)));
    }

    #[test]
    fn write_when_ron_fails() {
        let state_path = "stateful";
        let card_parsing_config = ParsingConfig {
            deck_delimiter: mocks::ERROR_ID.to_string(),
            ..Default::default()
        };
        let state = State::new(card_parsing_config, vec![], vec![]);
        let mut mock_file_handle = MockIO::new();
        mock_file_handle.expect_read().never();
        mock_file_handle.expect_write().never();
        mock_file_handle
            .expect_path()
            .return_const(state_path.to_string());
        let actual = state.write(mock_file_handle);
        assert!(actual.is_err());
        assert!(actual
            .unwrap_err()
            .to_string()
            .contains(&format!("Unable to serialise State to {}", state_path)));
    }
}
