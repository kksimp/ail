//! `ail test --native [PATH...]`: a differential runner (ADR 0679). Each
//! `@test` function is built with `ail build` and run as its own process,
//! then compared with what `ail test` says about it in the interpreter
//! (native.test.1-4).
//!
//! The backend has no `assert` and refuses to call a `@test` function, so
//! the runner builds a rewritten copy of the file: `@test` becomes an
//! ordinary `fn NAME() int?`, `assert(c)` and `assert_eq(a, b)` become calls
//! of a checking function, `main` is replaced by a dispatcher that picks a
//! test by the length of its first argument, and the copy sits beside the
//! original so its `use` lines resolve. A test the backend can't build yet
//! is left out and reported with the first construct that blocks it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const RUN_LIMIT: Duration = Duration::from_secs(30);
const MAX_ROUNDS: usize = 12;

/// A `@test` function found in the source: its name, and its first and last
/// lines (0-based, inclusive).
struct TestFn {
    name: String,
    first: usize,
    last: usize,
    header: usize,
}

/// What the runner concluded about one test.
struct Outcome {
    file: String,
    test: String,
    interp: &'static str,
    native: String,
    status: &'static str,
    reason: String,
}

/// native.test.1: run `ail test --native` over files and directories.
pub fn command(paths: &[&str], json: bool) -> ExitCode {
    let root = crate::build::repo_root();
    let mut files = Vec::new();
    if paths.is_empty() {
        for d in ["examples", "bootstrap/tests/cases", "tools"] {
            collect(&root.join(d), &mut files);
        }
    } else {
        for p in paths {
            let p = PathBuf::from(p);
            if p.is_dir() {
                collect(&p, &mut files);
            } else {
                files.push(p);
            }
        }
    }
    files.sort();
    let scratch = std::env::temp_dir().join(format!("ail-native-test-{}", std::process::id()));
    if std::fs::create_dir_all(&scratch).is_err() {
        eprintln!("error[native.test.1]: can't create {}", scratch.display());
        return ExitCode::from(2);
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Vec<Outcome>)>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 4);
    std::thread::scope(|s| {
        for w in 0..workers {
            let (files, next, results, scratch) = (&files, &next, &results, &scratch);
            s.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(file) = files.get(i) else { break };
                let out = run_file(file, &scratch.join(format!("w{w}")));
                results.lock().unwrap().push((i, out));
            });
        }
    });
    let _ = std::fs::remove_dir_all(&scratch);
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|r| r.0);
    let all: Vec<Outcome> = results.into_iter().flat_map(|r| r.1).collect();
    report(&all, json)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        if p.is_dir() {
            collect(&p, out);
        } else if name.ends_with(".ail") && std::fs::read_to_string(&p).is_ok_and(|s| s.contains("@test")) {
            out.push(p);
        }
    }
}

/// native.test.4: counts, then every test that differs.
fn report(all: &[Outcome], json: bool) -> ExitCode {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for o in all {
        *counts.entry(o.status).or_default() += 1;
    }
    let n = |k: &str| counts.get(k).copied().unwrap_or(0);
    if json {
        for o in all {
            println!(
                "{{\"file\":{},\"test\":{},\"interp\":\"{}\",\"native\":{},\"status\":\"{}\",\"reason\":{}}}",
                crate::diag::json_str(&o.file),
                crate::diag::json_str(&o.test),
                o.interp,
                crate::diag::json_str(&o.native),
                o.status,
                crate::diag::json_str(&o.reason)
            );
        }
        println!(
            "{{\"summary\":true,\"match\":{},\"differ\":{},\"not-native\":{},\"skipped\":{}}}",
            n("match"),
            n("DIFF"),
            n("not-native"),
            n("skipped")
        );
    } else {
        for o in all {
            let tail = if o.reason.is_empty() { String::new() } else { format!(": {}", o.reason) };
            println!("{} {}::{} (native {}, interpreter {}){tail}", o.status, o.file, o.test, o.native, o.interp);
        }
        println!(
            "{} tests: {} match, {} DIFFER, {} not in the native subset yet, {} skipped",
            all.len(),
            n("match"),
            n("DIFF"),
            n("not-native"),
            n("skipped")
        );
    }
    if n("DIFF") > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn outcome(file: &str, test: &str, interp: &'static str, native: &str, status: &'static str, reason: &str) -> Outcome {
    Outcome {
        file: file.into(),
        test: test.into(),
        interp,
        native: native.into(),
        status,
        reason: reason.into(),
    }
}

/// The interpreter's verdict on each test of `file`: `ail test --json`.
fn interpreter_results(file: &Path) -> Result<HashMap<String, &'static str>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let out = Command::new(exe)
        .args(["test", "--json"])
        .arg(file)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut map = HashMap::new();
    for line in text.lines() {
        let Some(at) = line.find("\"test\":\"") else { continue };
        let rest = &line[at + 8..];
        let Some(end) = rest.find('"') else { continue };
        let status = if line.contains("\"status\":\"pass\"") { "pass" } else { "fail" };
        map.insert(rest[..end].to_string(), status);
    }
    if map.is_empty() && !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().next().unwrap_or("`ail test` failed").to_string());
    }
    Ok(map)
}

