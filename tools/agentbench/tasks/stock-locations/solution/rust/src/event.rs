//! One line of the stock log.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Receive,
    Ship,
    Count,
    Move,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: Kind,
    pub sku: String,
    pub qty: i64,
    pub price: i64,
    /// Where the event happens; for `move`, the source.
    pub loc: String,
    /// The destination of a `move`; empty otherwise.
    pub to: String,
}

/// 1 to 5 ASCII digits, as a number.
pub fn number(s: &str) -> Option<i64> {
    if s.is_empty() || s.len() > 5 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

pub fn valid_sku(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}

fn valid_loc(s: &str) -> bool {
    match s.strip_prefix('@') {
        Some(name) => !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
        None => false,
    }
}

/// The event on a line, or None if the line is not a valid event.
pub fn parse(line: &str) -> Option<Event> {
    let mut f: Vec<&str> = line.split(' ').collect();
    let (kind, n) = match f[0] {
        "receive" => (Kind::Receive, 4),
        "ship" => (Kind::Ship, 3),
        "count" => (Kind::Count, 3),
        "move" => (Kind::Move, 5),
        _ => return None,
    };
    let mut loc = "@main".to_string();
    let mut to = String::new();
    if kind == Kind::Move {
        if f.len() != 5 || !valid_loc(f[3]) || !valid_loc(f[4]) {
            return None;
        }
        loc = f[3].to_string();
        to = f[4].to_string();
    } else if f.len() == n + 1 && valid_loc(f[n]) {
        loc = f[n].to_string();
        f.pop();
    } else if f.len() != n {
        return None;
    }
    if !valid_sku(f[1]) {
        return None;
    }
    let qty = number(f[2])?;
    let price = if kind == Kind::Receive { number(f[3])? } else { 0 };
    if qty == 0 && kind != Kind::Count {
        return None;
    }
    Some(Event { kind, sku: f[1].to_string(), qty, price, loc, to })
}
