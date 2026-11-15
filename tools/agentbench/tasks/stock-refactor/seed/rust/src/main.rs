use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;
use std::process::exit;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Receive,
    Ship,
    Count,
}

#[derive(Debug, Clone)]
struct Event {
    kind: Kind,
    sku: String,
    qty: i64,
    price: i64,
}

/// 1 to 5 ASCII digits, as a number.
fn number(s: &str) -> Option<i64> {
    if s.is_empty() || s.len() > 5 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn valid_sku(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}

/// The event on a line, or None if the line is not a valid event.
fn parse(line: &str) -> Option<Event> {
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
    let qty = number(f[2])?;
    let price = if kind == Kind::Receive { number(f[3])? } else { 0 };
    if qty == 0 && kind != Kind::Count {
        return None;
    }
    Some(Event { kind, sku: f[1].to_string(), qty, price })
}

#[derive(Debug, Clone, Default)]
struct Entry {
    qty: i64,
    cost: i64,
}

enum Problem {
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

struct Store {
    items: BTreeMap<String, Entry>,
}

impl Store {
    fn new() -> Store {
        Store { items: BTreeMap::new() }
    }

    /// Every SKU ever received, in ascending byte order, with its item.
    fn items(&self) -> impl Iterator<Item = (&String, &Entry)> + '_ {
        self.items.iter()
    }

    /// Applies one event; a rejected event changes nothing.
    fn apply(&mut self, e: &Event) -> Result<(), Problem> {
        if e.kind == Kind::Receive {
            let it = self.items.entry(e.sku.clone()).or_default();
            it.qty += e.qty;
            it.cost += e.qty * e.price;
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

/// Cents as dollars with two decimals: `1234.05`, `-0.50`.
fn money(c: i64) -> String {
    let sign = if c < 0 { "-" } else { "" };
    let a = c.unsigned_abs();
    format!("{}{}.{:02}", sign, a / 100, a % 100)
}

fn report(s: &Store) -> Vec<String> {
    s.items().map(|(sku, it)| format!("{}\t{}\t{}", sku, it.qty, money(it.cost))).collect()
}

fn low(s: &Store, n: i64) -> Vec<String> {
    s.items().filter(|(_, it)| it.qty < n).map(|(sku, _)| sku.clone()).collect()
}

fn total(s: &Store) -> Vec<String> {
    vec![money(s.items().map(|(_, it)| it.cost).sum())]
}

enum Command {
    Report,
    Low(i64),
    Total,
}

fn command(args: &[String]) -> Option<Command> {
    match (args.get(1).map(String::as_str), args.len()) {
        (Some("report"), 2) => Some(Command::Report),
        (Some("total"), 2) => Some(Command::Total),
        (Some("low"), 3) => number(&args[2]).map(Command::Low),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(cmd) = command(&args) else {
        eprintln!("usage: stock report|low N|total");
        exit(1);
    };
    let mut raw = Vec::new();
    std::io::stdin().read_to_end(&mut raw).expect("read stdin");
    let text = String::from_utf8_lossy(&raw);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut stock = Store::new();
    let mut bad = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let why = match parse(line) {
            None => Some("bad line".to_string()),
            Some(e) => stock.apply(&e).err().map(|p| p.to_string()),
        };
        if let Some(why) = why {
            eprintln!("error: line {}: {}", i + 1, why);
            bad += 1;
        }
    }
    let out = match cmd {
        Command::Report => report(&stock),
        Command::Low(n) => low(&stock, n),
        Command::Total => total(&stock),
    };
    for l in out {
        println!("{l}");
    }
    exit(if bad > 0 { 2 } else { 0 });
}
