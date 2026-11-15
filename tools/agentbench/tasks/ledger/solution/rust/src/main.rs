use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::process::exit;

struct Row<'a> {
    date: &'a str,
    account: &'a str,
    memo: &'a str,
    amount: i64,
}

fn money(c: i128) -> String {
    let sign = if c < 0 { "-" } else { "" };
    let a = c.unsigned_abs();
    format!("{}{}.{:02}", sign, a / 100, a % 100)
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn valid_date(d: &str) -> bool {
    let b = d.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    if !all_digits(&d[0..4]) || !all_digits(&d[5..7]) || !all_digits(&d[8..10]) {
        return false;
    }
    let m: u32 = d[5..7].parse().unwrap();
    let day: u32 = d[8..10].parse().unwrap();
    (1..=12).contains(&m) && (1..=31).contains(&day)
}

fn usage() -> ! {
    eprintln!("usage: ledger balance|top N|monthly");
    exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut top: Option<u64> = None;
    match args.len() {
        2 if args[1] == "balance" || args[1] == "monthly" => {}
        3 if args[1] == "top" && all_digits(&args[2]) => top = Some(args[2].parse().unwrap_or(u64::MAX)),
        _ => usage(),
    }
    let mut raw = Vec::new();
    io::stdin().read_to_end(&mut raw).expect("read stdin");
    let text = String::from_utf8_lossy(&raw);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    if lines.is_empty() || lines[0] != "date,account,amount_cents,memo" {
        eprintln!("error: line 1: bad header");
        exit(2);
    }
    let mut rows = Vec::new();
    let mut bad = false;
    for (i, line) in lines.iter().enumerate().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let why = if f.len() != 4 {
            Some("expected 4 fields")
        } else if !valid_date(f[0]) {
            Some("bad date")
        } else if f[1].is_empty() {
            Some("empty account")
        } else if !all_digits(f[2].strip_prefix('-').unwrap_or(f[2])) || f[2].parse::<i64>().is_err() {
            Some("bad amount")
        } else {
            None
        };
        match why {
            Some(w) => {
                eprintln!("error: line {}: {}", i + 1, w);
                bad = true;
            }
            None => rows.push(Row { date: f[0], account: f[1], memo: f[3], amount: f[2].parse().unwrap() }),
        }
    }
    let out = io::stdout();
    let mut out = out.lock();
    if let Some(n) = top {
        rows.sort_by_key(|r| std::cmp::Reverse(r.amount.unsigned_abs())); // stable: ties keep input order
        for r in rows.iter().take(n.min(usize::MAX as u64) as usize) {
            writeln!(out, "{}\t{}\t{}\t{}", r.date, r.account, money(r.amount as i128), r.memo).unwrap();
        }
    } else {
        let by_account = args[1] == "balance";
        let mut tot: BTreeMap<&str, i128> = BTreeMap::new();
        for r in &rows {
            let key = if by_account { r.account } else { &r.date[0..7] };
            *tot.entry(key).or_insert(0) += r.amount as i128;
        }
        for (k, v) in &tot {
            writeln!(out, "{}\t{}", k, money(*v)).unwrap();
        }
    }
    out.flush().unwrap();
    exit(if bad { 2 } else { 0 });
}
