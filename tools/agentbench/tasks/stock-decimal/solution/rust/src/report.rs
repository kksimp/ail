//! The three outputs.

use crate::store::Store;

/// Cents as dollars with two decimals: `1234.05`, `-0.50`.
pub fn money(c: i64) -> String {
    let sign = if c < 0 { "-" } else { "" };
    let a = c.unsigned_abs();
    format!("{}{}.{:02}", sign, a / 100, a % 100)
}

/// Thousandths with three decimals: `2.500`.
pub fn qty(q: i64) -> String {
    format!("{}.{:03}", q / 1000, q % 1000)
}

pub fn report(s: &Store) -> Vec<String> {
    s.items().map(|(sku, it)| format!("{}\t{}\t{}", sku, qty(it.qty), money(it.cost))).collect()
}

pub fn low(s: &Store, n: i64) -> Vec<String> {
    s.items().filter(|(_, it)| it.qty < n).map(|(sku, _)| sku.clone()).collect()
}

pub fn total(s: &Store) -> Vec<String> {
    vec![money(s.items().map(|(_, it)| it.cost).sum())]
}
