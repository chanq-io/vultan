#![allow(dead_code)] // TODO remove
#![allow(unused_variables)] // TODO remove
pub mod repl;
pub mod state;

use anyhow::{Context, Result};

pub fn study(notes_dirpath: &str, deck_name: &str) -> Result<()> {
    let notes_dirpath = std::path::PathBuf::from(notes_dirpath);
    let state = state::State::read(notes_dirpath.to_owned()).context("Unable to read State")?;
    // TODO add repl page for no cards due
    let deck = state
        .get_deck(deck_name)
        .with_context(|| format!("Unable to fetch deck: {deck_name}"))?;
    let hand = state
        .deal(deck_name)
        .with_context(|| format!("Unable to create revision queue for deck: {deck_name}"))?;
    let revised_cards = repl::run(deck, hand)?;
    let state = state.with_overriden_cards(revised_cards);
    let state_file_handle = state::file::FileHandle::from(std::path::PathBuf::from(
        notes_dirpath.join(state::STATE_FILENAME),
    ));
    println!("{:#?}", &state);
    state.write(state_file_handle)?;
    Ok(())
}
