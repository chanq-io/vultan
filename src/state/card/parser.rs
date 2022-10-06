use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
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
                tag: String::from(r"tags:"),
            },
            deck_delimiter: String::from(":"),
            question_pattern: ParsingPattern::WrappedMultiLine {
                opening_tag: String::from(r"# Question"),
                closing_tag: String::from(r"# Answer"),
            },
            answer_pattern: ParsingPattern::WrappedMultiLine {
                opening_tag: String::from(r"# Answer"),
                closing_tag: String::from(r"----\n"),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsingPattern {
    WrappedMultiLine {
        opening_tag: String,
        closing_tag: String,
    },
    TaggedLine {
        tag: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedCardFields<'a> {
    pub decks: Vec<&'a str>,
    pub question: &'a str,
    pub answer: &'a str,
}

pub trait Parse {
    fn parse<'a>(&self, input: &'a str) -> Result<ParsedCardFields<'a>, String>;
}

#[derive(Debug)]
pub struct Parser {
    decks_expression: Regex,
    deck_delimiter: String,
    question_expression: Regex,
    answer_expression: Regex,
}

impl Parser {
    pub fn from(user_config: ParsingConfig) -> Result<Self, String> {
        let partial_error = format!("Couldn't make Parser for {:?}", &user_config);
        Ok(Self {
            deck_delimiter: user_config.deck_delimiter,
            decks_expression: Self::make_regex(&user_config.decks_pattern, &partial_error)?,
            question_expression: Self::make_regex(&user_config.question_pattern, &partial_error)?,
            answer_expression: Self::make_regex(&user_config.answer_pattern, &partial_error)?,
        })
    }

    fn make_regex(pattern: &ParsingPattern, error_formatter: &str) -> Result<Regex, String> {
        let error_formatter = |e| format!("{} -> {}", error_formatter, e);
        Regex::new(&Self::make_regex_expression(&pattern)).map_err(error_formatter)
    }

    fn make_regex_expression(pattern: &ParsingPattern) -> String {
        use ParsingPattern::*;
        match pattern {
            TaggedLine { tag } => format!(r"{}(.*)", tag),
            WrappedMultiLine {
                opening_tag,
                closing_tag,
            } => format!(r"{}((?s).*){}", opening_tag, closing_tag),
        }
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

    fn error_if_none<T>(
        &self,
        parsed_field: Option<T>,
        field_id: &str,
        expression: &Regex,
    ) -> Result<T, String> {
        parsed_field.ok_or(format!(
            "Could not match {} against pattern(\"{}\")",
            field_id,
            expression.as_str()
        ))
    }
}

impl Parse for Parser {
    fn parse<'a>(&self, input: &'a str) -> Result<ParsedCardFields<'a>, String> {
        let maybe_decks = self.parse_decks(input);
        let maybe_question = self.parse_string(&self.question_expression, input);
        let maybe_answer = self.parse_string(&self.answer_expression, input);
        Ok(ParsedCardFields {
            decks: self.error_if_none(maybe_decks, "DECKS", &self.decks_expression)?,
            question: self.error_if_none(maybe_question, "QUESTION", &self.question_expression)?,
            answer: self.error_if_none(maybe_answer, "ANSWER", &self.answer_expression)?,
        })
    }
}

#[cfg(test)]
use mockall::*;

