//! Gift cards: sold with a balance that payments draw down.
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Card {
    pub sold: i64,
    pub balance: i64,
}

#[derive(Default)]
pub struct Cards {
    sold_cards: BTreeMap<String, Card>,
}

impl Cards {
    pub fn new() -> Cards {
        Cards::default()
    }

    pub fn has(&self, card: &str) -> bool {
        self.sold_cards.contains_key(card)
    }

    pub fn find(&self, card: &str) -> Option<&Card> {
        self.sold_cards.get(card)
    }

    pub fn sell(&mut self, card: &str, cents: i64) {
        self.sold_cards.insert(card.to_string(), Card { sold: cents, balance: cents });
    }

    /// Takes `cents` from a card's balance.
    pub fn draw(&mut self, card: &str, cents: i64) {
        self.sold_cards.get_mut(card).expect("sold card").balance -= cents;
    }

    /// Every card sold, in ascending byte order.
    pub fn all_cards(&self) -> Vec<String> {
        self.sold_cards.keys().cloned().collect()
    }
}
