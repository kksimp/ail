use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Write};
use std::process::exit;

fn valid_name(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'-')
}

fn reach<'a>(adj: &BTreeMap<&'a str, BTreeSet<&'a str>>, start: &'a str) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&str> = adj[start].iter().copied().collect();
    while let Some(x) = stack.pop() {
        if seen.insert(x) {
            stack.extend(adj[x].iter().copied());
        }
    }
    seen
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ok = (args.len() == 2 && (args[1] == "order" || args[1] == "cycles"))
        || (args.len() == 3 && (args[1] == "deps" || args[1] == "rdeps") && valid_name(&args[2]));
    if !ok {
        eprintln!("usage: depgraph order|cycles|deps NAME|rdeps NAME");
        exit(1);
    }
    let mut raw = Vec::new();
    io::stdin().read_to_end(&mut raw).expect("read stdin");
    let text = String::from_utf8_lossy(&raw);
    let mut needs: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut needed: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut bad = false;
    for (i, line) in text.split('\n').enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Option<(&str, &str)> = line.split_once(" -> ");
        match parts {
            Some((a, b)) if valid_name(a) && valid_name(b) => {
                needs.entry(a).or_default().insert(b);
                needs.entry(b).or_default();
                needed.entry(b).or_default().insert(a);
                needed.entry(a).or_default();
            }
            None if valid_name(line) => {
                needs.entry(line).or_default();
                needed.entry(line).or_default();
            }
            _ => {
                eprintln!("error: line {}: bad line", i + 1);
                bad = true;
            }
        }
    }
    let out = io::stdout();
    let mut out = out.lock();
    let mut rc = if bad { 2 } else { 0 };
    match args[1].as_str() {
        "deps" | "rdeps" => {
            let name = args[2].as_str();
            if !needs.contains_key(name) {
                eprintln!("error: unknown {}", name);
                exit(4);
            }
            let adj = if args[1] == "deps" { &needs } else { &needed };
            for n in reach(adj, name) {
                writeln!(out, "{}", n).unwrap();
            }
        }
        "cycles" => {
            for &n in needs.keys() {
                if reach(&needs, n).contains(n) {
                    writeln!(out, "{}", n).unwrap();
                }
            }
        }
        _ => {
            let mut left: BTreeMap<&str, usize> = needs.iter().map(|(k, v)| (*k, v.len())).collect();
            let mut ready: BTreeSet<&str> = left.iter().filter(|(_, &c)| c == 0).map(|(k, _)| *k).collect();
            let mut order = Vec::new();
            while let Some(n) = ready.iter().next().copied() {
                ready.remove(n);
                order.push(n);
                for &d in &needed[n] {
                    let c = left.get_mut(d).unwrap();
                    *c -= 1;
                    if *c == 0 {
                        ready.insert(d);
                    }
                }
            }
            if order.len() < needs.len() {
                eprintln!("error: cycle");
                rc = 3;
            } else {
                for n in order {
                    writeln!(out, "{}", n).unwrap();
                }
            }
        }
    }
    out.flush().unwrap();
    exit(rc);
}
