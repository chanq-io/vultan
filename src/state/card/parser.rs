use anyhow::{Context, Result};
use custom_error::custom_error;
use regex::Regex;
use serde::{Deserialize, Serialize};

custom_error! {
    #[derive(Clone)]
    pub ParsingError
    DeckParsingError{input: String} = "Malformed decks field in input = `{input}`",
    QuestionParsingError{input: String} = "Malformed question field in input = `{input}`",
    AnswerParsingError{input: String} = "Malformed answer field in input = `{input}`"
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParsingConfig {
    pub decks_pattern: ParsingPattern,
    pub deck_delimiter: String,
    pub question_pattern: ParsingPattern,
    pub answer_pattern: ParsingPattern,
}

impl Default for ParsingConfig {
    fn default() -> Self {
        Self {
            decks_pattern: ParsingPattern::TaggedLine {
                tag: "tags:".to_string(),
            },
            deck_delimiter: ":".to_string(),
            question_pattern: ParsingPattern::WrappedMultiLine {
                opening_tag: "# Question".to_string(),
                closing_tag: "# Answer".to_string(),
            },
            answer_pattern: ParsingPattern::WrappedMultiLine {
                opening_tag: "# Answer".to_string(),
                closing_tag: "----\n".to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ParsingPattern {
    WrappedMultiLine {
        opening_tag: String,
        closing_tag: String,
    },
    TaggedLine {
        tag: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedCardFields<'a> {
    pub decks: Vec<&'a str>,
    pub question: &'a str,
    pub answer: &'a str,
}

pub trait Parse {
    fn parse<'a>(&self, input: &'a str) -> Result<ParsedCardFields<'a>>;
}

#[derive(Debug)]
pub struct Parser {
    decks_expression: Regex,
    deck_delimiter: String,
    question_expression: Regex,
    answer_expression: Regex,
}

impl Parser {
    pub fn from(user_config: &ParsingConfig) -> Result<Self> {
        Ok(Self {
            deck_delimiter: user_config.deck_delimiter.clone(),
            decks_expression: Self::make_regex_expression(&user_config.decks_pattern, "decks")?,
            question_expression: Self::make_regex_expression(
                &user_config.question_pattern,
                "question",
            )?,
            answer_expression: Self::make_regex_expression(&user_config.answer_pattern, "answer")?,
        })
    }

    fn make_regex_expression(pattern: &ParsingPattern, pattern_id: &str) -> Result<Regex> {
        use ParsingPattern::*;
        let expr = match pattern {
            TaggedLine { tag } => format!(r"{}(.*)", tag),
            WrappedMultiLine {
                opening_tag,
                closing_tag,
            } => format!(r"{}((?s).*){}", opening_tag, closing_tag),
        };
        Regex::new(&expr).with_context(|| {
            format!(
                "Unable to construct parser. Supplied {} pattern is malformed: {:?}",
                pattern_id, pattern
            )
        })
    }

    fn parse_string<'a>(&self, expression: &Regex, input: &'a str) -> Option<&'a str> {
        Some(expression.captures(input)?.get(1)?.as_str().trim())
    }

    fn parse_decks<'a>(&self, input: &'a str) -> Option<Vec<&'a str>> {
        Some(
            self.parse_string(&self.decks_expression, input)?
                .split(&self.deck_delimiter)
                .filter(|tag| !tag.is_empty())
                .collect(),
        )
    }
}

impl Parse for Parser {
    fn parse<'a>(&self, input: &'a str) -> Result<ParsedCardFields<'a>> {
        Ok(ParsedCardFields {
            decks: self
                .parse_decks(input)
                .ok_or(ParsingError::DeckParsingError {
                    input: input.to_owned(),
                })
                .context("Could not match DECKS against pattern")?,
            question: self
                .parse_string(&self.question_expression, input)
                .ok_or(ParsingError::DeckParsingError {
                    input: input.to_owned(),
                })
                .context("Could not match QUESTION against pattern")?,
            answer: self
                .parse_string(&self.answer_expression, input)
                .ok_or(ParsingError::DeckParsingError {
                    input: input.to_owned(),
                })
                .context("Could not match ANSWER against pattern")?,
        })
    }
}

#[cfg(test)]
use mockall::*;

#[cfg(test)]
mock! {
    pub Parser{}
    impl Parse for Parser {
        fn parse(&self, input: &str) -> Result<ParsedCardFields<'static>>;
    }
}

#[cfg(test)]
mod unit_tests {

    use super::*;

    mod parsing_config {

        use super::*;

        #[test]
        fn default() {
            let expected_decks_pattern = ParsingPattern::TaggedLine {
                tag: String::from(r"tags:"),
            };
            let expected_tag_delimiter = String::from(":");
            let expected_question_pattern = ParsingPattern::WrappedMultiLine {
                opening_tag: String::from(r"# Question"),
                closing_tag: String::from(r"# Answer"),
            };
            let expected_answer_pattern = ParsingPattern::WrappedMultiLine {
                opening_tag: String::from(r"# Answer"),
                closing_tag: String::from("----\n"),
            };
            let actual = ParsingConfig::default();
            assert_eq!(expected_decks_pattern, actual.decks_pattern);
            assert_eq!(expected_tag_delimiter, actual.deck_delimiter);
            assert_eq!(expected_question_pattern, actual.question_pattern);
            assert_eq!(expected_answer_pattern, actual.answer_pattern);
        }
    }

    mod parser {

