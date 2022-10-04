pub mod parser; // TODO only ParsingConfig & ParsingPattern should be exposed publically
pub mod revision_settings;// Shouldn't need to be exposed publically
pub mod score;

use chrono::Utc;
use parser::Parse;
pub use revision_settings::RevisionSettings;// Shouldn't need to be exposed publically
pub use score::Score;

#[cfg(test)]
use mocks::mock_read_to_string as read_file;
#[cfg(not(test))]
use std::fs::read_to_string as read_file;

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub path: String,
    pub decks: Vec<String>,
    pub question: String,
    pub answer: String,
    pub revision_settings: RevisionSettings,
}

impl Card {
    pub fn new(
        path: String,
        decks: Vec<String>,
        question: String,
        answer: String,
        revision_settings: RevisionSettings,
    ) -> Self {
        Self {
            path,
            decks,
            question,
            answer,
            revision_settings,
        }
    }

    pub fn from(filepath: &str, parser: &impl Parse) -> Result<Self, String> {
        let error_message = format!("Unable to read card at filepath({})", filepath);
        let error_formatter = |e| format!("{} -> {}", error_message, e);
        let file_content = read_file(filepath).map_err(|e| error_formatter(e.to_string()))?;
        let parsed_fields = parser.parse(&file_content).map_err(error_formatter)?;
        Ok(Self {
            path: filepath.to_string(),
            decks: parsed_fields.decks.iter().map(|s| s.to_string()).collect(),
            question: parsed_fields.question.to_string(),
            answer: parsed_fields.answer.to_string(),
            revision_settings: RevisionSettings::default(),
        })
    }

    pub fn is_due(&self) -> bool {
        Utc::now() >= self.revision_settings.due
    }

    pub fn in_deck(&self, deck_id: &str) -> bool {
        self.decks.iter().any(|d| d == deck_id)
    }

    fn move_with_new_revision_settings(self, revision_settings: RevisionSettings) -> Self {
        Card {
            revision_settings,
            ..self
        }
    }
}

impl Default for Card {
    fn default() -> Card {
        Card::new(
            String::from(""),
            Vec::new(),
            String::from(""),
            String::from(""),
            RevisionSettings::default()
        )
    }
}

#[cfg(test)]
mod mocks {
    pub const ERRONEOUS_PATH: &str = "error this path is garbage";
    pub fn mock_read_to_string(path: &str) -> Result<String, String> {
        if path == ERRONEOUS_PATH {
            Err(String::from(path))
        } else {
            Ok(String::from(path))
        }
    }
}
#[cfg(test)]
pub mod assertions {
    use super::*;
    use revision_settings::assertions::assert_near as assert_revision_settings_near;

