//! Orders: their lines, promo, payments and status.
use crate::pricing::Line;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Open,
    Paid,
    Shipped,
    Cancelled,
}

impl Status {
    /// The status as the reports print it.
    pub fn text(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Paid => "paid",
            Status::Shipped => "shipped",
            Status::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Order {
    pub id: String,
    pub cust: String,
    pub status: Status,
    /// In the order the `order` entry listed them.
    pub lines: Vec<Line>,
    /// The promo code on the order, or `""`.
    pub promo: String,
    /// What has been paid on the order; never more than its total.
    pub paid: i64,
}

impl Order {
    /// The sum of the line totals.
    pub fn total(&self) -> i64 {
        self.lines.iter().map(|l| l.total).sum()
    }

    /// What is still to pay.
    pub fn due(&self) -> i64 {
        self.total() - self.paid
    }

    /// Whether a line of the order is in category `cat`.
    pub fn has_cat(&self, cat: &str) -> bool {
        self.lines.iter().any(|l| l.cat == cat)
    }
}

#[derive(Default)]
pub struct Orders {
    placed: Vec<Order>,
    index: HashMap<String, usize>,
}

impl Orders {
    pub fn new() -> Orders {
        Orders::default()
    }

    pub fn has(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }

    pub fn find(&self, id: &str) -> Option<&Order> {
        self.index.get(id).map(|&i| &self.placed[i])
    }

    /// Adds a new order after the others, or replaces the order with the same id in place.
    pub fn put(&mut self, o: Order) {
        let found = self.index.get(&o.id).copied();
        match found {
            Some(i) => self.placed[i] = o,
            None => {
                self.index.insert(o.id.clone(), self.placed.len());
                self.placed.push(o);
            }
        }
    }

    /// Every order, in the order they were placed.
    pub fn all(&self) -> &[Order] {
        &self.placed
    }

    /// The orders of one customer, in the order they were placed.
    pub fn of(&self, cust: &str) -> Vec<&Order> {
        self.placed.iter().filter(|o| o.cust == cust).collect()
    }
}
