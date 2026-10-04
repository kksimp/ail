//! `ail`: the AIL command-line tool, bootstrap edition (ADR 0023).
//!
//! Exit status: 0 success, 1 the program failed or trapped (mod.main.2,
//! err.trap.3), 2 the program was rejected or the command was invalid.

mod ast;
mod check;
mod diag;
mod interp;
mod lexer;
mod parser;

use std::process::ExitCode;

const USAGE: &str = "\
usage: ail <command> [--json] [args]

commands:
  run FILE      check and run an AIL program
  check FILE    check a program without running it
  version       print the version

options:
  --json        report diagnostics as JSON lines (for tools and AI agents)
";

/// Interpreter stack; deep AIL recursion traps at interp::MAX_DEPTH first.
const STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--json").collect();
    match rest.as_slice() {
        ["run", file] => run_file(file, json, true),
        ["check", file] => run_file(file, json, false),
        ["version"] | ["--version"] => {
            println!("ail {} (bootstrap interpreter)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run_file(path: &str, json: bool, execute: bool) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };

    let (tokens, mut diags) = lexer::lex(&src);
    let (program, parse_diags) = parser::parse(tokens);
    diags.extend(parse_diags);
    if diags.is_empty() {
        diags = check::check(&program);
    }
    if !diags.is_empty() {
        diag::report(&mut diags, path, &src, json);
        return ExitCode::from(2);
    }
    if !execute {
        return ExitCode::SUCCESS;
    }

    let result = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || interp::run(&program).map_err(|t| t.to_string()))
        .expect("spawn interpreter thread")
        .join();
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(trap)) => {
            eprintln!("{path}: {trap}");
            ExitCode::from(1)
        }
        Err(_) => {
            eprintln!("{path}: internal error in the bootstrap interpreter");
            ExitCode::from(1)
        }
    }
}
