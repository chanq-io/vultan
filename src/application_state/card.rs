mod revision_settings;
mod score;
mod text_file;

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
    fn clone_with_revision_settings(&self, revision_settings: RevisionSettings) -> Self {
        Card {
            revision_settings,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod card_tests {
    use chrono::Utc;
    use super::*;

    #[test]
    fn clone_with_revision_settings() {
        fn make_revision_settings(interval: f64, memorisation_factor: f64) -> RevisionSettings {
            RevisionSettings {
                due: Utc::now(),
                interval,
                memorisation_factor,
            }
        }
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
