pub mod card;
pub mod deck;
pub mod hand;

use deck::Deck;
use card::Card;

struct ApplicationState {
    decks: Vec<Deck>,
    cards: Vec<Card>
}
