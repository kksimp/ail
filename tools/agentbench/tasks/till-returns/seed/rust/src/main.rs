//! till: a shop's back office. Reads the journal on stdin and prints one report (spec.md).
mod catalog;
mod cli;
mod customers;
mod fields;
mod gifts;
mod journal;
mod money;
mod orders;
mod pricing;
mod promos;
mod purchasing;
mod reports;
mod shop;

use cli::Command;
use std::io::{Read, Write};
use std::process::exit;

/// Prints `error: no WHAT` and ends the program with exit code 3.
fn missing(what: &str) -> ! {
    eprintln!("error: no {}", what);
    exit(3)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = cli::parse(&args);
    if let Command::Usage = cmd {
        eprintln!("{}", cli::usage());
        exit(1);
    }
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).expect("read stdin");
    let text = String::from_utf8_lossy(&input);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut s = shop::Shop::new();
    let mut bad = 0;
    for (k, line) in lines.iter().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let result = match journal::parse(line) {
            Ok(e) => s.apply(e).map_err(|p| p.to_string()),
            Err(_) => Err("bad line".to_string()),
        };
        if let Err(why) = result {
            eprintln!("error: line {}: {}", k + 1, why);
            bad += 1;
        }
    }
    let out = match &cmd {
        Command::ShowOrders => reports::order_lines(&s),
        Command::Invoice(id) => reports::invoice(&s, id).unwrap_or_else(|| missing(&format!("order {}", id))),
        Command::ShowStock => reports::stock_lines(&s),
        Command::Low(n) => reports::low(&s, *n),
        Command::ShowCustomers => reports::customer_lines(&s),
        Command::History(cust) => reports::history(&s, cust).unwrap_or_else(|| missing(&format!("customer {}", cust))),
        Command::Sales => reports::sales(&s),
        Command::Margin => reports::margin(&s),
        Command::Purchases => reports::purchase_lines(&s),
        Command::Gifts => reports::gift_lines(&s),
        Command::ShowPromos => reports::promo_lines(&s),
        Command::Usage => Vec::new(),
    };
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    for l in out {
        writeln!(w, "{}", l).expect("write stdout");
    }
    w.flush().expect("flush stdout");
    if bad > 0 {
        exit(2);
    }
}
