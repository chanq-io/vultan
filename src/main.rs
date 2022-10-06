use vultan::state::card::parser::{Parser, ParsingConfig};
use vultan::state::card::Card;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let config = ParsingConfig::default();
    let parser = Parser::from(config)?;
    println!("{:?}", Card::from("./test_card.md", &parser));
    Ok(())
}
