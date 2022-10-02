use clanki::application_state::card::Card;
use clanki::application_state::card::parser::{Parser, ParsingConfig, ParsingPattern};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>>{
    let config = ParsingConfig{
        tags_pattern: ParsingPattern::TaggedLine {
            tag: String::from(r"tags:"),
        },
        tag_delimiter: String::from(":"),
        question_pattern: ParsingPattern::WrappedMultiLine {
            opening_tag: String::from(r"# Question"),
            closing_tag: String::from(r"# Answer"),
        },
        answer_pattern: ParsingPattern::WrappedMultiLine {
            opening_tag: String::from(r"# Answer"),
            closing_tag: String::from(r"----\n"),
        },
    };
    let parser = Parser::from(config)?;
    println!("{:?}", Card::from("./test_card.md", &parser));
    Ok(())
}