fn run_file(file: &Path, work: &Path) -> Vec<Outcome> {
    let name = file.to_string_lossy().into_owned();
    let Ok(src) = std::fs::read_to_string(file) else { return Vec::new() };
    let lines: Vec<&str> = src.lines().collect();
    let (tests, skipped, left_out) = find_tests(&lines);
    let mut out: Vec<Outcome> = skipped
        .iter()
        .map(|(n, why)| outcome(&name, n, "-", "-", "skipped", why))
        .collect();
    if tests.is_empty() {
        return out;
    }
    let interp = match interpreter_results(file) {
        Ok(m) => m,
        Err(why) => {
            out.extend(tests.iter().map(|t| outcome(&name, &t.name, "-", "-", "skipped", &format!("the interpreter rejects the file: {why}"))));
            return out;
        }
    };
    let istatus = |t: &str| interp.get(t).copied().unwrap_or("fail");
    let _ = std::fs::create_dir_all(work);
    let dir = file.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("t");
    let copy = dir.join(format!(".ail-native-test-{}-{stem}.ail", std::process::id()));
    let exe_path = work.join(stem);
    let mut dropped: HashMap<usize, String> = HashMap::new();
    let mut blocked_all: Option<String> = None;
    let mut built = false;
    for _ in 0..MAX_ROUNDS {
        let live: Vec<usize> = (0..tests.len()).filter(|i| !dropped.contains_key(i)).collect();
        if live.is_empty() {
            break;
        }
        let (text, origin) = rewrite(&lines, &tests, &left_out, &dropped);
        if std::fs::write(&copy, text).is_err() {
            blocked_all = Some("can't write the rewritten copy".into());
            break;
        }
        let ail = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ail"));
        let res = Command::new(ail)
            .arg("build")
            .arg(&copy)
            .arg("-o")
            .arg(&exe_path)
            .stdin(Stdio::null())
            .output();
        let Ok(res) = res else {
            blocked_all = Some("can't run `ail build`".into());
            break;
        };
        if res.status.success() {
            built = true;
            break;
        }
        let err = String::from_utf8_lossy(&res.stderr).into_owned();
        let mut progressed = false;
        let mut first_outside: Option<String> = None;
        if err.contains("fs.IoError") {
            blocked_all = Some("a module file of a package can't be built on its own yet".into());
            break;
        }
        for l in err.lines() {
            let Some(pos) = l.find(": error[") else { continue };
            let msg = l[pos + 2..].to_string();
            let head = &l[..pos];
            let mut parts = head.rsplitn(3, ':');
            let (_col, line, _path) = (parts.next(), parts.next(), parts.next());
            let Some(line) = line.and_then(|n| n.parse::<usize>().ok()) else {
                first_outside.get_or_insert(msg);
                continue;
            };
            let orig = origin.get(line.wrapping_sub(1)).copied().flatten();
            let owner = orig.and_then(|o| live.iter().copied().find(|&i| tests[i].first <= o && o <= tests[i].last));
            let why = format!("{} (line {})", short(&msg), orig.map_or(line, |o| o + 1));
            match owner {
                Some(i) => {
                    dropped.entry(i).or_insert(why);
                    progressed = true;
                }
                None => {
                    first_outside.get_or_insert(why);
                }
            }
        }
        if !progressed {
            let why = first_outside
                .or_else(|| err.lines().find(|l| !l.trim().is_empty()).map(|l| l.to_string()))
                .unwrap_or_else(|| "the build failed".into());
            blocked_all = Some(why);
            break;
        }
    }
    if std::env::var_os("AIL_NATIVE_TEST_KEEP").is_none() {
        let _ = std::fs::remove_file(&copy);
    }
    for (i, t) in tests.iter().enumerate() {
        let is = istatus(&t.name);
        if let Some(why) = dropped.get(&i) {
            out.push(outcome(&name, &t.name, is, "-", "not-native", why));
        } else if let Some(why) = &blocked_all {
            out.push(outcome(&name, &t.name, is, "-", "not-native", &format!("outside the tests: {why}")));
        } else if !built {
            out.push(outcome(&name, &t.name, is, "-", "not-native", "no build within the round limit"));
        } else {
            let slot = (0..i).filter(|j| !dropped.contains_key(j)).count();
            let (native, why) = run_one(&exe_path, slot + 1);
            let status = if native == is { "match" } else { "DIFF" };
            let reason = if status == "DIFF" || native == "fail" { why } else { String::new() };
            out.push(outcome(&name, &t.name, is, native, status, &reason));
        }
    }
    let _ = std::fs::remove_file(&exe_path);
    out
}

