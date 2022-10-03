use super::card::Card;
use super::deck::Deck;

pub struct Hand<'a> {
    cards: Vec<&'a Card>,
}

impl<'a> Hand<'a> {
    pub fn deal(deck: &Deck, cards: &'a Vec<Card>) -> Hand<'a> {
        Hand { cards: Vec::new() }
    }

    fn is_due(card: &Card) -> bool {
        false
    }
}
