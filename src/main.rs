use vultan::state::card::parser::{Parser, ParsingConfig};
use vultan::state::file::FileHandle;
use vultan::state::card::Card;
use std::error::Error;
/*
 * let state = State::read(&args.notes_dir);
 *    -> let state = Self::read_or_default(notes_dir)
 *    -> let card_parser = Parser::from(&state.card_parsing_config);
 *    -> let cards = Card::read_all_or_default(&card_parser, &args.notes_dir);
 *    -> let decks = Deck::many_from_cards(cards);
 *    -> state.with_merged_cards(cards).with_merged_decks(decks)
 * let hand = state.deal_hand(args.deck_name);
 * let revised_cards = hand.revise_until_none_fail(Repl::run_repl)
 * let state = state.with_overriden_cards(revised_cards);
 * State::write(&args.notes_dir);
 * */
fn main() -> Result<(), Box<dyn Error>> {
    let config = ParsingConfig::default();
    let parser = Parser::from(config)?;
    let file_handle = FileHandle::from("./test_card.md".to_string());
    println!("{:?}", Card::from(file_handle, &parser));
    Ok(())
}
