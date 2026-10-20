//! `ail`: the AIL command-line tool, bootstrap edition (ADR 0023).
//!
//! Exit status: 0 success, 1 the program failed or trapped (mod.main.2,
//! err.trap.3), 2 the program was rejected or the command was invalid.

mod api;
mod ast;
mod check;
mod diag;
mod explain;
mod fasthash;
mod docs;
mod fix;
mod fmt;
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
  test FILE     run the program's `@test` functions
  effects FILE  print each function's inferred effects (eff.infer.6)
  explain RULE  print a spec rule, its ADRs and an example
  api FILE      print the program's declarations as JSON (--pretty:
                indented; --text: an outline; --symbols: every reference)
  fmt PATH...   rewrite files (directories: every .ail file) in canonical layout;
                --check: exit 1 with a diff if any would change; --stdout: print instead
  fix FILE      apply the fixes diagnostics carry, re-check, repeat (up to 5
                rounds); --dry-run: print the diff instead of writing
  version       print the version

options:
  --json        report diagnostics (or an explanation) as JSON lines, schema v1
  --deny LIST   with run or check: reject a program whose `main` reaches a
                denied capability (fs, env, proc, clock, net, or all) (eff.cap.1)
";

/// Interpreter stack; deep AIL recursion traps at interp::MAX_DEPTH first.
const STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // std.env.1: the words after `run FILE` belong to the program; `ail`'s
    // own options come before FILE.
    let mut words = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {}
            "--deny" => i += 1,
            _ => words.push(i),
        }
        i += 1;
    }
    let (args, program_args) = match words.as_slice() {
        [cmd, file, ..] if args[*cmd] == "run" => (args[..=*file].to_vec(), args[*file + 1..].to_vec()),
        _ => (args, Vec::new()),
    };
    let json = args.iter().any(|a| a == "--json");
    let mut rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--json").collect();
    let deny = match take_deny(&mut rest) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("error: {msg}");
            return ExitCode::from(2);
        }
    };
    match rest.as_slice() {
        ["run", file] => {
            interp::set_program(file, program_args);
            run_file(file, json, true, &deny)
        }
        ["check", file] => run_file(file, json, false, &deny),
        ["test", file] => run_tests(file, json),
        ["effects", file] if deny.is_empty() => effects_file(file, json),
        ["fix", rest @ ..] => fix::command(rest, json),
        ["explain", rule] => explain::run(rule, json),
        ["api", rest @ ..] => api::command(rest, json),
        ["fmt", paths @ ..] if !paths.is_empty() => fmt::command(paths, json),
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

/// Remove `--deny LIST` from the arguments and return the denied
/// capabilities (eff.cap.2).
fn take_deny(args: &mut Vec<&str>) -> Result<Vec<&'static str>, String> {
    let Some(at) = args.iter().position(|a| *a == "--deny") else { return Ok(Vec::new()) };
    let Some(list) = args.get(at + 1).copied() else {
        return Err("`--deny` needs a list of capabilities, e.g. `--deny fs,proc`".into());
    };
    args.drain(at..at + 2);
    check::effects::parse_deny(list)
}

/// `ail effects FILE [--json]`: each function's effect set (eff.infer.6).
fn effects_file(path: &str, json: bool) -> ExitCode {
    let Some((program, resolved)) = front_end(path, json) else { return ExitCode::from(2) };
    print!("{}", check::effects::render(&check::effects::effects_of(&program, &resolved), json));
    ExitCode::SUCCESS
}

/// Lex, parse and check `path`, reporting diagnostics; `None` if the
/// program is rejected.
fn front_end(path: &str, json: bool) -> Option<(ast::Program, check::Resolved)> {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return None;
        }
    };
    let (tokens, mut diags) = lexer::lex(&src);
    let (program, parse_diags) = parser::parse_with_prelude(tokens);
    diags.extend(parse_diags);
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
    (!rejected).then_some((program, resolved))
}

fn run_file(path: &str, json: bool, execute: bool, deny: &[&str]) -> ExitCode {
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
        if !deny.is_empty() {
            diags.extend(check::effects::check_denied(&program, &resolved, deny));
        }
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

/// `ail test FILE` (test.run.1-4): check, run each `@test` function on its
/// own, report in source order. Exit 0 if all pass, 1 if any fails, 2 if
/// the program is rejected.
fn run_tests(path: &str, json: bool) -> ExitCode {
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
    let bad_lines: Vec<usize> = diags.iter().filter(|d| d.rule.starts_with("lex.str.")).map(|d| d.pos.line).collect();
    diags.retain(|d| d.rule.starts_with("lex.str.") || !bad_lines.contains(&d.pos.line));
    let mut resolved = check::Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&program);
    }
    // test.run.1: a file of tests needs no `main`.
    if !program.fns.iter().any(|f| f.name == "main" && f.owner.is_none() && f.top_level()) {
        diags.retain(|d| d.rule != "mod.main.1" || d.pos.line != 1);
    }
    let rejected = diags.iter().any(|d| !d.warning);
    if !diags.is_empty() {
        diag::report(&mut diags, path, &src, json);
    }
    if rejected {
        return ExitCode::from(2);
    }
    let outcomes = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            interp::run_tests(&program, &resolved)
                .into_iter()
                .map(|o| (o.name, o.pos, o.failure, o.output))
                .collect::<Vec<_>>()
        })
        .expect("spawn interpreter thread")
        .join();
    let Ok(outcomes) = outcomes else {
        eprintln!("{path}: internal error in the bootstrap interpreter");
        return ExitCode::from(1);
    };
    let failed = outcomes.iter().filter(|o| o.2.is_some()).count();
    let passed = outcomes.len() - failed;
    for (name, pos, failure, output) in &outcomes {
        if json {
            let (status, message) = match failure {
                Some(m) => ("fail", m.as_str()),
                None => ("pass", ""),
            };
            println!(
                "{{\"test\":{},\"status\":\"{status}\",\"line\":{},\"col\":{},\"message\":{}}}",
                diag::json_str(name),
                pos.line,
                pos.col,
                diag::json_str(message)
            );
        } else {
            match failure {
                None => println!("ok {name}"),
                Some(m) => {
                    println!("FAIL {name}: {m} at {}:{}", pos.line, pos.col);
                    for line in output.lines() {
                        println!("  | {line}");
                    }
                }
            }
        }
    }
    if json {
        println!("{{\"summary\":true,\"passed\":{passed},\"failed\":{failed}}}");
    } else {
        println!("{passed} passed, {failed} failed");
    }
    if failed == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
