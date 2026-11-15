//! Promo codes: a percentage off the lines of one category, on a limited number of orders.
use crate::money;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Promo {
    pub cat: String,
    pub pct: i64,
    pub uses: i64,
    /// Orders the code is on now; a cancelled order gives its use back.
    pub used: i64,
}

#[derive(Default)]
pub struct Promos {
    codes: BTreeMap<String, Promo>,
}

impl Promos {
    pub fn new() -> Promos {
        Promos::default()
    }

    pub fn has(&self, code: &str) -> bool {
        self.codes.contains_key(code)
    }

    pub fn get(&self, code: &str) -> Option<&Promo> {
        self.codes.get(code)
    }

    /// Every code, in ascending byte order.
    pub fn all_codes(&self) -> Vec<String> {
        self.codes.keys().cloned().collect()
    }

    pub fn add(&mut self, code: &str, p: Promo) {
        self.codes.insert(code.to_string(), p);
    }

    /// Counts one more order using `code` (`by` 1) or gives one use back (`by` -1).
    pub fn count_use(&mut self, code: &str, by: i64) {
        self.codes.get_mut(code).expect("defined promo").used += by;
    }
}

/// The promo discount on what is left of a line after its tier discount.
pub fn discount(amount: i64, pct: i64) -> i64 {
    money::percent(amount, pct)
}
