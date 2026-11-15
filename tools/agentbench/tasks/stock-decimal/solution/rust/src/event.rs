//! One line of the stock log.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Receive,
    Ship,
    Count,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: Kind,
    pub sku: String,
    /// Thousandths of a unit.
    pub qty: i64,
    /// Cents per whole unit.
    pub price: i64,
}

/// 1 to 5 ASCII digits, as a number.
pub fn number(s: &str) -> Option<i64> {
    if s.is_empty() || s.len() > 5 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// 1 to 5 digits, optionally `.` and 1 to 3 digits, as thousandths: `2.5` is 2500.
pub fn quantity(s: &str) -> Option<i64> {
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) => (w, f),
        None => (s, ""),
    };
    if s.contains('.') && (frac.is_empty() || frac.len() > 3 || !frac.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let w = number(whole)?;
    let f: i64 = format!("{frac:0<3}").parse().ok()?;
    Some(w * 1000 + f)
}

pub fn valid_sku(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}

/// The event on a line, or None if the line is not a valid event.
pub fn parse(line: &str) -> Option<Event> {
    let f: Vec<&str> = line.split(' ').collect();
    let kind = match (f[0], f.len()) {
        ("receive", 4) => Kind::Receive,
        ("ship", 3) => Kind::Ship,
        ("count", 3) => Kind::Count,
        _ => return None,
    };
    if !valid_sku(f[1]) {
        return None;
    }
    let qty = quantity(f[2])?;
    let price = if kind == Kind::Receive { number(f[3])? } else { 0 };
    if qty == 0 && kind != Kind::Count {
        return None;
    }
    Some(Event { kind, sku: f[1].to_string(), qty, price })
}