    pub fn assert_near(a: &Card, b: &Card) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.decks, b.decks);
        assert_eq!(a.question, b.question);
        assert_eq!(a.answer, b.answer);
        assert_revision_settings_near(&a.revision_settings, &b.revision_settings, 2);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use chrono::Duration;
    use mockall::predicate::eq;
    use parser::MockParser;
    use parser::ParsedCardFields;

    fn make_fake_parsed_fields(
        decks: Vec<&'static str>,
        question: &'static str,
        answer: &'static str,
    ) -> ParsedCardFields<'static> {
        ParsedCardFields {
            decks,
            question,
            answer,
        }
    }

    fn make_fake_revision_settings(interval: f64, memorisation_factor: f64) -> RevisionSettings {
        RevisionSettings {
            due: Utc::now(),
            interval,
            memorisation_factor,
        }
    }

    fn make_expected_card(
        path: &str,
        parsed_fields: &ParsedCardFields,
        revision_settings: RevisionSettings,
    ) -> Card {
        Card {
            path: path.to_string(),
            decks: parsed_fields.decks.iter().map(|s| s.to_string()).collect(),
            question: String::from(parsed_fields.question),
            answer: String::from(parsed_fields.answer),
            revision_settings,
        }
    }

    fn make_mock_parser(
        expected_filepath_arg: &'static str,
        expected_return_value: Result<ParsedCardFields<'static>, String>,
    ) -> MockParser {
        let mut mock_parser = MockParser::new();
        mock_parser
            .expect_parse()
            .with(eq(expected_filepath_arg.clone()))
            .return_const(expected_return_value);
        mock_parser
    }

    #[test]
    fn default() {
        let expected = Card {
            path: String::from(""),
            decks: Vec::new(),
            question: String::from(""),
            answer: String::from(""),
            revision_settings: RevisionSettings::default()
        };
        let actual = Card::default();
        assertions::assert_near(&expected, &actual);
    }

    #[test]
    fn from() {
        let filepath = "hello";
        let parsed_fields = make_fake_parsed_fields(vec!["tag"], "what?", "that");
        let mock_parser = make_mock_parser(filepath, Result::Ok(parsed_fields.clone()));
        let expected = make_expected_card(filepath, &parsed_fields, RevisionSettings::default());
        let actual = Card::from(filepath, &mock_parser).unwrap();
        assertions::assert_near(&expected, &actual);
    }

    #[test]
    fn from_where_parser_fails() {
        let filepath = "hello";
        let parsed_fields = make_fake_parsed_fields(vec!["tag"], "what?", "that");
        let parser_error = Result::Err(filepath.to_string());
        let mock_parser = make_mock_parser(filepath, parser_error);
        let actual = Card::from(filepath, &mock_parser);
        assert!(actual.is_err());
        assert!(actual
            .unwrap_err()
            .contains("Unable to read card at filepath(hello)"));
    }

    #[test]
    fn from_where_file_read_fails() {
        let filepath = mocks::ERRONEOUS_PATH;
        let parsed_fields = make_fake_parsed_fields(vec!["tag"], "what?", "that");
        let mock_parser = make_mock_parser(filepath, Result::Ok(parsed_fields.clone()));
        let expected_message =
            format!("Unable to read card at filepath({})", mocks::ERRONEOUS_PATH);
        let actual = Card::from(filepath, &mock_parser);
        assert!(actual.is_err());
        assert!(actual.unwrap_err().contains(&expected_message));
    }

    #[test]
    fn new() {
        let path = String::from("some-path");
        let decks = vec![String::from("some-tag")];
        let question = String::from("a question?");
        let answer = String::from("an answer.");
        let revision_settings = make_fake_revision_settings(2.0, 3.0);
        let expected = Card {
            path: path.clone(),
            decks: decks.clone(),
            question: question.clone(),
            answer: answer.clone(),
            revision_settings: revision_settings.clone(),
        };
        let actual = Card::new(path, decks, question, answer, revision_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn move_with_new_revision_settings() {
        fn make_card_with_revision_settings(revision_settings: RevisionSettings) -> Card {
            let fields = make_fake_parsed_fields(
                vec!["tag_1", "tag_2"],
                "What is the meaning of life, the universe, everything?",
                "42"
            );
            make_expected_card("some-identifier", &fields, revision_settings)
        }

        let old_revision_settings = make_fake_revision_settings(246.8, 135.5);
        let new_revision_settings = make_fake_revision_settings(135.5, 246.8);
        let input = make_card_with_revision_settings(old_revision_settings);
        let expected = make_card_with_revision_settings(new_revision_settings.clone());
        let actual = input.move_with_new_revision_settings(new_revision_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn is_due_when_due_date_in_past() {
        // Note, testing the exact present would be painful so this is the best next thing
        let mut revision_settings = RevisionSettings::default();
        revision_settings.due = Utc::now();
        let fields = make_fake_parsed_fields(vec!["deck"], "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, revision_settings);
        assert!(card.is_due());
    }

    #[test]
    fn is_due_when_due_date_in_future() {
        let mut revision_settings = RevisionSettings::default();
        revision_settings.due = Utc::now() + Duration::days(100);
        let fields = make_fake_parsed_fields(vec!["deck"], "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, revision_settings);
        assert!(!card.is_due());
    }

    #[test]
    fn in_deck_when_decks_contains_id() {
        let revision_settings = RevisionSettings::default();
        let deck_id = "some_deck";
        let fields = make_fake_parsed_fields(vec!["deck", deck_id], "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, RevisionSettings::default());
        assert!(card.in_deck(deck_id));
    }

    #[test]
    fn in_deck_when_decks_do_not_contain_id() {
        let revision_settings = RevisionSettings::default();
        let fields = make_fake_parsed_fields(vec!["deck"], "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, RevisionSettings::default());
        assert!(!card.in_deck("no"));
    }
}
