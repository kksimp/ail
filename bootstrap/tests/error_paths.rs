//! `ail check --errors` and `fails_with` in `ail api` (err.paths.1, err.api.1,
//! ADRs 0286-0287), on `cases/err_paths_ok.ail`.

use std::path::Path;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn case() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/err_paths_ok.ail").display().to_string()
}

#[test]
fn check_errors_lists_each_failure_path() {
    let path = case();
    let out = ail().args(["check", "--errors", &path]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8_lossy(&out.stdout);
    // `both` infers A | B; `?` on `a` carries the context.
    assert!(text.contains("\"id\":\"fn:both\",\"name\":\"both\",\"line\":17,\"fails_with\":[\"A\",\"B\"]"), "{text}");
    assert!(
        text.contains(
            "{\"line\":18,\"col\":14,\"kind\":\"propagate\",\"call\":\"a\",\"errors\":[\"A\"],\"context\":true}"
        ),
        "{text}"
    );
    assert!(
        text.contains("{\"line\":19,\"col\":14,\"kind\":\"propagate\",\"call\":\"b\",\"errors\":[\"B\"]}"),
        "{text}"
    );
    assert!(text.contains("\"id\":\"fn:a\",\"name\":\"a\",\"line\":7,\"fails_with\":[\"A\"]"), "{text}");
    // `main` handles the failure with `else`, so it doesn't fail itself.
    assert!(
        text.contains(
            "\"name\":\"main\",\"line\":22,\"paths\":[{\"line\":23,\"col\":11,\"kind\":\"handle\",\"how\":\"default\""
        ),
        "{text}"
    );
}

#[test]
fn api_gives_the_inferred_error_set() {
    let path = case();
    let json = ail().args(["api", &path]).output().unwrap();
    let json = String::from_utf8_lossy(&json.stdout);
    assert!(json.contains("\"returns\":\"!int\",\"fails_with\":[\"A\",\"B\"]"), "{json}");
    let text = ail().args(["api", "--text", &path]).output().unwrap();
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("fn both(n int) !int\n           fails with A | B"), "{text}");
}

#[test]
fn explain_shows_a_rejected_program_and_its_repair() {
    let out = ail().args(["explain", "err.ret.1"]).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("rejected example:") && text.contains("repaired:"), "{text}");
    assert!(!text.contains("\nexample:"), "{text}");
}
