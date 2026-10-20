//! `ail`: the AIL command-line tool, bootstrap edition (ADR 0023).
//!
//! Exit status: 0 success, 1 the program failed or trapped (mod.main.2,
//! err.trap.3), 2 the program was rejected or the command was invalid.

mod ast;
mod check;
mod diag;
mod implicit_self;
mod interp;
mod lexer;
mod modules;
mod parser;

use std::process::ExitCode;

const USAGE: &str = "\
usage: ail <command> [--json] [args]

commands:
  run FILE      check and run an AIL program (with the modules it imports)
  run DIR       check and run the package whose root module is DIR
  check FILE    check a program (or package directory) without running it
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
    // A single file, or a package of modules (mod.unit.1-5).
    let (program, mut diags) = match modules::load(path) {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    // A string literal with an error throws off the rest of its line, so
    // report only the string's own errors there.
    let bad_lines: Vec<usize> = diags.iter().filter(|d| d.rule.starts_with("lex.str.")).map(|d| d.pos.line).collect();
    diags.retain(|d| d.rule.starts_with("lex.str.") || !bad_lines.contains(&d.pos.line));
    let mut resolved = check::Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&program);
    }
    let rejected = diags.iter().any(|d| !d.warning);
    if !diags.is_empty() {
        modules::report(&program.files, diags, json);
    }
    if rejected {
        return ExitCode::from(2);
    }
    if !execute {
        return ExitCode::SUCCESS;
    }

    let result = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            let at = |pos| {
                let (file, p) = modules::locate(&program.files, pos);
                format!("{file}:{}:{}", p.line, p.col)
            };
            interp::run(&program, &resolved).map_err(|stop| match stop {
                interp::Stop::Trap(t) => format!("{}: trap[{}]: {}", at(t.pos), t.rule, t.message),
                interp::Stop::Failed(report) => report.render_with(at),
            })
        })
        .expect("spawn interpreter thread")
        .join();
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(message)) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
        Err(_) => {
            eprintln!("{path}: internal error in the bootstrap interpreter");
            ExitCode::from(1)
        }
    }
}
