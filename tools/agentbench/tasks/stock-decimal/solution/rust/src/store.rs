//! On-hand quantities and average cost per SKU.

use crate::event::{Event, Kind};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub qty: i64,
    pub cost: i64,
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

    /// Every SKU ever received, in ascending byte order, with its item.
    pub fn items(&self) -> impl Iterator<Item = (&String, &Entry)> + '_ {
        self.items.iter()
    }

    /// Applies one event; a rejected event changes nothing.
    pub fn apply(&mut self, e: &Event) -> Result<(), Problem> {
        if e.kind == Kind::Receive {
            let it = self.items.entry(e.sku.clone()).or_default();
            it.qty += e.qty;
            it.cost += e.qty * e.price / 1000;
            return Ok(());
        }
        let Some(it) = self.items.get_mut(&e.sku) else {
            return Err(Problem::Unknown(e.sku.clone()));
        };
        match e.kind {
            Kind::Ship => {
                if e.qty > it.qty {
                    return Err(Problem::Short(e.sku.clone()));
                }
                it.cost -= it.cost * e.qty / it.qty;
                it.qty -= e.qty;
            }
            _ => {
                if it.qty > 0 {
                    it.cost = it.cost * e.qty / it.qty;
                }
                it.qty = e.qty;
            }
        }
        Ok(())
    }
}
