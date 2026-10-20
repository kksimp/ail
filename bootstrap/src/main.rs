//! `ail`: the AIL command-line tool, bootstrap edition (ADR 0023).
//!
//! Exit status: 0 success, 1 the program failed or trapped (mod.main.2,
//! err.trap.3), 2 the program was rejected or the command was invalid.

mod api;
mod ast;
mod budget;
mod bump;
mod check;
mod derive;
mod diag;
mod explain;
mod fasthash;
mod doctest;
mod docs;
mod fix;
mod foreign;
mod fmt;
mod implicit_self;
mod interp;
mod lexer;
mod modules;
mod codec;
mod net;
mod num;
mod packages;
mod parser;
mod stdlib;
mod tensor;
mod ui;

use std::process::ExitCode;

const USAGE: &str = "\
usage: ail <command> [--json] [args]

commands:
  run FILE [ARGS...]
                check and run an AIL program, with the modules it imports;
                ARGS are the program's (std.env.1: `env.args()`)
  run DIR [ARGS...]
                check and run the package whose root module is DIR
  check FILE    check a program (or a package DIR) without running it;
                --errors: each function's error types and failure paths as JSON
  test FILE     run the program's `@test` functions, properties and doc examples;
                --cases N (default 100) and --seed S set a property's inputs
  effects FILE  print each function's inferred effects (eff.infer.6);
                --parallel: each `for` loop's parallel verdict instead (conc.par.6)
  explain RULE  print a spec rule, its ADRs and an example
  explain       list every rule with its chapter and whether `ail fix` repairs it (--json)
  api FILE      print the program's declarations as JSON (--pretty:
                indented; --text: an outline; --symbols: every reference)
  api --card [--budget N] DIR|FILE
                a program's card: modules, signatures, tests, manifest, examples
  fmt PATH...   rewrite files (directories: every .ail file) in canonical layout;
                --check: exit 1 with a diff if any would change; --stdout: print instead
  fix FILE      apply the fixes diagnostics carry, re-check, repeat (up to 5
                rounds); --dry-run: print the diff instead of writing
  pkg update [NAME...]
                in a package root: fetch the named git dependencies (all if
                none) again, ignoring `ail.lock`, and rewrite it (mod.pkg.9)
  version       print the version

every command:
  --budget [N]  show about N tokens of output (the command's default when N
                is left off): errors, failed tests and warnings first, then
                a note counting the rest; --from K continues (budget.out.1-5)

options:
  --json        report diagnostics (or an explanation) as JSON lines, schema v1
  --deterministic
                with run or check: same as `--deny clock,random,proc,net`, so
                the same inputs always give the same output (std.random.5),
                and runs every loop on one thread (conc.par.9)
  --parallel    with run: run every loop proven independent on the thread
                pool, not only `par for` (conc.par.8); AIL_THREADS=N sets its size
  --seed N      with run: seed `std.random` so a run repeats (std.random.3)
  --schedule=random[:SEED]
                with test or run: switch tasks at random points picked from
                SEED (a new one if left off), so a test that fails on one
                interleaving can be replayed; a failing test prints the
                seed (conc.sched.1-2)
  --deny LIST   with run or check: reject a program whose `main` reaches a
                denied capability (fs, env, proc, clock, random, net, ui,
                or all) (eff.cap.1)
  --no-contracts
                with run or test: skip `requires`/`ensures` checks (contract.run.3)
  --no-bytecode with run or test: run every function on the tree walker, not
                the bytecode tier (ADR 0361)
  --trace[=calls|effects|all]
                with run: write each call, return, effect, task and contract
                check to standard error (trace.run.1); --trace-limit N caps
                the events shown (default 1000) before a summary line; with
                test, only a failing test reports its events (trace.test.1)
  --watch NAME  with run or test: show each assignment to a `mut` variable NAME,
                through `mut` arguments and methods too (trace.watch.1, trace.watch.3)
  --verbose     with run: a trap shows every frame and value (trace.snap.3)
";

