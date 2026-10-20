//! `requires` and `ensures` end to end: trap messages, `--no-contracts`,
//! `ail api` and `ail explain` (contract.req.1, contract.ens.1, contract.run.3,
//! contract.api.1), and round two: `old`, inheritance and invariants
//! (contract.ens.3, contract.inh.1-3, contract.inv.1-3, contract.api.2).

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
    for rule in [
        "contract.decl.3",
        "contract.req.1",
        "contract.ens.1",
        "contract.run.3",
        "contract.ens.3",
        "contract.inh.1",
        "contract.inh.2",
        "contract.inh.3",
        "contract.inv.1",
        "contract.inv.2",
        "contract.inv.3",
        "contract.api.2",
    ] {
        let out = ail().args(["explain", rule]).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{rule}");
    }
}

fn trap_text(name: &str) -> String {
    let out = ail().args(["run", &case(name)]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn failed_ensures_with_old_shows_the_value_on_entry() {
    let stderr = trap_text("contract_old_trap");
    assert!(
        stderr.contains("`Counter.add` ensures `n == old(n) + k`, but it returned at 4:5 with self = Counter(n: 4), k = 2"),
        "{stderr}"
    );
    assert!(stderr.contains("values: self = Counter(n: 4), old(n) = 1, k = 2"), "{stderr}");
}

#[test]
fn inherited_requires_names_the_trait_and_uses_the_implementations_parameters() {
    let stderr = trap_text("contract_trait_requires_trap");
    assert!(
        stderr.contains(
            "`Plain.scale` requires `k > 0` (inherited from `Shape.scale`), but it was called with self = Plain(side: 4), factor = 0 (contract at 5:9)"
        ),
        "{stderr}"
    );
}

#[test]
fn inherited_ensures_traps_in_the_implementation() {
    let stderr = trap_text("contract_trait_ensures_trap");
    assert!(stderr.contains("`Odd.area` ensures `result >= 0` (inherited from `Shape.area`), but it returned -6"), "{stderr}");
}

#[test]
fn invariants_name_the_struct_the_method_and_the_value() {
    let stderr = trap_text("contract_invariant_method_trap");
    assert!(
        stderr.contains(
            "`Account` invariant `balance >= 0` was broken by `Account.withdraw`, which returned at 7:5 with self = Account(balance: -7) (invariant at 4:5)"
        ),
        "{stderr}"
    );
    let stderr = trap_text("contract_invariant_built_trap");
    assert!(
        stderr.contains("`Range` invariant `lo <= hi` doesn't hold for Range(lo: 3, hi: 2), built at 9:16 (invariant at 5:5)"),
        "{stderr}"
    );
}

#[test]
fn api_shows_inherited_contracts_and_invariants() {
    let out = ail().args(["api", &case("contract_trait_inherit")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(r#""inherits":"Shape.scale","inherited_requires":["k > 0"],"inherited_ensures":["result >= 0"]"#),
        "{stdout}"
    );
    let out = ail().args(["api", &case("contract_invariant")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(r#""invariants":["balance >= 0","balance <= limit"]"#), "{stdout}");
    let out = ail().args(["api", "--text", &case("contract_trait_inherit")]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("requires k > 0 (inherited)"), "{stdout}");
}
