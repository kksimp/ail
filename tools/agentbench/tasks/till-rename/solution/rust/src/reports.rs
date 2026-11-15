//! The reports, as lines without their newline (spec "Reports").
use crate::money::show;
use crate::orders::Status;
use crate::shop::Shop;
use std::collections::BTreeMap;

/// Each placed order: `ORDER ID STATUS TOTAL PAID`.
pub fn order_lines(s: &Shop) -> Vec<String> {
    s.book
        .all()
        .iter()
        .map(|o| format!("{}\t{}\t{}\t{}\t{}", o.id, o.account, o.status.text(), show(o.total()), show(o.paid)))
        .collect()
}

/// The invoice of order `id`, or `None` if it was never placed.
pub fn invoice(s: &Shop, id: &str) -> Option<Vec<String>> {
    let o = s.book.find(id)?;
    let mut out = vec![format!("order {} {} {}", o.id, o.account, o.status.text())];
    let (mut gross, mut discount, mut tax) = (0, 0, 0);
    for l in &o.lines {
        let amounts: Vec<String> = [l.price, l.gross, l.discount, l.tax, l.total].iter().map(|&c| show(c)).collect();
        out.push(format!("{}\t{}\t{}", l.sku, l.qty, amounts.join("\t")));
        gross += l.gross;
        discount += l.discount;
        tax += l.tax;
    }
    if !o.promo.is_empty() {
        out.push(format!("promo\t{}", o.promo));
    }
    let due = if o.status == Status::Open { o.due() } else { 0 };
    out.push(format!("subtotal\t{}", show(gross)));
    out.push(format!("discount\t{}", show(discount)));
    out.push(format!("tax\t{}", show(tax)));
    out.push(format!("total\t{}", show(o.total())));
    out.push(format!("paid\t{}", show(o.paid)));
    out.push(format!("due\t{}", show(due)));
    Some(out)
}

/// Each catalog SKU: `SKU CAT PRICE ONHAND RESERVED AVAILABLE VALUE`.
pub fn stock_lines(s: &Shop) -> Vec<String> {
    let mut out = Vec::new();
    for sku in s.stock.skus() {
        let it = s.stock.item(&sku);
        out.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sku,
            it.cat,
            show(it.price),
            it.onhand,
            it.reserved,
            it.available(),
            show(it.value)
        ));
    }
    out
}

/// Each catalog SKU with fewer than `n` units available: `SKU AVAILABLE`.
pub fn low(s: &Shop, n: i64) -> Vec<String> {
    let mut out = Vec::new();
    for sku in s.stock.skus() {
        let left = s.stock.item(&sku).available();
        if left < n {
            out.push(format!("{}\t{}", sku, left));
        }
    }
    out
}

/// Each account: `ID TIER ORDERS SPENT CREDIT`.
pub fn account_lines(s: &Shop) -> Vec<String> {
    let mut out = Vec::new();
    for account in s.people.ids() {
        let c = s.people.get(&account);
        let mut placed = 0;
        let mut spent = 0;
        for o in s.book.of(&account) {
            if o.status != Status::Cancelled {
                placed += 1;
            }
            if o.status == Status::Shipped {
                spent += o.total();
            }
        }
        out.push(format!("{}\t{}\t{}\t{}\t{}", account, c.tier, placed, show(spent), show(c.credit)));
    }
    out
}

/// A account's orders, `ORDER STATUS TOTAL`, then `credit`; `None` for a account never registered.
pub fn history(s: &Shop, account: &str) -> Option<Vec<String>> {
    if !s.people.has(account) {
        return None;
    }
    let mut out: Vec<String> =
        s.book.of(account).iter().map(|o| format!("{}\t{}\t{}", o.id, o.status.text(), show(o.total()))).collect();
    out.push(format!("credit\t{}", show(s.people.get(account).credit)));
    Some(out)
}

#[derive(Default, Clone, Copy)]
struct Tally {
    units: i64,
    net: i64,
    tax: i64,
}

/// Units, net and tax of the shipped lines, by category, then `total`.
pub fn sales(s: &Shop) -> Vec<String> {
    let mut cats: BTreeMap<String, Tally> = BTreeMap::new();
    let mut all = Tally::default();
    for o in s.book.all().iter().filter(|o| o.status == Status::Shipped) {
        for l in &o.lines {
            let t = cats.entry(l.cat.clone()).or_default();
            t.units += l.qty;
            t.net += l.net;
            t.tax += l.tax;
            all.units += l.qty;
            all.net += l.net;
            all.tax += l.tax;
        }
    }
    let mut out: Vec<String> =
        cats.iter().map(|(cat, t)| format!("{}\t{}\t{}\t{}", cat, t.units, show(t.net), show(t.tax))).collect();
    out.push(format!("total\t{}\t{}\t{}", all.units, show(all.net), show(all.tax)));
    out
}

#[derive(Default, Clone, Copy)]
struct Margin {
    net: i64,
    cost: i64,
}

fn margin_line(name: &str, m: Margin) -> String {
    format!("{}\t{}\t{}\t{}", name, show(m.net), show(m.cost), show(m.net - m.cost))
}

/// Net, cost and margin of the shipped lines, by category, then `total`.
pub fn margin(s: &Shop) -> Vec<String> {
    let mut cats: BTreeMap<String, Margin> = BTreeMap::new();
    let mut all = Margin::default();
    for o in s.book.all().iter().filter(|o| o.status == Status::Shipped) {
        for l in &o.lines {
            let m = cats.entry(l.cat.clone()).or_default();
            m.net += l.net;
            m.cost += l.cost;
            all.net += l.net;
            all.cost += l.cost;
        }
    }
    let mut out: Vec<String> = cats.iter().map(|(cat, m)| margin_line(cat, *m)).collect();
    out.push(margin_line("total", all));
    out
}

/// Each purchase order: `PO STATUS UNITS COST`.
pub fn purchase_lines(s: &Shop) -> Vec<String> {
    s.buying.all().iter().map(|p| format!("{}\t{}\t{}\t{}", p.po, p.status.text(), p.units(), show(p.cost()))).collect()
}

/// Each gift card: `CARD SOLD BALANCE`.
pub fn gift_lines(s: &Shop) -> Vec<String> {
    let mut out = Vec::new();
    for card in s.cards.all_cards() {
        if let Some(c) = s.cards.find(&card) {
            out.push(format!("{}\t{}\t{}", card, show(c.sold), show(c.balance)));
        }
    }
    out
}

/// Each promo code: `CODE CAT PCT% USED/USES`.
pub fn promo_lines(s: &Shop) -> Vec<String> {
    let mut out = Vec::new();
    for code in s.deals.all_codes() {
        if let Some(p) = s.deals.get(&code) {
            out.push(format!("{}\t{}\t{}%\t{}/{}", code, p.cat, p.pct, p.used, p.uses));
        }
    }
    out
}
