//! Pricing of order lines (spec "Prices"). The amounts are computed from the fields an
//! order fixes when it is placed (price, tax rate, tier discount) and the order's promo.
use crate::{catalog, customers, promos};

#[derive(Clone, Debug)]
pub struct Line {
    pub sku: String,
    pub qty: i64,
    pub price: i64,
    pub cat: String,
    /// Tax rate of the category, in percent, when the order was placed.
    pub rate: i64,
    /// Tier discount of the customer, in percent, when the order was placed.
    pub tier: i64,
    pub gross: i64,
    /// Tier discount plus promo discount.
    pub discount: i64,
    pub net: i64,
    pub tax: i64,
    pub total: i64,
    /// What the shipped units cost the shop; 0 until the order ships.
    pub cost: i64,
}

/// A line whose amounts `priced` has still to fill in.
pub fn fresh_line(sku: &str, qty: i64, price: i64, cat: &str, rate: i64, tier: i64) -> Line {
    Line {
        sku: sku.to_string(),
        qty,
        price,
        cat: cat.to_string(),
        rate,
        tier,
        gross: 0,
        discount: 0,
        net: 0,
        tax: 0,
        total: 0,
        cost: 0,
    }
}

/// The line with its amounts computed. The promo (`promo_pct` off lines of `promo_cat`)
/// applies only to lines of that category; pass `""` when the order has no promo.
pub fn priced(l: &Line, promo_cat: &str, promo_pct: i64) -> Line {
    let mut p = l.clone();
    p.gross = l.price * l.qty;
    let tier_off = customers::discount(p.gross, l.tier);
    let mut promo_off = 0;
    if promo_cat == l.cat {
        promo_off = promos::discount(p.gross - tier_off, promo_pct);
    }
    p.discount = tier_off + promo_off;
    p.net = p.gross - p.discount;
    p.tax = catalog::tax(p.net, l.rate);
    p.total = p.net + p.tax;
    p
}
