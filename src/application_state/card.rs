mod parser;
mod revision_settings;
mod score;

use revision_settings::RevisionSettings;

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub id: String,
    pub tags: Vec<String>,
    pub question: String,
    pub answer: String,
    pub revision_settings: RevisionSettings,
}

impl Card {
    fn new(
        id: String,
        tags: Vec<String>,
        question: String,
        answer: String,
        revision_settings: RevisionSettings,
    ) -> Self {
        Self {
            id,
            tags,
            question,
            answer,
            revision_settings,
        }
    }

    fn clone_with_revision_settings(&self, revision_settings: RevisionSettings) -> Self {
        Card {
            revision_settings,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use chrono::Utc;

    fn make_revision_settings(interval: f64, memorisation_factor: f64) -> RevisionSettings {
        RevisionSettings {
            due: Utc::now(),
            interval,
            memorisation_factor,
        }
    }

    #[test]
    fn new() {
        let id = String::from("some-id");
        let tags = vec![String::from("some-tag")];
        let question = String::from("a question?");
        let answer = String::from("an answer.");
        let revision_settings = make_revision_settings(2.0, 3.0);
        let expected = Card {
            id: id.clone(),
            tags: tags.clone(),
            question: question.clone(),
            answer: answer.clone(),
            revision_settings: revision_settings.clone(),
        };
        let actual = Card::new(id, tags, question, answer, revision_settings);
        assert_eq!(expected, actual);
    }

    #[test]
    fn clone_with_revision_settings() {
        fn make_card_with_revision_settings(revision_settings: RevisionSettings) -> Card {
            Card {
                id: String::from("some-identifier"),
                tags: vec![String::from("tag_1"), String::from("tag_2")],
                question: String::from("What is the meaning of life, the universe, everything?"),
                answer: String::from("42"),
                revision_settings,
            }
        }

        let old_revision_settings = make_revision_settings(246.8, 135.5);
        let new_revision_settings = make_revision_settings(135.5, 246.8);
        let input = make_card_with_revision_settings(old_revision_settings);
        let expected = make_card_with_revision_settings(new_revision_settings.clone());
        let actual = input.clone_with_revision_settings(new_revision_settings);
        assert_eq!(expected, actual);
    }
}
