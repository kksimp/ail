//! The shapes of journal and command-line fields (spec "Fields").

fn upper(c: u8) -> bool {
    c.is_ascii_uppercase()
}

fn lower(c: u8) -> bool {
    c.is_ascii_lowercase()
}

fn digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn sku_char(c: u8) -> bool {
    upper(c) || digit(c) || c == b'-'
}

fn code_char(c: u8) -> bool {
    upper(c) || digit(c)
}

fn id_char(c: u8) -> bool {
    lower(c) || digit(c)
}

/// `lo` to `hi` bytes, the first accepted by `first` and every one by `rest`.
fn shaped(s: &str, lo: usize, hi: usize, first: fn(u8) -> bool, rest: fn(u8) -> bool) -> bool {
    let b = s.as_bytes();
    b.len() >= lo && b.len() <= hi && first(b[0]) && b.iter().all(|&c| rest(c))
}

/// 1 to `width` ASCII digits, as a number (leading zeros allowed); `None` for anything else.
pub fn digits(s: &str, width: usize) -> Option<i64> {
    if s.is_empty() || s.len() > width || !s.bytes().all(digit) {
        return None;
    }
    s.parse().ok()
}

/// SKU: 1 to 12 of `A-Z 0-9 -`, starting with a letter.
pub fn is_sku(s: &str) -> bool {
    shaped(s, 1, 12, upper, sku_char)
}

/// CAT: 1 to 12 letters `a-z`.
pub fn is_cat(s: &str) -> bool {
    shaped(s, 1, 12, lower, lower)
}

/// ID: 1 to 12 of `a-z 0-9`, starting with a letter.
pub fn is_id(s: &str) -> bool {
    shaped(s, 1, 12, lower, id_char)
}

/// CODE: 3 to 10 of `A-Z 0-9`, starting with a letter.
pub fn is_code(s: &str) -> bool {
    shaped(s, 3, 10, upper, code_char)
}

fn prefixed(s: &str, p: &str) -> bool {
    match s.strip_prefix(p) {
        Some(rest) => digits(rest, 6).is_some(),
        None => false,
    }
}

/// ORDER: `o` and 1 to 6 digits.
pub fn is_order(s: &str) -> bool {
    prefixed(s, "o")
}

/// PO: `p` and 1 to 6 digits.
pub fn is_po(s: &str) -> bool {
    prefixed(s, "p")
}

/// CARD: `g` and 1 to 6 digits.
pub fn is_card(s: &str) -> bool {
    prefixed(s, "g")
}

/// TIER: `basic`, `silver` or `gold`.
pub fn is_tier(s: &str) -> bool {
    s == "basic" || s == "silver" || s == "gold"
}
