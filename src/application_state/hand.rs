use super::card::Card;
use super::deck::Deck;

pub struct Hand {
    cards: Vec<Card>
}

impl Hand {
    pub fn deal (deck: &Deck, cards: &Vec<Card>) -> Hand {
        Hand {cards: Vec::new()}
    }

    fn is_due(card: &Card) -> bool {
        false
    }
}
