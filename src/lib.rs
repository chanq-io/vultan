#![allow(dead_code)] // TODO remove
#![allow(unused_variables)] // TODO remove
pub mod state;
/*
use glob::glob;
use state::{card::parser::Parser, State};
use state::{file::FileHandle, tools::IO};

const STATE_FILE_NAME: &str = ".vultan.ron";

fn load_state(file_handle: impl IO) -> State {
    let state = State::read(file_handle).unwrap_or(State::default());
    let parser = Parser::from(&state.card_parsing_config);
    todo!()
}
// fn load_state(...)
// fn run(...)
// Repl::run
//

fn file_handle_from(path: String) -> FileHandle {
    FileHandle::from(path)
}

#[cfg(test)]
mod mocks {
    use super::*;
    use state::tools::test_tools::MockIO;
    pub fn mock_filesystem_reader(path: String) -> MockIO {
        let mut handle = MockIO::new();
        let path = path.to_string();
        handle.expect_path().return_const(path.clone());
        handle
            .expect_read()
            .returning(move || std::fs::read_to_string(path.clone()));
        handle
    }

    pub fn mock_filesystem_writer(path: String) -> MockIO {
        let mut handle = MockIO::new();
        handle.expect_path().return_const(path.to_string());
        let path = path.to_string();
        handle.expect_write().returning(move |content: String| {
            std::fs::write(path.clone(), content.as_str())
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, ""))
        });
        handle
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;
    use assert_fs::prelude::*;
    use state::{
        assertions,
        card::{parser::ParsingConfig, revision_settings::RevisionSettings, Card},
        deck::Deck,
        tools::test_tools::Expect,
        State,
    };

    fn setup_filesystem(
        fake_state: &State,
        fake_card_paths_and_markdown: Vec<(&str, String)>,
    ) -> assert_fs::TempDir {
        let temp = assert_fs::TempDir::new().unwrap();
        let temp_path = temp.path().to_str().expect("Bad Test");
        let state_file_path = format!("{}/{}", temp_path, STATE_FILE_NAME);
        let state_file_handle = mocks::mock_filesystem_writer(state_file_path);
        assert!(fake_state.write(state_file_handle).is_ok());
        for (path, markdown) in fake_card_paths_and_markdown {
            temp.child(path)
                .write_str(markdown.as_str())
                .expect("Bad Test");
        }
        temp.child("not_card.md")
            .write_str("#Some\nmarkdown")
            .expect("Bad test");
        temp
    }

    fn fake_card_markdown(deck_names: &[&str], question: &str, answer: &str) -> String {
        format!(
            "---\nk1: v1\ntags: :{}:\n---\n# Question\n{}# Answer\n{}\n----\n",
            deck_names.join(":"),
            question,
            answer
        )
    }

    fn fake_deck_with_name(name: &str) -> Deck {
        let mut deck = Deck::default();
        deck.name = name.to_string();
        deck
    }

    fn fake_card(path: &str, decks: Vec<&str>, question: &str, answer: &str) -> Card {
        Card::new(
            path.to_string(),
            decks.iter().map(|d| d.to_string()).collect(),
            question.to_string(),
            answer.to_string(),
            RevisionSettings::default(),
        )
    }

    #[test]
    fn can_load_state() {
        let (deck_name_a, deck_name_b, deck_name_c) = ("a", "b", "c");
        let (question_a, question_b) = ("what?", "who?");
        let (path_a, path_b) = ("card_a.md", "card_b.md");
        let (answer_a, answer_b) = ("this", "that");
        let (card_decks_a, card_decks_b) = (
            vec![deck_name_a, deck_name_b],
            vec![deck_name_a, deck_name_c],
        );
        let (card_markdown_a, card_markdown_b) = (
            fake_card_markdown(&card_decks_a, question_a, answer_a),
            fake_card_markdown(&card_decks_b, question_b, answer_b),
        );
        let card_a = fake_card(path_a, card_decks_a, question_a, answer_a);
        let card_b = fake_card(path_b, card_decks_b, question_b, answer_b);
        let deck_a = fake_deck_with_name(deck_name_a);
        let deck_b = fake_deck_with_name(deck_name_b);
        let deck_c = fake_deck_with_name(deck_name_c);
        let initial_state = State::new(
            ParsingConfig::default(),
            vec![card_a.clone()],
            vec![deck_a.clone()],
        );
        let temp_dir = setup_filesystem(
            &initial_state,
            vec![(path_a, card_markdown_a), (path_b, card_markdown_b)],
        );
        let expected_parsing_config = ParsingConfig::default();
        let expected_cards = vec![Expect::DoesContain(card_a), Expect::DoesContain(card_b)];
        let expected_decks = vec![
            Expect::DoesContain(deck_a),
            Expect::DoesContain(deck_b),
            Expect::DoesContain(deck_c),
        ];
        let notes_dir = temp_dir.path().to_str().unwrap();
        let fake_state_path = format!("{}/{}", notes_dir, STATE_FILE_NAME);
        let actual = load_state(mocks::mock_filesystem_reader(fake_state_path));
        assertions::assert_state_eq(
            &actual,
            &expected_parsing_config,
            expected_cards,
            expected_decks,
        );
        temp_dir.close().expect("Failed to close test temp dir");
    }
}
*/
