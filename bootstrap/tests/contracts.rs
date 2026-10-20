//! `requires` and `ensures` end to end: trap messages, `--no-contracts`,
//! `ail api` and `ail explain` (contract.req.1, contract.ens.1, contract.run.3,
//! contract.api.1).

use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn case(name: &str) -> String {
    format!("{}/tests/cases/{name}.ail", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn failed_requires_names_the_function_condition_and_values() {
    let out = ail().args(["run", &case("contract_requires_trap")]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains(":8:11: trap[contract.req.1]: `divide` requires `b != 0`, but it was called with a = 7, b = 0 (contract at 3:5)"),
        "{stderr}"
    );
}

#[test]
fn failed_ensures_names_the_result_and_where_it_returned() {
    let out = ail().args(["run", &case("contract_ensures_trap")]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains(
            ":4:5: trap[contract.ens.1]: `divide` ensures `result * b == a`, but it returned 3 at 5:5 with a = 7, b = 2"
        ),
        "{stderr}"
    );
}

#[test]
fn no_contracts_skips_the_checks() {
    let out = ail().args(["run", "--no-contracts", &case("contract_ensures_trap")]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n3\n");
}

#[test]
fn api_lists_contracts_with_the_signature() {
    let out = ail().args(["api", &case("contract_ensures_trap")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(r#""requires":["b != 0"],"ensures":["result * b == a"]"#), "{stdout}");
    let out = ail().args(["api", "--text", &case("contract_ensures_trap")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fn divide(a int, b int) int\n           requires b != 0\n"), "{stdout}");
}

#[test]
fn explain_knows_the_contract_rules() {
    for rule in ["contract.decl.3", "contract.req.1", "contract.ens.1", "contract.run.3"] {
        let out = ail().args(["explain", rule]).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{rule}");
    }
}