// ADR 0332: commands that only read a program use the front-end allocator.
#[global_allocator]
static ALLOC: bump::FrontEnd = bump::FrontEnd::new();

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
            "--json" | "--deterministic" | "--no-contracts" | "--no-bytecode" | "--parallel" => {}
            a if a == "--coverage" || a.starts_with("--coverage=") => {}
            a if a.starts_with("--schedule") => {}
            "--deny" | "--seed" | "--timeout" | "--max-memory" | "--max-steps" | "--from" => i += 1,
            // budget.out.5: `--budget` may stand alone (the command's default).
            "--budget" if args.get(i + 1).is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit())) => i += 1,
            "--budget" => {}
            a if interp::trace_option_words(a).is_some() => i += interp::trace_option_words(a).unwrap_or(1) - 1,
            _ => words.push(i),
        }
        i += 1;
    }
    let (args, program_args) = match words.as_slice() {
        [cmd, file, ..] if args[*cmd] == "run" => (args[..=*file].to_vec(), args[*file + 1..].to_vec()),
        _ => (args, Vec::new()),
    };
    let json = args.iter().any(|a| a == "--json");
    // budget.out.1: `--budget N [--from K]` runs the command as a child and
    // cuts its output.
    let mut args = args;
    match budget::take(&mut args) {
        Ok(Some(b)) => return budget::run(&b, &[args, program_args].concat(), json),
        Ok(None) => {}
        Err(msg) => {
            eprintln!("error: {msg}");
            return ExitCode::from(2);
        }
    }
    // contract.run.3: `--no-contracts` skips the runtime contract checks.
    interp::set_contracts(!args.iter().any(|a| a == "--no-contracts"));
    // ADR 0361: `--no-bytecode` runs every function on the tree walker, as
    // do `--deterministic` and `--schedule` for now.
    interp::set_bytecode(
        !args.iter().any(|a| a == "--no-bytecode" || a == "--deterministic" || a.starts_with("--schedule")),
    );
    let mut rest: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json" && *a != "--no-contracts" && *a != "--no-bytecode")
        .collect();
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
    // conc.sched.1: `--schedule=random[:SEED]` for `ail run` and `ail test`.
    if let Err(msg) = take_schedule(&mut rest) {
        eprintln!("error: {msg}");
        return ExitCode::from(2);
    }
    // run.limit.1-4, test.cover.1-3: limits for `ail run` and `ail test`, coverage for `ail test`.
    let coverage = match take_limit_options(&mut rest) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("error: {msg}");
            return ExitCode::from(2);
        }
    };
    // trace.run.1, trace.watch.1, trace.snap.3: `ail run`'s tracing options.
    match interp::take_trace_options(&mut rest, json) {
        Ok(true) if !matches!(rest.first(), Some(&"run" | &"test")) => {
            eprintln!("error: `--trace`, `--watch` and `--verbose` work with `ail run` and `ail test` (trace.run.1)");
            return ExitCode::from(2);
        }
        Ok(_) => {}
        Err(msg) => {
            eprintln!("error: {msg}");
            return ExitCode::from(2);
        }
    }
    let deny = match take_deny(&mut rest).and_then(|d| take_run_options(&mut rest, d)) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("error: {msg}");
            return ExitCode::from(2);
        }
    };
    // mod.pkg.7-8: `run` and `test` write `ail.lock`; `--deterministic`
    // refuses a stale one; `--deny net` fetches nothing.
    packages::git::configure(packages::git::Options {
        write_lock: matches!(rest.first(), Some(&"run" | &"test")),
        frozen: args.iter().any(|a| a == "--deterministic"),
        offline: deny.contains(&"net"),
        refresh: None,
    });
    // ADR 0332: `check`, `api` and `effects` make one pass over the program
    // and exit, so they use the front-end allocator.
    // `AIL_SYSTEM_ALLOC=1` keeps the system allocator, to compare the two.
    if matches!(rest.first(), Some(&"check" | &"api" | &"effects")) && std::env::var_os("AIL_SYSTEM_ALLOC").is_none() {
        ALLOC.enable();
    }
    match rest.as_slice() {
        ["run", file] => {
            interp::set_program(file, program_args);
            run_file(file, json, true, &deny)
        }
        ["check", "--errors", file] | ["check", file, "--errors"] => api::errors::command(file, json),
        ["check", file] => run_file(file, json, false, &deny),
        ["test", file] => match take_test_options(&mut flags) {
            Ok((cases, seed)) => run_tests(file, json, cases, seed, coverage),
            Err(msg) => {
                eprintln!("error: {msg}");
                ExitCode::from(2)
            }
        },
        ["effects", file] if deny.is_empty() => effects_file(file, json),
        ["effects", "--parallel", file] | ["effects", file, "--parallel"] if deny.is_empty() => parallel_file(file, json),
        ["pkg", "update", names @ ..] => packages::update_command(names, json),
        ["fix", rest @ ..] => fix::command(rest, json),
        ["explain"] => explain::list(json),
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

/// Remove `--timeout MS`, `--max-memory MB`, `--max-steps N` and `--coverage[=lines]`,
/// and set them. Returns whether coverage was asked for, and whether with `=lines`.
fn take_limit_options(rest: &mut Vec<&str>) -> Result<Option<bool>, String> {
    let cmd = rest.first().copied().unwrap_or("");
    let mut config = interp::limits::Config::default();
    let mut coverage = None;
    let mut i = 0;
    while i < rest.len() {
        let name = rest[i];
        let number = |rest: &Vec<&str>, unit: &str| -> Result<u64, String> {
            rest.get(i + 1)
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| format!("`{name}` needs a positive number of {unit}, e.g. `{name} 100` (run.limit.4)"))
        };
        match name {
            "--timeout" | "--max-memory" | "--max-steps" => {
                if cmd != "run" && cmd != "test" {
                    return Err(format!("`{name}` works with `ail run` and `ail test` (run.limit.4)"));
                }
                match name {
                    "--timeout" => config.timeout_ms = Some(number(rest, "milliseconds")?),
                    "--max-memory" => {
                        if !interp::limits::memory_supported() {
                            return Err("`--max-memory` is not available on Windows yet (run.limit.2)".into());
                        }
                        config.memory_mb = Some(number(rest, "megabytes")?)
                    }
                    _ => config.steps = Some(number(rest, "steps")?),
                }
                rest.drain(i..i + 2);
            }
            "--coverage" | "--coverage=lines" => {
                if cmd != "test" {
                    return Err("`--coverage` works with `ail test` (test.cover.1)".into());
                }
                coverage = Some(name.ends_with("lines"));
                rest.remove(i);
            }
            a if a.starts_with("--coverage=") => {
                return Err(format!("`{a}` isn't a coverage mode; use `--coverage` or `--coverage=lines` (test.cover.2)"));
            }
            _ => i += 1,
        }
    }
    config.coverage = coverage.is_some();
    interp::limits::configure(config);
    Ok(coverage)
}

