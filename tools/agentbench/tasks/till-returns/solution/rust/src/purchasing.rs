//! Purchase orders to suppliers: goods ordered, then received into stock.
use crate::journal::Buy;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoStatus {
    Ordered,
    Received,
}

impl PoStatus {
    pub fn text(self) -> &'static str {
        match self {
            PoStatus::Ordered => "ordered",
            PoStatus::Received => "received",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Purchase {
    pub po: String,
    pub status: PoStatus,
    pub buys: Vec<Buy>,
}

impl Purchase {
    /// The units of every SKU on the purchase order.
    pub fn units(&self) -> i64 {
        self.buys.iter().map(|b| b.qty).sum()
    }

    /// What the goods cost: QTY × CENTS over every SKU.
    pub fn cost(&self) -> i64 {
        self.buys.iter().map(|b| b.qty * b.cents).sum()
    }
}

#[derive(Default)]
pub struct Purchases {
    entered: Vec<Purchase>,
    index: HashMap<String, usize>,
}

impl Purchases {
    pub fn new() -> Purchases {
        Purchases::default()
    }

    pub fn has(&self, po: &str) -> bool {
        self.index.contains_key(po)
    }

    pub fn find(&self, po: &str) -> Option<&Purchase> {
        self.index.get(po).map(|&i| &self.entered[i])
    }

    /// Adds a new purchase order after the others, or replaces the one with the same PO in place.
    pub fn put(&mut self, p: Purchase) {
        let found = self.index.get(&p.po).copied();
        match found {
            Some(i) => self.entered[i] = p,
            None => {
                self.index.insert(p.po.clone(), self.entered.len());
                self.entered.push(p);
            }
        }
    }

    /// Every purchase order, in the order they were entered.
    pub fn all(&self) -> &[Purchase] {
        &self.entered
    }
}
