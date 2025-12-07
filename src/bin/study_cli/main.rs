use anyhow::{Context, Result};
use clap::Parser;

mod repl;
use vultan::state;

#[derive(Parser, Debug)]
#[command(name = "study-cli")]
#[command(about = "Study flashcards from a deck", long_about = None)]
struct Args {
    /// Path to the notes directory
    #[arg(short, long, default_value = "./tests/res")]
    notes_dirpath: String,

    /// Name of the deck to study
    #[arg(short, long, default_value = "topic-1")]
    deck_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    study(&args.notes_dirpath, &args.deck_name)
}

pub fn study(notes_dirpath: &str, deck_name: &str) -> Result<()> {
    let notes_dirpath = std::path::PathBuf::from(notes_dirpath);
    let state = state::State::read(notes_dirpath.to_owned()).context("Unable to read State")?;
    let deck = state
        .get_deck(deck_name)
        .with_context(|| format!("Unable to fetch deck: {deck_name}"))?;
    let hand = state.deal(deck_name).ok();
    let revised_cards = repl::run(deck, hand)?;
    let state = state.with_overriden_cards(revised_cards);
    let state_file_handle =
        state::file::FileHandle::from(notes_dirpath.join(state::STATE_FILENAME));
    println!("{:#?}", &state);
    state.write(state_file_handle)?;

    Ok(())
}
