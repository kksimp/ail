//! Registered customers: their tier and their credit.
use crate::money;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Customer {
    pub tier: String,
    /// Cents the shop owes the customer: overpayments and refunds of cancelled orders.
    pub credit: i64,
}

#[derive(Default)]
pub struct Customers {
    known: BTreeMap<String, Customer>,
}

impl Customers {
    pub fn new() -> Customers {
        Customers::default()
    }

    pub fn has(&self, id: &str) -> bool {
        self.known.contains_key(id)
    }

    /// The customer `id`, which `has` reports; panics otherwise.
    pub fn get(&self, id: &str) -> &Customer {
        &self.known[id]
    }

    /// Every registered customer, in ascending byte order.
    pub fn ids(&self) -> Vec<String> {
        self.known.keys().cloned().collect()
    }

    /// Registers `id` with no credit, or changes the tier of a registered customer.
    pub fn set_tier(&mut self, id: &str, tier: &str) {
        let c = self.known.entry(id.to_string()).or_insert(Customer { tier: String::new(), credit: 0 });
        c.tier = tier.to_string();
    }

    /// Adds `cents` to the customer's credit; a negative amount takes credit away.
    pub fn add_credit(&mut self, id: &str, cents: i64) {
        self.known.get_mut(id).expect("registered customer").credit += cents;
    }
}

/// The tier discount in percent: 0 for `basic`, 5 for `silver`, 10 for `gold`.
pub fn tier_pct(tier: &str) -> i64 {
    match tier {
        "gold" => 10,
        "silver" => 5,
        _ => 0,
    }
}

/// The tier discount on a line's gross amount.
pub fn discount(gross: i64, pct: i64) -> i64 {
    money::percent(gross, pct)
}
