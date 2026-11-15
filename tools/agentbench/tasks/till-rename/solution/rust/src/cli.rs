//! The command line: which report to print (spec "Reports", "Exit codes and stderr").
use crate::fields;

pub enum Command {
    ShowOrders,
    Invoice(String),
    ShowStock,
    Low(i64),
    ShowAccounts,
    History(String),
    Sales,
    Margin,
    Purchases,
    Gifts,
    ShowPromos,
    /// Missing, unknown or malformed: print the usage line and exit 1.
    Usage,
}

pub fn usage() -> &'static str {
    "usage: till orders|invoice ORDER|stock|low N|accounts|history ID|sales|margin|purchases|gifts|promos"
}

/// The command the arguments ask for (the program name is not one of them).
pub fn parse(args: &[String]) -> Command {
    match args {
        [one] => match one.as_str() {
            "orders" => Command::ShowOrders,
            "stock" => Command::ShowStock,
            "accounts" => Command::ShowAccounts,
            "sales" => Command::Sales,
            "margin" => Command::Margin,
            "purchases" => Command::Purchases,
            "gifts" => Command::Gifts,
            "promos" => Command::ShowPromos,
            _ => Command::Usage,
        },
        [cmd, arg] => match cmd.as_str() {
            "invoice" if fields::is_order(arg) => Command::Invoice(arg.clone()),
            "history" if fields::is_id(arg) => Command::History(arg.clone()),
            "low" => match fields::digits(arg, 5) {
                Some(n) => Command::Low(n),
                None => Command::Usage,
            },
            _ => Command::Usage,
        },
        _ => Command::Usage,
    }
}
