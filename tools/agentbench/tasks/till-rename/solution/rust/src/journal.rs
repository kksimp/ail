//! One line of the journal as an entry (spec "Journal"). Only the syntax is checked here;
//! the rules that need the shop's state are in `shop`.
use crate::fields;

/// One `SKU:QTY` of an `order` entry.
#[derive(Clone, Debug)]
pub struct Want {
    pub sku: String,
    pub qty: i64,
}

/// One `SKU:QTY@CENTS` of a `buy` entry.
#[derive(Clone, Debug)]
pub struct Buy {
    pub sku: String,
    pub qty: i64,
    pub cents: i64,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Item { sku: String, cat: String, cents: i64 },
    Tax { cat: String, pct: i64 },
    Restock { sku: String, qty: i64 },
    Adjust { sku: String, qty: i64 },
    Account { account: String, tier: String },
    Promo { code: String, cat: String, pct: i64, uses: i64 },
    Purchase { po: String, buys: Vec<Buy> },
    Receive { po: String },
    Gift { card: String, cents: i64 },
    Order { id: String, account: String, wants: Vec<Want> },
    Apply { id: String, code: String },
    Pay { id: String, cents: i64 },
    PayCredit { id: String },
    PayGift { id: String, card: String },
    Ship { id: String },
    Cancel { id: String },
}

/// The line is not a valid entry.
#[derive(Debug)]
pub struct BadLine;

/// A number of 1 to `width` digits that is at least `least`.
fn num(s: &str, width: usize, least: i64) -> Result<i64, BadLine> {
    match fields::digits(s, width) {
        Some(v) if v >= least => Ok(v),
        _ => Err(BadLine),
    }
}

/// The items of an `order` entry: `SKU:QTY` separated by commas, no SKU twice.
fn wants(s: &str) -> Result<Vec<Want>, BadLine> {
    let mut out: Vec<Want> = Vec::new();
    for part in s.split(',') {
        let p: Vec<&str> = part.split(':').collect();
        if p.len() != 2 || !fields::is_sku(p[0]) {
            return Err(BadLine);
        }
        let qty = num(p[1], 5, 1)?;
        if out.iter().any(|w| w.sku == p[0]) {
            return Err(BadLine);
        }
        out.push(Want { sku: p[0].to_string(), qty });
    }
    Ok(out)
}

/// The goods of a `buy` entry: `SKU:QTY@CENTS` separated by commas, no SKU twice.
fn buys(s: &str) -> Result<Vec<Buy>, BadLine> {
    let mut out: Vec<Buy> = Vec::new();
    for part in s.split(',') {
        let at: Vec<&str> = part.split('@').collect();
        if at.len() != 2 {
            return Err(BadLine);
        }
        let p: Vec<&str> = at[0].split(':').collect();
        if p.len() != 2 || !fields::is_sku(p[0]) {
            return Err(BadLine);
        }
        let qty = num(p[1], 5, 1)?;
        let cents = num(at[1], 7, 0)?;
        if out.iter().any(|b| b.sku == p[0]) {
            return Err(BadLine);
        }
        out.push(Buy { sku: p[0].to_string(), qty, cents });
    }
    Ok(out)
}

/// The entry on a line, or `BadLine`.
pub fn parse(line: &str) -> Result<Entry, BadLine> {
    let f: Vec<&str> = line.split(' ').collect();
    let n = f.len();
    let s = |i: usize| f[i].to_string();
    match f[0] {
        "item" if n == 4 && fields::is_sku(f[1]) && fields::is_cat(f[2]) => {
            Ok(Entry::Item { sku: s(1), cat: s(2), cents: num(f[3], 7, 0)? })
        }
        "tax" if n == 3 && fields::is_cat(f[1]) => Ok(Entry::Tax { cat: s(1), pct: num(f[2], 2, 0)? }),
        "restock" if n == 3 && fields::is_sku(f[1]) => Ok(Entry::Restock { sku: s(1), qty: num(f[2], 5, 1)? }),
        "adjust" if n == 3 && fields::is_sku(f[1]) => Ok(Entry::Adjust { sku: s(1), qty: num(f[2], 5, 0)? }),
        "account" if n == 3 && fields::is_id(f[1]) && fields::is_tier(f[2]) => {
            Ok(Entry::Account { account: s(1), tier: s(2) })
        }
        "promo" if n == 5 && fields::is_code(f[1]) && fields::is_cat(f[2]) => Ok(Entry::Promo {
            code: s(1),
            cat: s(2),
            pct: num(f[3], 2, 0)?,
            uses: num(f[4], 3, 1)?,
        }),
        "buy" if n == 3 && fields::is_po(f[1]) => Ok(Entry::Purchase { po: s(1), buys: buys(f[2])? }),
        "receive" if n == 2 && fields::is_po(f[1]) => Ok(Entry::Receive { po: s(1) }),
        "gift" if n == 3 && fields::is_card(f[1]) => Ok(Entry::Gift { card: s(1), cents: num(f[2], 7, 1)? }),
        "order" if n == 4 && fields::is_order(f[1]) && fields::is_id(f[2]) => {
            Ok(Entry::Order { id: s(1), account: s(2), wants: wants(f[3])? })
        }
        "apply" if n == 3 && fields::is_order(f[1]) && fields::is_code(f[2]) => Ok(Entry::Apply { id: s(1), code: s(2) }),
        "pay" if n == 3 && fields::is_order(f[1]) && f[2] == "credit" => Ok(Entry::PayCredit { id: s(1) }),
        "pay" if n == 3 && fields::is_order(f[1]) && fields::is_card(f[2]) => Ok(Entry::PayGift { id: s(1), card: s(2) }),
        "pay" if n == 3 && fields::is_order(f[1]) => Ok(Entry::Pay { id: s(1), cents: num(f[2], 7, 1)? }),
        "ship" if n == 2 && fields::is_order(f[1]) => Ok(Entry::Ship { id: s(1) }),
        "cancel" if n == 2 && fields::is_order(f[1]) => Ok(Entry::Cancel { id: s(1) }),
        _ => Err(BadLine),
    }
}