/// native.test.3: a diagnostic without its `error[RULE]: ` prefix and any backticked detail noise.
fn short(msg: &str) -> String {
    match msg.find("]: ") {
        Some(p) => msg[p + 3..].to_string(),
        None => msg.to_string(),
    }
}

/// Run the built tests binary on test number `k` (an argument of `k` letters).
fn run_one(exe: &Path, k: usize) -> (&'static str, String) {
    let child = Command::new(exe)
        .arg("x".repeat(k))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let Ok(mut child) = child else { return ("fail", "can't start the native binary".into()) };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > RUN_LIMIT => {
                let _ = child.kill();
                let _ = child.wait();
                return ("fail", "timed out".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => break,
        }
    }
    let Ok(out) = child.wait_with_output() else { return ("fail", "no output".into()) };
    if out.status.success() {
        ("pass", String::new())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().next().unwrap_or("").to_string();
        let code = out.status.code().map_or("a signal".to_string(), |c| format!("exit {c}"));
        ("fail", if first.is_empty() { code } else { format!("{code}: {first}") })
    }
}

fn indent(l: &str) -> usize {
    l.len() - l.trim_start().len()
}

/// The last line of the block starting at `first` (a column-0 header).
fn block_end(lines: &[&str], first: usize) -> usize {
    let mut last = first;
    for (j, l) in lines.iter().enumerate().skip(first + 1) {
        if l.trim().is_empty() || l.trim_start().starts_with('#') && indent(l) == 0 {
            continue;
        }
        if indent(l) == 0 {
            break;
        }
        last = j;
    }
    last
}

/// Every plain `@test fn NAME()` in the file, and the names of those left out
/// (a signature, or a property test) with why.
fn find_tests(lines: &[&str]) -> (Vec<TestFn>, Vec<(String, String)>, Vec<(usize, usize)>) {
    let mut tests = Vec::new();
    let mut skipped = Vec::new();
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if let Some(rest) = l.strip_prefix("@test") {
            if rest.is_empty() || rest.starts_with(' ') {
                let (header, text) = if rest.trim().is_empty() {
                    match (i + 1..lines.len()).find(|&j| !lines[j].trim().is_empty()) {
                        Some(j) => (j, lines[j].to_string()),
                        None => (i, String::new()),
                    }
                } else {
                    (i, rest.trim_start().to_string())
                };
                if let Some(sig) = text.strip_prefix("fn ") {
                    if let Some(open) = sig.find('(') {
                        let name = sig[..open].trim().to_string();
                        let after = sig[open..].trim_end();
                        if after == "()" {
                            let last = block_end(lines, header);
                            tests.push(TestFn { name, first: i, last, header });
                            i = last;
                        } else {
                            skipped.push((name, "a property test (it takes arguments)".to_string()));
                            ranges.push((i, block_end(lines, header)));
                        }
                    }
                }
            }
        }
        i += 1;
    }
    (tests, skipped, ranges)
}

/// Is `b` a byte that can continue an identifier?
fn ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The index just past a string literal that starts at `i` (a `"`), skipping
/// escapes and `{...}` interpolations.
fn skip_string(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            b'{' => i = skip_braces(b, i),
            _ => i += 1,
        }
    }
    b.len()
}

