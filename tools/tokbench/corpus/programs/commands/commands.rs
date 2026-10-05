use std::fmt;

enum CommandError {
    Empty,
    UnknownVerb { verb: String },
    MissingArgument { verb: String },
    BadQuantity { text: String },
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::Empty => write!(f, "CommandError.Empty"),
            CommandError::UnknownVerb { verb } => write!(f, "CommandError.UnknownVerb(verb: {verb:?})"),
            CommandError::MissingArgument { verb } => write!(f, "CommandError.MissingArgument(verb: {verb:?})"),
            CommandError::BadQuantity { text } => write!(f, "CommandError.BadQuantity(text: {text:?})"),
        }
    }
}

enum Command {
    Add { item: String, qty: i64 },
    Remove { item: String },
    Show,
}

struct Entry {
    item: String,
    qty: i64,
}

fn parse(line: &str) -> Result<Command, CommandError> {
    let words: Vec<&str> = line.split(' ').collect();
    let verb = words[0];
    if verb.is_empty() {
        return Err(CommandError::Empty);
    }
    if verb == "show" {
        return Ok(Command::Show);
    }
    if words.len() < 2 {
        return Err(CommandError::MissingArgument { verb: verb.to_string() });
    }
    if verb == "remove" {
        return Ok(Command::Remove { item: words[1].to_string() });
    }
    if verb != "add" {
        return Err(CommandError::UnknownVerb { verb: verb.to_string() });
    }
    if words.len() < 3 {
        return Err(CommandError::MissingArgument { verb: verb.to_string() });
    }
    let qty = words[2].parse().map_err(|_| CommandError::BadQuantity { text: words[2].to_string() })?;
    Ok(Command::Add { item: words[1].to_string(), qty })
}

fn apply(stock: &mut Vec<Entry>, cmd: Command) {
    match cmd {
        Command::Add { item, qty } => {
            for entry in stock.iter_mut() {
                if entry.item == item {
                    entry.qty += qty;
                    return;
                }
            }
            stock.push(Entry { item, qty });
        }
        Command::Remove { item } => {
            for entry in stock.iter_mut() {
                if entry.item == item {
                    entry.qty = 0;
                }
            }
        }
        Command::Show => {
            for entry in stock.iter() {
                if entry.qty > 0 {
                    println!("  {}: {}", entry.item, entry.qty);
                }
            }
        }
    }
}

fn total_added(lines: &[&str]) -> Result<i64, String> {
    let mut total = 0;
    for line in lines {
        let cmd = parse(line).map_err(|e| format!("reading {line:?}: {e}"))?;
        if let Command::Add { qty, .. } = cmd {
            total += qty;
        }
    }
    Ok(total)
}

fn main() -> Result<(), String> {
    let lines = [
        "add milk 2",
        "add eggs 12",
        "add milk 1",
        "remove eggs",
        "add tea lots",
        "sell milk 1",
        "",
        "show",
    ];
    let mut stock = vec![Entry { item: "bread".to_string(), qty: 1 }];
    for line in lines {
        let cmd = match parse(line) {
            Ok(cmd) => cmd,
            Err(err) => {
                println!("error: {err}");
                continue;
            }
        };
        apply(&mut stock, cmd);
    }
    println!("added: {}", total_added(&lines[0..4])?);
    Ok(())
}
