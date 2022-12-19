use std::error::Error;
use vultan::state::card;
use vultan::state::card::parser::{Parser, ParsingConfig};
use vultan::state::deck;
use vultan::state::file::FileHandle;
use vultan::state::State;
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
    let fake_notes_dir = std::path::PathBuf::from(r"./tests/res/");
    let state_handle = FileHandle::from(std::path::PathBuf::from(r"./tests/res/.vultan.ron"));
    let state = State::from_file(state_handle).unwrap_or(State::default());
    let parser = Parser::from(&state.card_parsing_config)?;
    let cards = card::try_load_many(fake_notes_dir, &parser)?;
    let decks = deck::many_from_cards(&cards.succeeded);
    let state = state
        .with_merged_cards(cards.succeeded)
        .with_merged_decks(decks);
    println!("{:#?}", &state);
    Ok(())
}