/// The index just past a `{...}` that starts at `i`.
fn skip_braces(b: &[u8], mut i: usize) -> usize {
    let mut depth = 0;
    while i < b.len() {
        match b[i] {
            b'"' => {
                i = skip_string(b, i);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    b.len()
}

/// Rewrite `assert(c)` and `assert_eq(a, b)` in one line-joined text into
/// calls of `ail_native_check`, which takes the original line number.
fn rewrite_asserts(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 64);
    let mut i = 0;
    let mut line = 1;
    let mut copied = 0;
    while i < b.len() {
        match b[i] {
            b'\n' => {
                line += 1;
                i += 1;
            }
            b'#' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                let end = skip_string(b, i);
                line += b[i..end].iter().filter(|&&c| c == b'\n').count();
                i = end;
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < b.len() && ident_byte(b[i]) {
                    i += 1;
                }
                let word = &text[start..i];
                let prev_dot = start > 0 && b[start - 1] == b'.';
                if (word == "assert" || word == "assert_eq") && b.get(i) == Some(&b'(') && !prev_dot {
                    if let Some((args, end)) = call_args(text, i) {
                        let call = match (word, args.len()) {
                            ("assert", 1) => Some(format!("ail_native_check({}, {line})", args[0])),
                            ("assert_eq", 2) => {
                                Some(format!("ail_native_check(({}) == ({}), {line})", args[0], args[1]))
                            }
                            _ => None,
                        };
                        if let Some(call) = call {
                            out.push_str(&text[copied..start]);
                            out.push_str(&call);
                            // Trimmed arguments lose their newlines; give them back so lines stay put.
                            let lost = text[start..end].matches('\n').count() - call.matches('\n').count();
                            out.push_str(&"\n".repeat(lost));
                            line += text[start..end].matches('\n').count();
                            copied = end;
                            i = end;
                        }
                    }
                }
            }
            _ => i += 1,
        }
    }
    out.push_str(&text[copied..]);
    out
}

/// The top-level arguments of the call whose `(` is at `open`, and the index
/// after its `)`.
fn call_args(text: &str, open: usize) -> Option<(Vec<String>, usize)> {
    let b = text.as_bytes();
    let (mut depth, mut i, mut from) = (0i32, open, open + 1);
    let mut args = Vec::new();
    while i < b.len() {
        match b[i] {
            b'"' => {
                i = skip_string(b, i);
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    let last = text[from..i].trim();
                    if !(last.is_empty() && !args.is_empty()) {
                        args.push(last.to_string());
                    }
                    return Some((args, i + 1));
                }
            }
            b',' if depth == 1 => {
                args.push(text[from..i].trim().to_string());
                from = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// native.test.2: the rewritten copy of the file, and for each of its lines the original
/// line it came from (`None` for lines the runner added). Left-out tests and
/// `main` become blank lines, so positions stay put.
fn rewrite(lines: &[&str], tests: &[TestFn], left_out: &[(usize, usize)], dropped: &HashMap<usize, String>) -> (String, Vec<Option<usize>>) {
    let mut keep: Vec<Option<String>> = lines.iter().map(|l| Some((*l).to_string())).collect();
    for (i, t) in tests.iter().enumerate() {
        if dropped.contains_key(&i) {
            for j in t.first..=t.last {
                keep[j] = Some(String::new());
            }
        } else {
            let h = lines[t.header];
            let sig = h.trim_start_matches("@test").trim_start();
            keep[t.first] = Some(String::new());
            keep[t.header] = Some(format!("{} int?", sig.trim_end()));
        }
    }
    for &(a, z) in left_out {
        for j in a..=z {
            keep[j] = Some(String::new());
        }
    }
    // `main` is the dispatcher's.
    if let Some(m) = lines.iter().position(|l| l.starts_with("fn main(")) {
        for j in m..=block_end(lines, m) {
            keep[j] = Some(String::new());
        }
    }
    let joined: Vec<String> = keep.into_iter().map(|l| l.unwrap_or_default()).collect();
    let body = rewrite_asserts(&joined.join("\n"));
    let rewritten: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut origin: Vec<Option<usize>> = Vec::new();
    for l in ["use std.env", "use std.io"] {
        if !lines.iter().any(|x| x.trim_end() == l) {
            out.push(l.to_string());
            origin.push(None);
        }
    }
    let ends: HashMap<usize, ()> = tests
        .iter()
        .enumerate()
        .filter(|(i, _)| !dropped.contains_key(i))
        .map(|(_, t)| (t.last, ()))
        .collect();
    for (j, l) in rewritten.iter().enumerate() {
        out.push((*l).to_string());
        origin.push(Some(j));
        if ends.contains_key(&j) {
            out.push("    return 0".to_string());
            origin.push(None);
        }
    }
    let mut tail = String::from(
        "\nfn ail_native_check(ok bool, line int)\n    if not ok\n        io.eprint(\"assertion failed at line {line}\")\n        io.exit(1)\n\nfn main()\n    args := env.args()\n    k := args[0].len()\n",
    );
    let mut slot = 1;
    for (i, t) in tests.iter().enumerate() {
        if dropped.contains_key(&i) {
            continue;
        }
        tail.push_str(&format!(
            "    if k == {slot}\n        if r := {}()\n            io.exit(r)\n        io.exit(3)\n",
            t.name
        ));
        slot += 1;
    }
    tail.push_str("    io.exit(4)\n");
    for l in tail.split('\n') {
        out.push(l.to_string());
        origin.push(None);
    }
    (out.join("\n"), origin)
}