#[cfg(test)]
mock! {
    pub Parser{}
    impl Parse for Parser {
        fn parse(&self, input: &str) -> Result<ParsedCardFields<'static>, String>;
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
                closing_tag: String::from(r"----\n"),
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

        #[test]
        fn from() {
            let user_config = ParsingConfig::default();
            let expected_delimiter = user_config.deck_delimiter.to_string();
            let parser = Parser::from(user_config).unwrap();
            assert_eq!(r"tags:(.*)", parser.decks_expression.as_str());
            assert_eq!(expected_delimiter, parser.deck_delimiter);
            assert_eq!(
                r"# Question((?s).*)# Answer",
                parser.question_expression.as_str()
            );
            assert_eq!(r"# Answer((?s).*)----\n", parser.answer_expression.as_str());
        }

        #[test]
        fn from_fails_for_malformed_decks_pattern() {
            let mut user_config = ParsingConfig::default();
            user_config.decks_pattern = ParsingPattern::TaggedLine {
                tag: String::from(r"(("),
            };
            let error = Parser::from(user_config);
            assert!(error.is_err());
            assert!(error
                .unwrap_err()
                .contains("Couldn't make Parser for ParsingConfig"));
        }

        #[test]
        fn from_fails_for_malformed_question_pattern() {
            let mut user_config = ParsingConfig::default();
            user_config.question_pattern = ParsingPattern::TaggedLine {
                tag: String::from(r"(("),
            };
            let error = Parser::from(user_config);
            assert!(error.is_err());
            assert!(error
                .unwrap_err()
                .contains("Couldn't make Parser for ParsingConfig"));
        }

        #[test]
        fn from_fails_for_malformed_answer_pattern() {
            let mut user_config = ParsingConfig::default();
            user_config.answer_pattern = ParsingPattern::TaggedLine {
                tag: String::from(r"(("),
            };
            let error = Parser::from(user_config);
            assert!(error.is_err());
            assert!(error
                .unwrap_err()
                .contains("Couldn't make Parser for ParsingConfig"));
        }

        #[test]
        fn parse_with_default_config() {
            let user_config = ParsingConfig::default();
            let parser = Parser::from(user_config).unwrap();
            let expected_decks = vec!["a", "b", "c"];
            let expected_question =
                "What is the \n answer to life,\n the universe\nand everything?";
            let expected_answer = "42";
            let input = format!(
                "---\na_key: a_value\ntags: :{}:\n\
                 another_key: another_value\n---\n# Question\n\
                 {}\n# Answer \n{}\n\n----\nBacklink: SOMELINK\n",
                expected_decks.join(":"),
                expected_question,
                expected_answer
            );

            let actual = parser.parse(&input).unwrap();
            assert_eq!(expected_decks, actual.decks);
            assert_eq!(expected_question, actual.question);
            assert_eq!(expected_answer, actual.answer);
        }

        #[test]
        fn parse_with_multi_line_decks_single_line_question_single_line_answer() {
            let user_config = ParsingConfig {
                decks_pattern: ParsingPattern::WrappedMultiLine {
                    opening_tag: String::from(r"Decks:"),
                    closing_tag: String::from(r"Question:"),
                },
                deck_delimiter: String::from("\n - "),
                question_pattern: ParsingPattern::TaggedLine {
                    tag: String::from(r"Question:"),
                },
                answer_pattern: ParsingPattern::TaggedLine {
                    tag: String::from(r"Answer:"),
                },
            };
            let expected_delimiter = user_config.deck_delimiter.to_string();
            let expected_decks = vec!["a", "b", "c"];
            let expected_question = "What is the answer to life, the universe and everything?";
            let expected_answer = "42";
            let parser = Parser::from(user_config).unwrap();
            let input = format!(
                "some noise\nDecks: {}\nQuestion: {}\nAnswer: {}\nsome noise",
                expected_decks.join(&expected_delimiter),
                expected_question,
                expected_answer
            );
            let actual = parser.parse(&input).unwrap();
            assert_eq!(expected_decks, actual.decks);
            assert_eq!(expected_question, actual.question);
            assert_eq!(expected_answer, actual.answer);
        }

        #[test]
        fn parse_where_decks_expression_has_no_captures() {
            let user_config = ParsingConfig::default();
            let parser = Parser::from(user_config).unwrap();
            let input = "---\na_key: a_value\nanother_key: another_value\n---\n# Question\n\
                         a question?\n# Answer \nan answer\n\n----\nBacklink: SOMELINK\n";
            let actual = parser.parse(&input);
            assert!(actual.is_err());
            assert!(actual
                .unwrap_err()
                .contains("Could not match DECKS against pattern"));
        }

        #[test]
        fn parse_where_question_expression_has_no_captures() {
            let user_config = ParsingConfig::default();
            let parser = Parser::from(user_config).unwrap();
            let input = "---\ntags: :a:\nanother_key: another_value\n---\n# A Question\n\
                         a question?\n# Answer \nan answer\n\n----\nBacklink: SOMELINK\n";
            let actual = parser.parse(&input);
            assert!(actual.is_err());
            assert!(actual
                .unwrap_err()
                .contains("Could not match QUESTION against pattern"));
        }

        #[test]
        fn parse_where_answer_expression_has_no_captures() {
            let user_config = ParsingConfig::default();
            let parser = Parser::from(user_config).unwrap();
            let input = "---\ntags: :a:\nanother_key: another_value\n---\n# Question\n\
                         a question?\n# Answer \nan answer\n\n--_--\nBacklink: SOMELINK\n";
            let actual = parser.parse(&input);
            assert!(actual.is_err());
            assert!(actual
                .unwrap_err()
                .contains("Could not match ANSWER against pattern"));
        }
    }
}
