//! Run limits and coverage end to end (run.limit.1-4, test.cover.1-3, test.run.5).

use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn case(name: &str) -> String {
    format!("{}/tests/limits/{name}.ail", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> (Option<i32>, String, String) {
    let out = ail().args(args).output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn timeout_traps_an_endless_loop_with_a_snapshot() {
    let (code, _, err) = run(&["run", "--timeout", "200", &case("limit_endless")]);
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("trap[run.limit.1]"), "{err}");
    assert!(err.contains("in main("), "{err}");
}

#[test]
fn max_steps_traps_the_same_way_every_time() {
    let first = run(&["run", "--max-steps", "1000", &case("limit_endless")]);
    let second = run(&["run", "--max-steps", "1000", &case("limit_endless")]);
    assert_eq!(first.0, Some(1), "{}", first.2);
    assert!(first.2.contains("trap[run.limit.3]"), "{}", first.2);
    assert_eq!(first.2, second.2);
}

#[cfg(not(windows))]
#[test]
fn max_memory_traps_a_growing_list() {
    let (code, _, err) = run(&["run", "--max-memory", "100", &case("limit_memory")]);
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("trap[run.limit.2]"), "{err}");
}

#[test]
fn a_test_that_times_out_fails_and_the_next_still_runs() {
    let (code, out, err) = run(&["test", "--timeout", "200", &case("limit_test_timeout")]);
    assert_eq!(code, Some(1), "{err}");
    assert!(out.contains("FAIL slow: trap[run.limit.1]"), "{out}");
    assert!(out.contains("ok quick"), "{out}");
    assert!(out.contains("1 passed, 1 failed"), "{out}");
}

#[test]
fn limits_need_a_positive_number_and_run_or_test() {
    for args in [
        vec!["run", "--timeout", "0", "x.ail"],
        vec!["run", "--max-steps", "x", "x.ail"],
        vec!["check", "--timeout", "5", "x.ail"],
    ] {
        let (code, _, err) = run(&args);
        assert_eq!(code, Some(2), "{err}");
        assert!(err.contains("run.limit.4"), "{err}");
    }
}

#[test]
fn coverage_counts_lines_and_functions_of_a_two_function_file() {
    let (code, out, err) = run(&["test", "--coverage=lines", &case("coverage_two_fns")]);
    assert_eq!(code, Some(0), "{err}{out}");
    assert!(out.contains("coverage "), "{out}");
    assert!(out.contains(": lines 3/5 (60%), functions 1/2"), "{out}");
    assert!(out.contains("  not run: 11-12"), "{out}");
}

#[test]
fn coverage_json_lists_missed_ranges() {
    let (_, out, _) = run(&["test", "--json", "--coverage", &case("coverage_two_fns")]);
    let line = out.lines().last().unwrap();
    assert!(line.starts_with("{\"coverage\":true,\"file\":"), "{out}");
    assert!(
        line.ends_with("\"lines\":5,\"lines_hit\":3,\"functions\":2,\"functions_hit\":1,\"missed\":[[11,12]]}"),
        "{out}"
    );
}

#[test]
fn a_failing_test_shows_its_contract_and_stack() {
    let (code, out, _) = run(&["test", &case("limit_test_contract")]);
    assert_eq!(code, Some(1));
    assert!(out.contains("trap[contract.req.1]"), "{out}");
    assert!(out.contains("values: x = 0"), "{out}");
    assert!(out.contains("in boom()"), "{out}");
    let (_, json, _) = run(&["test", "--json", &case("limit_test_contract")]);
    assert!(json.contains("\"snapshot\":\""), "{json}");
}
