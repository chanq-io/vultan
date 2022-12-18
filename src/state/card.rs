pub mod parser; // TODO only ParsingConfig & ParsingPattern should be exposed publically
pub mod revision_settings; // Shouldn't need to be exposed publically
pub mod score;

use super::deck::IntervalCoefficients;
use super::file::FileHandle;
use super::tools::IO;
use super::tools::{Merge, UID};
use anyhow;
use anyhow::Context;
use chrono::Utc;
use custom_error::custom_error;
use glob::glob;
use parser::Parse;
pub use revision_settings::RevisionSettings; // Shouldn't need to be exposed publically
pub use score::Score;
use serde::{Deserialize, Serialize};

custom_error! {
    #[derive(Clone)]
    CardError
    GlobError { path: String } = "Unable to construct glob for path = `{path}`",
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
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

    pub fn from(file_handle: impl IO, parser: &impl Parse) -> anyhow::Result<Self> {
        let file_path = file_handle.path();
        let file_content = file_handle
            .read()
            .with_context(|| format!("Unable to parse card at '{file_path}'"))?;
        let parsed_fields = parser.parse(&file_content)?;
        Ok(Self {
            path: file_path.to_string(),
            decks: parsed_fields.decks.iter().map(|s| s.to_string()).collect(),
            question: parsed_fields.question.to_string(),
            answer: parsed_fields.answer.to_string(),
            revision_settings: RevisionSettings::default(),
        })
    }

    pub fn transform(self, score: Score, interval_coefficients: &IntervalCoefficients) -> Self {
        let revision_settings = self
            .revision_settings
            .clone()
            .transform(score, interval_coefficients);
        self.with_revision_settings(revision_settings)
    }

    pub fn with_revision_settings(self, revision_settings: RevisionSettings) -> Self {
        Self {
            revision_settings,
            ..self
        }
    }

    pub fn is_due(&self) -> bool {
        Utc::now() >= self.revision_settings.due
    }

    pub fn in_deck(&self, deck_id: &str) -> bool {
        self.decks.iter().any(|d| d == deck_id)
    }
}

impl UID for Card {
    fn uid(&self) -> &str {
        &self.path[..]
    }
}

impl Merge<Card> for Card {
    fn merge(self, other: &Card) -> Self {
        self.with_revision_settings(other.revision_settings.clone())
    }
}

#[derive(Default)]
pub struct LoadedCards {
    succeeded: Vec<Card>,
    failed: Vec<String>,
}

fn make_glob_pattern(notes_dir: std::path::PathBuf) -> anyhow::Result<String> {
    Ok(notes_dir
        .join("**/*.md")
        .to_str()
        .ok_or(CardError::GlobError {
            path: String::from(notes_dir.to_string_lossy()),
        })?
        .to_owned())
}

fn try_load_many(
    notes_dir: std::path::PathBuf,
    parser: &impl Parse,
) -> anyhow::Result<LoadedCards, String> {
    Err("".to_string())

    //let notes_dir_path = notes_dir
    //    .join("**/*.md")
    //    .to_str()
    //    .ok_or("TODO")
    //    .with_context(|_| "TODO".to_string());
    //let markdown_glob = glob(notes_dir_path)?.with_context(|| format!("TODO"))?;
    //Ok(
    //    markdown_glob.fold(LoadedCards::default(), |mut loaded_cards, maybe_file| {
    //        match maybe_file {
    //            Ok(path) => {
    //                let file_handle = FileHandle::from(path);
    //                match Card::from(file_handle, parser) {
    //                    Ok(card) => {
    //                        loaded_cards.succeeded.push(card);
    //                    }
    //                    Err(err_message) => {
    //                        loaded_cards.failed.push(err_message.to_string());
    //                    }
    //                }
    //            }
    //            Err(err_message) => {
    //                loaded_cards.failed.push(err_message.to_string());
    //            }
    //        }
    //        loaded_cards
    //    }),
    //)
    //    */
}

#[cfg(test)]
pub mod assertions {
    use super::*;
    use revision_settings::assertions::assert_revision_settings_near;

    pub fn assert_cards_near(a: &Card, b: &Card) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.decks, b.decks);
        assert_eq!(a.question, b.question);
        assert_eq!(a.answer, b.answer);
        assert_revision_settings_near(&a.revision_settings, &b.revision_settings, 2);
    }
}

#[cfg(test)]
mod unit_tests {

    use super::revision_settings::test_tools::make_expected_revision_settings;
    use super::*;
    use crate::state::tools::test_tools::{assert_truthy, Expect, MockIO};
    use assert_fs::prelude::*;
    use chrono::{Duration, Utc};
    use mockall::predicate::eq;
    use parser::MockParser;
    use parser::ParsedCardFields;
    use parser::ParsingError;
    use rstest::*;

    const FAKE_PATH: &str = "a_path";

