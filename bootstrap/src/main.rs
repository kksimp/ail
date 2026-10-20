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
mod doctest;
mod docs;
mod fix;
mod fmt;
mod implicit_self;
mod interp;
mod lexer;
mod modules;
mod num;
mod parser;
mod stdlib;

use std::process::ExitCode;

const USAGE: &str = "\
usage: ail <command> [--json] [args]

commands:
  run FILE [ARGS...]
                check and run an AIL program, with the modules it imports;
                ARGS are the program's (std.env.1: `env.args()`)
  run DIR [ARGS...]
                check and run the package whose root module is DIR
  check FILE    check a program (or a package DIR) without running it
  test FILE     run the program's `@test` functions, properties and doc examples;
                --cases N (default 100) and --seed S set a property's inputs
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
  --deterministic
                with run or check: same as `--deny clock,random,proc,net`, so
                the same inputs always give the same output (std.random.5)
  --seed N      with run: seed `std.random` so a run repeats (std.random.3)
  --deny LIST   with run or check: reject a program whose `main` reaches a
                denied capability (fs, env, proc, clock, random, net, or all) (eff.cap.1)
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
            "--json" | "--deterministic" => {}
            "--deny" | "--seed" => i += 1,
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
    // `ail test` owns `--cases` and `--seed` (test.prop.3); `ail run` owns
    // `--seed` and `--deterministic` (std.random.3, eff.det.1).
    let mut flags: Vec<&str> = Vec::new();
    if rest.first() == Some(&"test") {
        for name in ["--cases", "--seed"] {
            if let Some(at) = rest.iter().position(|a| *a == name) {
                flags.extend(rest.drain(at..(at + 2).min(rest.len())));
            }
        }
    }
    let deny = match take_deny(&mut rest).and_then(|d| take_run_options(&mut rest, d)) {
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
        ["test", file] => match take_test_options(&mut flags) {
            Ok((cases, seed)) => run_tests(file, json, cases, seed),
            Err(msg) => {
                eprintln!("error: {msg}");
                ExitCode::from(2)
            }
        },
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

/// `--cases N` and `--seed S` for `ail test` (test.prop.3).
fn take_test_options(flags: &mut Vec<&str>) -> Result<(u64, Option<u64>), String> {
    let (mut cases, mut seed) = (100, None);
    for pair in flags.chunks(2) {
        let [name, value] = pair else { return Err(format!("`{}` needs a number", pair[0])) };
        let n: u64 = value.parse().map_err(|_| format!("`{name}` needs a non-negative number, found `{value}`"))?;
        if *name == "--cases" {
            cases = n;
        } else {
            seed = Some(n);
        }
    }
    Ok((cases, seed))
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

/// Remove `--deterministic` and `--seed S`, and return the denied
/// capabilities including those `--deterministic` adds (std.random.3,
/// std.random.5).
fn take_run_options(args: &mut Vec<&str>, mut deny: Vec<&'static str>) -> Result<Vec<&'static str>, String> {
    if let Some(at) = args.iter().position(|a| *a == "--seed") {
        let seed = args.get(at + 1).and_then(|s| s.parse::<i64>().ok());
        let Some(seed) = seed else { return Err("`--seed` needs an integer, e.g. `--seed 42` (std.random.3)".into()) };
        args.drain(at..at + 2);
        interp::set_seed(seed);
    }
    if let Some(at) = args.iter().position(|a| *a == "--deterministic") {
        args.remove(at);
        deny.extend(check::effects::DETERMINISTIC_DENY);
        deny.sort_unstable();
        deny.dedup();
    }
    Ok(deny)
}

/// `ail effects FILE [--json]`: each function's effect set (eff.infer.6).
fn effects_file(path: &str, json: bool) -> ExitCode {
    let Some((program, resolved)) = front_end(path, json) else { return ExitCode::from(2) };
    print!("{}", check::effects::render(&check::effects::effects_of(&program, &resolved), json));
    ExitCode::SUCCESS
}

/// Read and parse `path`, a file or a package directory, with the modules
/// it imports (mod.unit.1-5). `None` if it can't be read.
fn load(path: &str) -> Option<(ast::Program, Vec<diag::Diagnostic>)> {
    let (program, mut diags) = match modules::load(path) {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };
    // A string literal with an error throws off the rest of its line, so
    // report only the string's own errors there.
    let bad_lines: Vec<usize> = diags.iter().filter(|d| d.rule.starts_with("lex.str.")).map(|d| d.pos.line).collect();
    diags.retain(|d| d.rule.starts_with("lex.str.") || !bad_lines.contains(&d.pos.line));
    Some((program, diags))
}

/// Lex, parse and check `path`, reporting diagnostics; `None` if the
/// program is rejected.
fn front_end(path: &str, json: bool) -> Option<(ast::Program, check::Resolved)> {
    let (program, mut diags) = load(path)?;
    let mut resolved = check::Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&program);
    }
    let rejected = diags.iter().any(|d| !d.warning);
    if !diags.is_empty() {
        modules::report(&program.files, diags, json);
    }
    (!rejected).then_some((program, resolved))
}

fn run_file(path: &str, json: bool, execute: bool, deny: &[&str]) -> ExitCode {
    let Some((program, mut diags)) = load(path) else { return ExitCode::from(2) };
    let mut resolved = check::Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&program);
        if !deny.is_empty() {
            diags.extend(check::effects::check_denied(&program, &resolved, deny));
        }
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

/// `ail test FILE` (test.run.1-4): check, run each `@test` function on its
/// own, report in source order. Exit 0 if all pass, 1 if any fails, 2 if
/// the program is rejected. Doc examples run after the file's own tests
/// (test.doc.1), and properties run `cases` inputs (test.prop.3).
fn run_tests(path: &str, json: bool, cases: u64, seed: Option<u64>) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    // The file's own module plus what it imports (mod.unit.1-5).
    let (mut program, mut diags) = modules::load_src(path, &src);
    // test.doc.1: each ```ail block in a doc comment becomes a test.
    let mut doc_tests = Vec::new();
    let mut line_map = Vec::new();
    if diags.is_empty() {
        doc_tests = doctest::collect(&src, &program);
        if !doc_tests.is_empty() {
            let (augmented, map) = doctest::augment(&src, &doc_tests);
            line_map = map;
            let (p, pd) = modules::load_src(path, &augmented);
            program = p;
            diags.extend(pd);
        }
    }
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
    // test.doc.3: a problem in an example is reported at the example's own line.
    for d in &mut diags {
        if let Some(&line) = line_map.get(d.pos.line.wrapping_sub(1)).filter(|l| **l != 0) {
            d.pos.line = line;
        }
    }
    let rejected = diags.iter().any(|d| !d.warning);
    if !diags.is_empty() {
        modules::report(&program.files, diags, json);
    }
    if rejected {
        return ExitCode::from(2);
    }
    let outcomes = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            interp::run_tests(&program, &resolved, cases, seed)
                .into_iter()
                .map(|o| (o.name, o.pos, o.failure, o.output, o.prop))
                .collect::<Vec<_>>()
        })
        .expect("spawn interpreter thread")
        .join();
    let Ok(mut outcomes) = outcomes else {
        eprintln!("{path}: internal error in the bootstrap interpreter");
        return ExitCode::from(1);
    };
    for o in &mut outcomes {
        let Some(t) = doc_tests.iter().find(|t| t.fn_name == o.0) else { continue };
        // test.doc.2-3: report under the example's name, at its own line.
        o.0 = t.name.clone();
        if o.2.is_none() {
            o.2 = doctest::check_output(t, &o.3);
            o.1 = diag::Pos { line: t.line(), col: 1 };
        } else if let Some(&line) = line_map.get(o.1.line.wrapping_sub(1)).filter(|l| **l != 0) {
            o.1.line = line;
        }
    }
    let failed = outcomes.iter().filter(|o| o.2.is_some()).count();
    let passed = outcomes.len() - failed;
    for (name, pos, failure, output, prop) in &outcomes {
        if json {
            let (status, message) = match failure {
                Some(m) => ("fail", m.as_str()),
                None => ("pass", ""),
            };
            let mut line = format!(
                "{{\"test\":{},\"status\":\"{status}\",\"line\":{},\"col\":{},\"message\":{}",
                diag::json_str(name),
                pos.line,
                pos.col,
                diag::json_str(message)
            );
            if let Some(p) = prop {
                line.push_str(&format!(",\"cases\":{},\"seed\":{}", p.cases, p.seed));
                if let Some(input) = &p.input {
                    line.push_str(&format!(",\"input\":{}", diag::json_str(input)));
                }
            }
            println!("{line}}}");
        } else {
            let tail = match prop {
                Some(p) => format!(" ({} cases, seed {})", p.cases, p.seed),
                None => String::new(),
            };
            match failure {
                None => println!("ok {name}{tail}"),
                Some(m) => {
                    println!("FAIL {name}: {m} at {}:{}{tail}", pos.line, pos.col);
                    if let Some(input) = prop.as_ref().and_then(|p| p.input.as_ref()) {
                        println!("  input: {input}");
                    }
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