        use super::*;
        use rstest::*;

        fn fake_parsing_config(
            decks_pattern: ParsingPattern,
            deck_delimiter: String,
            question_pattern: ParsingPattern,
            answer_pattern: ParsingPattern,
        ) -> ParsingConfig {
            ParsingConfig {
                decks_pattern,
                deck_delimiter,
                question_pattern,
                answer_pattern,
            }
        }

        fn fake_tagged_line_parsing_pattern(tag: &str) -> ParsingPattern {
            ParsingPattern::TaggedLine {
                tag: tag.to_string(),
            }
        }

        fn fake_wrapped_multi_line_parsing_pattern(
            opening_tag: &str,
            closing_tag: &str,
        ) -> ParsingPattern {
            ParsingPattern::WrappedMultiLine {
                opening_tag: opening_tag.to_string(),
                closing_tag: closing_tag.to_string(),
            }
        }

        fn fake_custom_user_config() -> ParsingConfig {
            fake_parsing_config(
                fake_wrapped_multi_line_parsing_pattern("Decks:", "Question:"),
                "\n - ".to_string(),
                fake_tagged_line_parsing_pattern("Question:"),
                fake_tagged_line_parsing_pattern("Answer:"),
            )
        }

        fn make_fake_config(field: &str, value: &str) -> ParsingConfig {
            let mut user_config = ParsingConfig::default();
            match field.to_lowercase().as_str() {
                "decks" => {
                    user_config.decks_pattern = fake_tagged_line_parsing_pattern(value);
                }
                "question" => {
                    user_config.decks_pattern = fake_tagged_line_parsing_pattern(value);
                }
                "answer" => {
                    user_config.decks_pattern = fake_tagged_line_parsing_pattern(value);
                }
                _ => panic!("BAD TEST"),
            };
            user_config
        }

        #[rstest]
        #[case::default(
            ParsingConfig::default(),
            Ok((r"tags:(.*)", r"# Question((?s).*)# Answer", "# Answer((?s).*)----\n"))
        )]
        #[case::fails_for_malformed_decks_pattern(
            make_fake_config("decks", "(("),
            Err("Couldn't make Parser for ParsingConfig")
        )]
        #[case::fails_for_malformed_question_pattern(
            make_fake_config("question", "(("),
            Err("Couldn't make Parser for ParsingConfig")
        )]
        #[case::fails_for_malformed_answer_pattern(
            make_fake_config("answer", "(("),
            Err("Couldn't make Parser for ParsingConfig")
        )]
        fn from(#[case] config: ParsingConfig, #[case] expected: Result<(&str, &str, &str), &str>) {
            let expected_delimiter = config.deck_delimiter.to_string();
            let actual = Parser::from(&config);
            match expected {
                Ok((expected_decks, expected_question, expected_answer)) => {
                    let actual = actual.unwrap();
                    assert_eq!(expected_decks, actual.decks_expression.as_str());
                    assert_eq!(expected_delimiter, actual.deck_delimiter);
                    assert_eq!(expected_question, actual.question_expression.as_str());
                    assert_eq!(expected_answer, actual.answer_expression.as_str());
                }
                Err(_) => {
                    assert!(actual.is_err());
                    assert!(format!("{:#?}", actual.unwrap_err())
                        .contains("Unable to construct parser"));
                }
            }
        }

        #[rstest]
        #[case::with_default_config(
            ParsingConfig::default(),
            "---\nk1: v1\ntags: :a:b:c:\n---\n# Question\nwho\ndis?\n# Answer\nme\n\n----\n",
            Ok((vec!["a","b","c"], "who\ndis?", "me"))
        )]
        #[case::with_multi_line_decks_single_line_question_single_line_answer(
            fake_custom_user_config(),
            "some noise\nDecks:\n a\n - b\n - c\nQuestion: what?\nAnswer: thing\nsome noise",
            Ok((vec!["a","b","c"], "what?", "thing"))
        )]
        #[case::with_decks_expression_that_have_no_captures(
            ParsingConfig::default(),
            "---\nk1: v1\n---\n# Question\nwhat?\n# Answer \nthing\n\n----\nBacklink: SOMELINK\n",
            Err("Could not match DECKS against pattern")
        )]
        #[case::with_question_expression_that_have_no_captures(
            ParsingConfig::default(),
            "---\nk1: v1\ntags: :a:\n---\n# A Q\nwhat?\n# Answer \nthing\n\n----\n",
            Err("Could not match QUESTION against pattern")
        )]
        #[case::with_answer_expression_that_have_no_captures(
            ParsingConfig::default(),
            "---\ntags: :a:\n---\n# Question\nwho?\n# Answer \ntme\n\n--_--\n",
            Err("Could not match ANSWER against pattern")
        )]
        fn parse(
            #[case] user_config: ParsingConfig,
            #[case] input: &str,
            #[case] expected: Result<(Vec<&str>, &str, &str), &str>,
        ) {
            let parser = Parser::from(&user_config).unwrap();
            let actual = parser.parse(input);
            match expected {
                Ok((expected_decks, expected_question, expected_answer)) => {
                    let actual = actual.unwrap();
                    assert_eq!(expected_decks, actual.decks);
                    assert_eq!(expected_question, actual.question);
                    assert_eq!(expected_answer, actual.answer);
                }
                Err(expected_message) => {
                    assert!(format!("{:#?}", actual.unwrap_err()).contains(expected_message));
                }
            }
        }
    }
}