    fn make_fake_card(
        path: &str,
        decks: Vec<&str>,
        question: &str,
        answer: &str,
        revision_settings: RevisionSettings,
    ) -> Card {
        Card::new(
            path.to_string(),
            decks.iter().map(|s| s.to_string()).collect(),
            question.to_string(),
            answer.to_string(),
            revision_settings,
        )
    }
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
        make_fake_card(
            path,
            parsed_fields.decks.to_owned(),
            parsed_fields.question,
            parsed_fields.answer,
            revision_settings,
        )
    }

    fn make_mock_parser(
        expected_filepath_arg: &'static str,
        expected_return_value: anyhow::Result<ParsedCardFields<'static>>,
    ) -> MockParser {
        let mut mock_parser = MockParser::new();
        mock_parser
            .expect_parse()
            .with(eq(expected_filepath_arg.clone()))
            .return_once(move |_| expected_return_value);
        mock_parser
    }

    #[fixture]
    fn successful_file_handle() -> MockIO {
        let mut mock_file_handle = MockIO::new();
        let content = FAKE_PATH.to_owned().to_string();
        mock_file_handle
            .expect_path()
            .return_const(FAKE_PATH.to_string());
        mock_file_handle
            .expect_read()
            .returning(move || Ok(content.clone()));
        mock_file_handle
    }

    #[fixture]
    fn failing_file_handle() -> MockIO {
        let mut mock_file_handle = MockIO::new();
        mock_file_handle
            .expect_read()
            .returning(move || Err(std::io::Error::from(std::io::ErrorKind::NotFound)));
        mock_file_handle
            .expect_path()
            .return_const(FAKE_PATH.to_string());
        mock_file_handle
    }

    fn fake_card_markdown(decks: &[&str], question: &str, answer: &str) -> String {
        format!(
            "---\ntitle:something\ntags: :{}:\n---\n# Question\n{}\n# Answer\n{}\n----\n",
            decks.join(":"),
            question,
            answer,
        )
    }

    fn write_fake_card_markdown_files(
        temp_dir: &assert_fs::TempDir,
        fake_card_paths_and_markdown: Vec<(&str, String)>,
    ) {
        for (path, markdown) in fake_card_paths_and_markdown {
            temp_dir
                .child(path)
                .write_str(markdown.as_str())
                .expect("Test setup: write_fake_card_files failed!");
        }
    }

    fn write_non_card_markdown_files(temp_dir: &assert_fs::TempDir, paths: Vec<&str>) {
        paths.iter().for_each(|p| {
            temp_dir
                .child(p)
                .write_str("#Some\nmarkdown")
                .expect("Setup test filesystem failed!");
        });
    }

    fn expected_parsed_fields<'a>(
        decks: &[&'a str],
        question: &'a str,
        answer: &'a str,
    ) -> parser::ParsedCardFields<'a> {
        parser::ParsedCardFields {
            question,
            answer,
            decks: decks.to_vec(),
        }
    }

    #[test]
    fn try_load_many_cards() {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let fake_notes_dir_path = temp_dir
            .path()
            .to_str()
            .expect("Setup test filesystem failed!");
        let (path_a, path_b) = ("a.md", "b.md");
        let (decks_a, decks_b) = (vec!["a", "b"], vec!["b", "c"]);
        let (question_a, question_b) = ("what?", "who?");
        let (answer_a, answer_b) = ("this", "that");
        let non_markdown_paths = vec![path_a, path_b];
        write_non_card_markdown_files(&temp_dir, non_markdown_paths.clone());
        write_fake_card_markdown_files(
            &temp_dir,
            vec![
                (path_a, fake_card_markdown(&decks_a, question_a, answer_a)),
                (path_b, fake_card_markdown(&decks_b, question_b, answer_b)),
            ],
        );
        let revision_settings = RevisionSettings::default();
        let parsed_fields_a = make_fake_parsed_fields(decks_a, question_a, question_b);
        let parsed_fields_b = make_fake_parsed_fields(decks_b, question_b, question_b);
        let expected_succeeded = vec![
            make_expected_card(path_a, &parsed_fields_a, revision_settings.clone()),
            make_expected_card(path_b, &parsed_fields_b, revision_settings),
        ];
        let parsing_config = parser::ParsingConfig::default();
        let parser = parser::Parser::from(&parsing_config).unwrap();
        let mut loaded_cards = try_load_many(fake_notes_dir_path.into(), &parser).unwrap();
        loaded_cards.failed.sort();
        loaded_cards.succeeded.sort_by(|a, b| a.path.cmp(&b.path));
        expected_succeeded
            .iter()
            .zip(loaded_cards.succeeded)
            .for_each(|(expected, actual)| assertions::assert_cards_near(&expected, &actual));

        non_markdown_paths
            .iter()
            .zip(loaded_cards.failed)
            .for_each(|(expected, actual)| assert_ne!(expected.to_string(), actual));
    }

    #[test]
    fn default() {
        let expected = Card {
            path: String::from(""),
            decks: Vec::new(),
            question: String::from(""),
            answer: String::from(""),
            revision_settings: RevisionSettings::default(),
        };
        let actual = Card::default();
        assertions::assert_cards_near(&expected, &actual);
    }

    #[rstest]
    fn from(successful_file_handle: MockIO) {
        let parsed_fields = make_fake_parsed_fields(vec!["tag"], "what?", "that");
        let mock_parser = make_mock_parser(FAKE_PATH, anyhow::Result::Ok(parsed_fields.clone()));
        let expected = make_expected_card(FAKE_PATH, &parsed_fields, RevisionSettings::default());
        let actual = Card::from(successful_file_handle, &mock_parser).unwrap();
        assertions::assert_cards_near(&expected, &actual);
    }

    #[rstest]
    fn from_where_parser_fails(successful_file_handle: MockIO) {
        let parser_error = anyhow::Result::Err(ParsingError::DeckParsingError {
            input: "FAIL".to_owned(),
        });
        let mock_parser = make_mock_parser(FAKE_PATH, parser_error.context("Whatever"));
        let actual = Card::from(successful_file_handle, &mock_parser);
        assert!(actual.is_err());
        assert!(format!("{:#?}", actual.unwrap_err()).contains("FAIL"));
    }

    #[rstest]
    fn from_where_file_read_fails(failing_file_handle: MockIO) {
        let parser_result: anyhow::Result<ParsedCardFields, anyhow::Error> =
            anyhow::Result::Ok(ParsedCardFields::default());
        let mock_parser = make_mock_parser(FAKE_PATH, parser_result);
        let expected_message = format!("Unable to parse card at '{}'", FAKE_PATH);
        let actual = Card::from(failing_file_handle, &mock_parser);
        assert!(actual.is_err());
        let actual_err = actual.unwrap_err();
        assert!(actual_err.to_string().contains(&expected_message));
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
    fn with_revision_settings() {
        let card = Card::default();
        let revision_settings = RevisionSettings::new(Utc::now(), 9000.0, 10000.0);
        let mut expected = card.clone();
        expected.revision_settings = revision_settings.clone();
        assert_eq!(expected, card.with_revision_settings(revision_settings));
    }

    #[test]
    fn transform() {
        let score = Score::Easy;
        let in_due_date = Utc::now() - Duration::days(4);
        let in_factor = 2000.0;
        let in_interval = 1.0;
        let revision_settings = RevisionSettings::new(in_due_date, in_interval, in_factor);
        let path = String::from("p");
        let decks = vec![String::from("d")];
        let question = String::from("q");
        let answer = String::from("a");
        let input = Card::new(path, decks, question, answer, revision_settings.clone());
        let coefficients = IntervalCoefficients::new(1.0, 2.0, 0.0);
        let out_revision_settings = make_expected_revision_settings(&in_due_date, 20.0, 2150.0);
        let mut expected = input.clone();
        expected.revision_settings = out_revision_settings;
        let actual = input.transform(score, &coefficients);
        assert_eq!(expected, actual)
    }

    #[rstest]
    #[case::when_due_date_in_past(Utc::now() - Duration::days(100), Expect::Truthy)]
    #[case::when_due_date_in_present(Utc::now(), Expect::Truthy)]
    #[case::when_due_date_in_future(Utc::now() + Duration::days(100), Expect::Falsy)]
    fn is_due_when_due_date_in_past(
        #[case] due_date: chrono::DateTime<Utc>,
        #[case] expectation: Expect<i32>,
    ) {
        let mut revision_settings = RevisionSettings::default();
        revision_settings.due = due_date;
        let fields = make_fake_parsed_fields(vec!["deck"], "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, revision_settings);
        assert_truthy(expectation, card.is_due());
    }

    #[rstest]
    #[case::when_decks_contains_id(vec!["deck", "THIS"], "THIS", Expect::Truthy)]
    #[case::when_decks_do_not_contain_id(vec![], "THIS", Expect::Falsy)]
    fn in_deck(
        #[case] decks: Vec<&'static str>,
        #[case] input: &'static str,
        #[case] expectation: Expect<i32>,
    ) {
        let revision_settings = RevisionSettings::default();
        let fields = make_fake_parsed_fields(decks, "q?", "ans");
        let card = make_expected_card("some-identifier", &fields, RevisionSettings::default());
        assert_truthy(expectation, card.in_deck(input));
    }

    #[test]
    fn uid() {
        let path = "the/path";
        let q = "".to_string();
        let a = "".to_string();
        let card = Card::new(path.to_string(), vec![], q, a, RevisionSettings::default());
        assert_eq!(path, card.uid());
    }

    #[test]
    fn merge() {
        let question = "huh?".to_string();
        let answer = "don't worry".to_string();
        let revision_settings_a = RevisionSettings::default();
        let a = Card::new(
            "a".to_string(),
            vec![],
            question,
            answer,
            revision_settings_a,
        );
        let mut b = a.clone();
        b.path = "b".to_string();
        b.revision_settings = RevisionSettings::new(Utc::now(), 654.25, 9876.5);
        let mut expected = a.clone();
        expected.revision_settings = b.revision_settings.clone();
        assert_eq!(expected, a.merge(&b));
    }
}
