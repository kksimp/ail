//! `ail api --locals/--type/--refs` (api.loc.1-2, api.type.1-2, api.refs.1-3,
//! ADRs 0258-0261): exit codes, and a reference across user modules.

use std::fs;
use std::path::Path;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn package(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("api_queries").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("geo")).unwrap();
    fs::write(dir.join("geo/shapes.ail"), "pub struct P\n    pub x int\n\npub fn origin() P\n    P(x: 0)\n").unwrap();
    fs::write(dir.join("main.ail"), "use geo\n\nfn main()\n    p := geo.origin()\n    print(\"{p.x}\")\n").unwrap();
    dir
}

#[test]
fn refs_cross_user_modules() {
    let dir = package("refs");
    let main = dir.join("main.ail");
    let out = ail().args(["api", "--refs"]).arg(&main).args(["4", "12"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("\"symbol\":\"fn:geo.origin\""), "{text}");
    let shapes = dir.join("geo/shapes.ail");
    assert!(
        text.contains(&format!("{{\"file\":{:?},\"line\":4,\"col\":8,\"def\":true}}", shapes.to_str().unwrap())),
        "{text}"
    );
    assert!(text.contains("\"line\":4,\"col\":10,\"def\":false"), "{text}");
}

#[test]
fn type_of_a_call_into_a_module_names_the_callee() {
    let dir = package("type");
    let out = ail().args(["api", "--type"]).arg(dir.join("main.ail")).args(["4", "12"]).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("\"type\":\"geo.P\""), "{text}");
    assert!(
        text.contains("\"signature\":\"fn geo.origin() P\"") || text.contains("\"id\":\"fn:geo.origin\""),
        "{text}"
    );
}

#[test]
fn exit_codes() {
    let dir = package("codes");
    let main = dir.join("main.ail");
    // api.query.1: nothing at the position is exit 1 with a JSON error on stderr.
    let out = ail().args(["api", "--type"]).arg(&main).args(["3", "1"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("{\"api\":1,\"severity\":\"error\""));
    // A missing position is a usage error.
    let out = ail().args(["api", "--locals"]).arg(&main).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    // A rejected program has no answer (api.out.5).
    let bad = dir.join("bad.ail");
    fs::write(&bad, "fn main()\n    x\n").unwrap();
    let out = ail().args(["api", "--locals", "--json"]).arg(&bad).args(["2", "5"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\"rule\":"));
}

#[test]
fn every_example_answers_locals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    for e in fs::read_dir(root).unwrap() {
        let path = e.unwrap().path();
        if path.extension().is_some_and(|x| x == "ail") && path.with_extension("out").exists() {
            let n = fs::read_to_string(&path).unwrap().lines().count();
            let out = ail().args(["api", "--locals"]).arg(&path).args([&n.to_string(), "5"]).output().unwrap();
            assert_eq!(out.status.code(), Some(0), "{}: {}", path.display(), String::from_utf8_lossy(&out.stderr));
        }
    }
}

fn big_package(name: &str) -> std::path::PathBuf {
    let dir = package(name);
    fs::write(dir.join("main.ail"), "use geo\n\nfn main()\n    p := geo.origin()\n    print(\"{p.x} {far()}\")\n")
        .unwrap();
    fs::write(dir.join("util.ail"), "use geo\n\npub fn far() int\n    geo.origin().x\n").unwrap();
    dir
}

/// api.pkg.1: a package directory is outlined whole, with module-prefixed ids,
/// file-local lines and a `module` and `file` per item.
#[test]
fn outline_of_a_package_directory() {
    let dir = big_package("outline");
    let out = ail().args(["api", "--outline"]).arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("{\"api\":1,\"package\":"), "{text}");
    for want in
        ["\"id\":\"fn:geo.origin\"", "\"id\":\"type:geo.P\"", "\"id\":\"fn:far\"", "\"module\":\"geo\"", "util.ail"]
    {
        assert!(text.contains(want), "{want} in {text}");
    }
    let symbols = ail().args(["api", "--symbols"]).arg(&dir).output().unwrap();
    let symbols = String::from_utf8(symbols.stdout).unwrap();
    assert!(symbols.contains("\"references\":[") && symbols.contains("\"id\":\"fn:geo.origin\""), "{symbols}");
    let text_form = ail().args(["api", "--text"]).arg(&dir).output().unwrap();
    assert!(String::from_utf8_lossy(&text_form.stdout).contains("fn geo.origin() geo.P"));
}

/// api.pkg.2-3: `--refs DIR FILE LINE COL` finds uses in every file of the
/// package, from a use, from the name after `geo.`, or from the declaration.
#[test]
fn refs_across_a_package() {
    let dir = big_package("refs_pkg");
    let util = dir.join("util.ail");
    let shapes = dir.join("geo/shapes.ail");
    let ask = |file: &Path, line: &str, col: &str| {
        let out = ail().args(["api", "--refs"]).arg(&dir).arg(file).args([line, col]).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8(out.stdout).unwrap()
    };
    for text in [ask(&util, "4", "9"), ask(&util, "4", "5"), ask(&shapes, "4", "8")] {
        assert!(text.contains("\"symbol\":\"fn:geo.origin\""), "{text}");
        for file in ["main.ail", "util.ail", "geo/shapes.ail"] {
            assert!(text.contains(&format!("{}\"", dir.join(file).display())), "{file} in {text}");
        }
    }
    // The single-file form keeps working (api.pkg.4).
    let single = package("refs_single");
    let out = ail().args(["api", "--refs"]).arg(single.join("main.ail")).args(["4", "12"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    // A file outside the package is an error.
    let stray = dir.join("../stray.ail");
    fs::write(&stray, "fn main()\n    print(\"x\")\n").unwrap();
    let out = ail().args(["api", "--refs"]).arg(&dir).arg(&stray).args(["1", "4"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("api.pkg.2"));
}
