mod event;
mod report;
mod store;

use std::io::Read;
use std::process::exit;

enum Command {
    Report,
    Low(i64),
    Total,
    Where(String),
}

fn command(args: &[String]) -> Option<Command> {
    match (args.get(1).map(String::as_str), args.len()) {
        (Some("report"), 2) => Some(Command::Report),
        (Some("total"), 2) => Some(Command::Total),
        (Some("low"), 3) => event::number(&args[2]).map(Command::Low),
        (Some("where"), 3) if event::valid_sku(&args[2]) => Some(Command::Where(args[2].clone())),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(cmd) = command(&args) else {
        eprintln!("usage: stock report|low N|total|where SKU");
        exit(1);
    };
    let mut raw = Vec::new();
    std::io::stdin().read_to_end(&mut raw).expect("read stdin");
    let text = String::from_utf8_lossy(&raw);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut stock = store::Store::new();
    let mut bad = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let why = match event::parse(line) {
            None => Some("bad line".to_string()),
            Some(e) => stock.apply(&e).err().map(|p| p.to_string()),
        };
        if let Some(why) = why {
            eprintln!("error: line {}: {}", i + 1, why);
            bad += 1;
        }
    }
    let out = match cmd {
        Command::Report => report::report(&stock),
        Command::Low(n) => report::low(&stock, n),
        Command::Total => report::total(&stock),
        Command::Where(sku) => match report::where_is(&stock, &sku) {
            Some(lines) => lines,
            None => {
                eprintln!("error: unknown {sku}");
                exit(4);
            }
        },
    };
    for l in out {
        println!("{l}");
    }
    exit(if bad > 0 { 2 } else { 0 });
}
