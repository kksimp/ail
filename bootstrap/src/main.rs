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
mod parser;
mod stdlib;

use std::process::ExitCode;

const USAGE: &str = "\
usage: ail <command> [--json] [args]

commands:
  run FILE [ARGS...]
                check and run an AIL program; ARGS are the program's
                (std.env.1: `env.args()`)
  check FILE    check a program without running it
  version       print the version

options:
  --json        report diagnostics as JSON lines (for tools and AI agents)
";

/// Interpreter stack; deep AIL recursion traps at interp::MAX_DEPTH first.
const STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // std.env.1: the words after `run FILE` belong to the program.
    let words: Vec<usize> = (0..args.len()).filter(|&i| args[i] != "--json").collect();
    let (args, program_args) = match words.as_slice() {
        [cmd, file, ..] if args[*cmd] == "run" => (args[..=*file].to_vec(), args[*file + 1..].to_vec()),
        _ => (args, Vec::new()),
    };
    let json = args.iter().any(|a| a == "--json");
    let rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--json").collect();
    match rest.as_slice() {
        ["run", file] => {
            interp::set_program_args(program_args);
            run_file(file, json, true)
        }
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
    let (program, parse_diags) = parser::parse_with_prelude(tokens);
    diags.extend(parse_diags);
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
        diag::report(&mut diags, path, &src, json);
    }
    if rejected {
        return ExitCode::from(2);
    }
    if !execute {
        return ExitCode::SUCCESS;
    }

    let file = path.to_string();
    let result = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            interp::run(&program, &resolved).map_err(|stop| match stop {
                interp::Stop::Trap(t) => format!("{file}:{t}"),
                interp::Stop::Failed(report) => report.render(&file),
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
