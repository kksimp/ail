//! Quantities per location and average cost per SKU.

use crate::event::{Event, Kind};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Entry {
    /// Units per location, in ascending byte order of location.
    pub at: BTreeMap<String, i64>,
    pub cost: i64,
}

impl Entry {
    /// Units over all locations.
    pub fn qty(&self) -> i64 {
        self.at.values().sum()
    }

    fn at(&self, loc: &str) -> i64 {
        self.at.get(loc).copied().unwrap_or(0)
    }
}

pub enum Problem {
    Unknown(String),
    Short(String),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Problem::Unknown(s) => write!(f, "unknown {s}"),
            Problem::Short(s) => write!(f, "short {s}"),
        }
    }
}

pub struct Store {
    items: BTreeMap<String, Entry>,
}

impl Store {
    pub fn new() -> Store {
        Store { items: BTreeMap::new() }
    }

    /// Every SKU ever received, in ascending byte order, with its entry.
    pub fn items(&self) -> impl Iterator<Item = (&String, &Entry)> + '_ {
        self.items.iter()
    }

    pub fn get(&self, sku: &str) -> Option<&Entry> {
        self.items.get(sku)
    }

    /// Applies one event; a rejected event changes nothing.
    pub fn apply(&mut self, e: &Event) -> Result<(), Problem> {
        if e.kind == Kind::Receive {
            let it = self.items.entry(e.sku.clone()).or_default();
            *it.at.entry(e.loc.clone()).or_insert(0) += e.qty;
            it.cost += e.qty * e.price;
            return Ok(());
        }
        let Some(it) = self.items.get_mut(&e.sku) else {
            return Err(Problem::Unknown(e.sku.clone()));
        };
        let total = it.qty();
        match e.kind {
            Kind::Ship => {
                if e.qty > it.at(&e.loc) {
                    return Err(Problem::Short(e.sku.clone()));
                }
                it.cost -= it.cost * e.qty / total;
                *it.at.entry(e.loc.clone()).or_insert(0) -= e.qty;
            }
            Kind::Move => {
                if e.qty > it.at(&e.loc) {
                    return Err(Problem::Short(e.sku.clone()));
                }
                *it.at.entry(e.loc.clone()).or_insert(0) -= e.qty;
                *it.at.entry(e.to.clone()).or_insert(0) += e.qty;
            }
            _ => {
                let new_total = total - it.at(&e.loc) + e.qty;
                if total > 0 {
                    it.cost = it.cost * new_total / total;
                }
                it.at.insert(e.loc.clone(), e.qty);
            }
        }
        Ok(())
    }
}
