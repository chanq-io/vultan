pub mod card;
pub mod deck;
pub mod hand;

use deck::Deck;
use card::Card;

struct ApplicationState<'a> {
    decks: Vec<Deck<'a>>,
    cards: Vec<Card>
}
