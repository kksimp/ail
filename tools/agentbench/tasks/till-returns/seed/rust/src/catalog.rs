//! The catalog: each SKU's category, price and stock, and each category's tax rate.
use crate::money;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Item {
    pub cat: String,
    pub price: i64,
    pub onhand: i64,
    /// Units held by orders that are open or paid.
    pub reserved: i64,
    /// What the units on hand cost the shop, in cents (average cost).
    pub value: i64,
}

impl Item {
    /// Units on hand that no order holds.
    pub fn available(&self) -> i64 {
        self.onhand - self.reserved
    }
}

#[derive(Default)]
pub struct Catalog {
    items: BTreeMap<String, Item>,
    rates: BTreeMap<String, i64>,
}

impl Catalog {
    pub fn new() -> Catalog {
        Catalog::default()
    }

    pub fn has(&self, sku: &str) -> bool {
        self.items.contains_key(sku)
    }

    pub fn get(&self, sku: &str) -> Option<&Item> {
        self.items.get(sku)
    }

    /// The item of a SKU that `has` reports; panics otherwise.
    pub fn item(&self, sku: &str) -> &Item {
        &self.items[sku]
    }

    /// Every SKU in the catalog, in ascending byte order.
    pub fn skus(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }

    pub fn put(&mut self, sku: &str, it: Item) {
        self.items.insert(sku.to_string(), it);
    }

    /// The tax rate of a category in percent; 0 until a `tax` entry sets it.
    pub fn rate(&self, cat: &str) -> i64 {
        *self.rates.get(cat).unwrap_or(&0)
    }

    pub fn set_rate(&mut self, cat: &str, pct: i64) {
        self.rates.insert(cat.to_string(), pct);
    }

    fn item_mut(&mut self, sku: &str) -> &mut Item {
        self.items.get_mut(sku).expect("SKU in the catalog")
    }

    /// Changes the reserved quantity of a SKU by `by` units; a negative `by` releases them.
    pub fn reserve(&mut self, sku: &str, by: i64) {
        self.item_mut(sku).reserved += by;
    }

    /// Ships `qty` reserved units: they leave both the on-hand and the reserved quantity,
    /// taking their share of the stock value. Returns that share, the cost of the units.
    pub fn take(&mut self, sku: &str, qty: i64) -> i64 {
        let it = self.item_mut(sku);
        let cost = it.value * qty / it.onhand;
        it.value -= cost;
        it.onhand -= qty;
        it.reserved -= qty;
        cost
    }

    /// Adds `qty` units that cost `cents` in all.
    pub fn receive(&mut self, sku: &str, qty: i64, cents: i64) {
        let it = self.item_mut(sku);
        it.onhand += qty;
        it.value += cents;
    }

    /// A stocktake: sets the units on hand, scaling the stock value with them.
    pub fn stocktake(&mut self, sku: &str, qty: i64) {
        let it = self.item_mut(sku);
        if it.onhand > 0 {
            it.value = it.value * qty / it.onhand;
        }
        it.onhand = qty;
    }
}

/// The tax on a line's net amount at `rate` percent.
pub fn tax(net: i64, rate: i64) -> i64 {
    money::percent(net, rate)
}
