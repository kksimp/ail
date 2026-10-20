//! End-to-end tests of the interpreter's thread pool (conc.par.8-11).

use std::process::Command;

fn run(threads: &str, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(args)
        .env("AIL_THREADS", threads)
        .env("AIL_PAR_LOG", "1")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap()
}

#[test]
fn par_for_runs_on_the_pool_with_sequential_output() {
    let file = "bootstrap/tests/cases/par_pool.ail";
    let one = run("1", &["run", file]);
    let four = run("4", &["run", file]);
    assert!(four.status.success(), "{}", String::from_utf8_lossy(&four.stderr));
    assert_eq!(one.stdout, four.stdout);
    let log = String::from_utf8_lossy(&four.stderr);
    assert!(log.contains("par: loop at line 23: 200 iterations on 4 threads"), "{log}");
    assert!(String::from_utf8_lossy(&one.stderr).is_empty(), "one thread uses no pool");
}

#[test]
fn parallel_flag_pools_proven_loops() {
    let one = run("1", &["run", "examples/parallel.ail"]);
    let four = run("4", &["run", "--parallel", "examples/parallel.ail"]);
    assert_eq!(one.stdout, four.stdout);
    let log = String::from_utf8_lossy(&four.stderr);
    assert!(log.contains("par: loop at line 34: 16 iterations on 4 threads"), "{log}");
    // Without the flag only `par for` loops use the pool.
    let plain = run("4", &["run", "examples/parallel.ail"]);
    assert!(String::from_utf8_lossy(&plain.stderr).is_empty());
}

#[test]
fn deterministic_runs_on_one_thread() {
    let out = run("4", &["run", "--deterministic", "bootstrap/tests/cases/par_pool.ail"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).is_empty(), "{}", String::from_utf8_lossy(&out.stderr));
}