/// Remove `--schedule=random[:SEED]` and set the random schedule
/// (conc.sched.1). Without a seed, one is made from the clock, and a
/// failing test prints it (conc.sched.2). Pooled loops and tasks run on
/// the scheduler instead, so the seed decides every interleaving.
fn take_schedule(args: &mut Vec<&str>) -> Result<(), String> {
    let Some(at) = args.iter().position(|a| a.starts_with("--schedule")) else { return Ok(()) };
    let word = args.remove(at);
    if !matches!(args.first(), Some(&"run" | &"test")) {
        return Err("`--schedule` works with `ail run` and `ail test` (conc.sched.1)".into());
    }
    let seed = match word {
        "--schedule=random" => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64 % 1_000_000_000),
        "--schedule=fixed" => return Ok(()),
        w => match w.strip_prefix("--schedule=random:").map(str::parse::<u64>) {
            Some(Ok(n)) => n,
            _ => {
                return Err(format!(
                    "`{w}` isn't a schedule; use `--schedule=random` or `--schedule=random:SEED` with a non-negative SEED (conc.sched.1)"
                ));
            }
        },
    };
    interp::set_schedule(Some(seed));
    interp::parallel::set_mode(interp::parallel::Mode::Off);
    Ok(())
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
    // conc.par.8: `run --parallel` puts every proven loop on the pool.
    if args.first() == Some(&"run")
        && let Some(at) = args.iter().position(|a| *a == "--parallel")
    {
        args.remove(at);
        interp::parallel::set_mode(interp::parallel::Mode::Proven);
    }
    if let Some(at) = args.iter().position(|a| *a == "--deterministic") {
        args.remove(at);
        // conc.par.9: deterministic runs keep every loop on one thread.
        interp::parallel::set_mode(interp::parallel::Mode::Off);
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

/// `ail effects --parallel FILE [--json]`: each loop's verdict (conc.par.6).
fn parallel_file(path: &str, json: bool) -> ExitCode {
    let Some((program, resolved)) = front_end(path, json) else { return ExitCode::from(2) };
    let at = |pos| {
        let (file, p) = modules::locate(&program.files, pos);
        let file = if file.is_empty() { path } else { file };
        format!("{file}:{}:{}", p.line, p.col)
    };
    print!("{}", check::parallel::render(&check::parallel::analyze(&program, &resolved), json, &at));
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
        diags.extend(packages::check_grants(&program, &resolved));
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

    // std.ui: this thread hosts any window the program opens (ADR 0178).
    let result = ui::run_hosted(STACK_BYTES, move || {
        let at = |pos| {
            let (file, p) = modules::locate(&program.files, pos);
            format!("{file}:{}:{}", p.line, p.col)
        };
        interp::run(&program, &resolved).map_err(|stop| match stop {
            // trace.snap.1-4: the trap with its snapshot.
            interp::Stop::Trap(t) => interp::render_trap(&program, &t, interp::take_snapshot(), json),
            interp::Stop::Failed(report) => report.render_with(at),
        })
    });
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
fn run_tests(path: &str, json: bool, cases: u64, seed: Option<u64>, coverage: Option<bool>) -> ExitCode {
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
        diags.extend(packages::check_grants(&program, &resolved));
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
    let cover_map = line_map.clone();
    let outcomes = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || {
            let outcomes = interp::run_tests(&program, &resolved, cases, seed)
                .into_iter()
                .map(|o| (o.name, o.pos, o.failure, o.output, o.prop, o.snapshot, o.trace))
                .collect::<Vec<_>>();
            // test.cover.1: what the tests ran, read once they are all done.
            let cover = coverage.map(|_| interp::limits::report(&program, &cover_map));
            (outcomes, cover)
        })
        .expect("spawn interpreter thread")
        .join();
    let Ok((mut outcomes, cover)) = outcomes else {
        eprintln!("{path}: internal error in the bootstrap interpreter");
        return ExitCode::from(1);
    };
    for o in &mut outcomes {
        let Some(t) = doc_tests.iter().find(|t| t.fn_name == o.0) else { continue };
        // test.doc.2-3: report under the example's name, at its own line.
        o.0 = t.name.clone();
        o.5 = None; // the example is generated code; its frame would name `doc_example_N`
        o.6.clear();
        if o.2.is_none() {
            o.2 = doctest::check_output(t, &o.3);
            o.1 = diag::Pos { line: t.line(), col: 1 };
        } else if let Some(&line) = line_map.get(o.1.line.wrapping_sub(1)).filter(|l| **l != 0) {
            o.1.line = line;
        }
    }
    let failed = outcomes.iter().filter(|o| o.2.is_some()).count();
    let passed = outcomes.len() - failed;
    for (name, pos, failure, output, prop, snapshot, trace) in &outcomes {
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
            if let Some(snap) = snapshot {
                line.push_str(&format!(",\"snapshot\":{}", diag::json_str(snap)));
            }
            // trace.test.1: a failing test's events, as the objects `ail run --trace --json` writes.
            if !trace.is_empty() {
                line.push_str(&format!(",\"trace\":[{}]", trace.join(",")));
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
                    if let Some(snap) = snapshot {
                        for line in snap.lines() {
                            println!("  {line}");
                        }
                    }
                    for line in trace {
                        println!("  {line}");
                    }
                    for line in output.lines() {
                        println!("  | {line}");
                    }
                }
            }
        }
    }
    // conc.sched.2: a failure under a random schedule names its seed.
    let schedule = interp::schedule_seed().filter(|_| failed > 0);
    if json {
        let seed = schedule.map_or(String::new(), |s| format!(",\"schedule_seed\":{s}"));
        println!("{{\"summary\":true,\"passed\":{passed},\"failed\":{failed}{seed}}}");
    } else {
        println!("{passed} passed, {failed} failed");
        if let Some(s) = schedule {
            println!("schedule seed {s}: replay with `ail test --schedule=random:{s} {path}`");
        }
    }
    // test.cover.2-3: the table, or per-file JSON lines; `=lines` lists the unexecuted ranges.
    for c in cover.iter().flatten() {
        let pct = if c.lines == 0 { 100 } else { c.lines_hit * 100 / c.lines };
        let ranges: Vec<String> =
            c.missed.iter().map(|&(a, b)| if a == b { a.to_string() } else { format!("{a}-{b}") }).collect();
        if json {
            let missed: Vec<String> = c.missed.iter().map(|(a, b)| format!("[{a},{b}]")).collect();
            println!(
                "{{\"coverage\":true,\"file\":{},\"lines\":{},\"lines_hit\":{},\"functions\":{},\"functions_hit\":{},\"missed\":[{}]}}",
                diag::json_str(&c.file),
                c.lines,
                c.lines_hit,
                c.fns,
                c.fns_hit,
                missed.join(",")
            );
        } else {
            println!("coverage {}: lines {}/{} ({pct}%), functions {}/{}", c.file, c.lines_hit, c.lines, c.fns_hit, c.fns);
            if coverage == Some(true) && !ranges.is_empty() {
                println!("  not run: {}", ranges.join(", "));
            }
        }
    }
    if failed == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
