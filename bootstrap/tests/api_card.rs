//! `ail api --card` (api.card.1-5, ADRs 0310-0311): the content of a card,
//! the trimming under a budget, and the committed example cards.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn card_of(dir: &Path, extra: &[&str]) -> String {
    let out = ail().args(["api", "--card"]).args(extra).arg(dir).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

fn package(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("api_card").join(name).join("notes");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("geo")).unwrap();
    fs::create_dir_all(dir.join("examples")).unwrap();
    fs::create_dir_all(dir.join("../textkit")).unwrap();
    fs::write(dir.join("ail.pkg"), "# manifest\npackage notes\ndep textkit path ../textkit caps fs\n").unwrap();
    fs::write(
        dir.join("geo/shapes.ail"),
        "## A point on the plane.\npub struct P\n    pub x int\n    pub y int\n\n## The origin.\npub fn origin() P\n    P(x: 0, y: 0)\n\nfn hidden() int\n    1\n\nfn hidden_two(a int, b int) int\n    a + b\n\nfn hidden_three(a int, b int) int\n    a * b\n\nfn hidden_four(a int, b int) int\n    a - b\n\nfn hidden_five(a int, b int) int\n    a / b\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.ail"),
        "use geo\n\n## Shows the origin.\npub fn show(p geo.P, n int) str\n    requires n > 0\n    \"{p.x}\"\n\n@test fn shows_origin()\n    assert(show(geo.origin(), 1) == \"0\")\n\nfn main()\n    print(show(geo.origin(), 1))\n",
    )
    .unwrap();
    fs::write(dir.join("examples/hello.ail"), "## Prints a greeting.\nfn main()\n    print(\"hi\")\n").unwrap();
    dir
}

/// api.card.1: modules, signatures, contracts, docs, entry, tests, manifest, examples.
#[test]
fn card_lists_the_program() {
    let card = card_of(&package("card_lists_the_program"), &[]);
    for want in [
        "# notes\n",
        "entry: main in main.ail\n",
        "tests: shows_origin\n",
        "    package notes\n",
        "    dep textkit path ../textkit caps fs\n",
        "    hello.ail  ## Prints a greeting.\n",
        "## (root)\n",
        "pub fn show(p geo.P, n int) str  ## Shows the origin.\n",
        "    requires n > 0\n",
        "## geo\n",
        "pub fn geo.origin() geo.P  ## The origin.\n",
        "pub struct geo.P  ## A point on the plane.\n",
        "fn geo.hidden() int\n",
    ] {
        assert!(card.contains(want), "missing {want:?} in\n{card}");
    }
    assert!(!card.contains("trimmed"), "{card}");
    // The root module comes first, then modules by name.
    assert!(card.find("## (root)").unwrap() < card.find("## geo").unwrap(), "{card}");
}

/// api.card.3: over budget, private items go first, and the card says so.
#[test]
fn private_items_are_dropped_first() {
    let dir = package("private_items_are_dropped_first");
    let full = card_of(&dir, &[]);
    // The first budget, going down, that the full card no longer fits.
    let trimmed = (1..full.len() / 3)
        .rev()
        .map(|b| card_of(&dir, &["--budget", &b.to_string()]))
        .find(|c| c.lines().last().is_some_and(|l| l.ends_with("dropped: private items")))
        .expect("some budget drops only private items");
    assert!(!trimmed.contains("hidden()"), "{trimmed}");
    assert!(trimmed.contains("pub fn geo.origin() geo.P  ## The origin."), "{trimmed}");
    let last = trimmed.lines().last().unwrap();
    assert!(last.starts_with("trimmed to fit ") && last.ends_with("dropped: private items"), "{last}");
}

/// api.card.3: a tiny budget drops docs, parameter names, then signatures.
#[test]
fn a_small_budget_drops_in_order() {
    let dir = package("a_small_budget_drops_in_order");
    let docs_gone = card_of(&dir, &["--budget", "120"]);
    assert!(!docs_gone.contains("## The origin."), "{docs_gone}");
    let last = docs_gone.lines().last().unwrap();
    assert!(last.contains("private items, doc comments"), "{last}");
    let names = card_of(&dir, &["--budget", "1"]);
    assert!(names.contains("(2 items)") || names.contains("items)"), "{names}");
    assert!(names.lines().last().unwrap().ends_with("signatures (names only)"), "{names}");
}

/// api.card.2: the JSON form has the same content.
#[test]
fn json_form() {
    let out = ail().args(["api", "--card", "--json"]).arg(package("json_form")).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    for want in [
        "\"card\":\"notes\"",
        "\"entry\":\"main in main.ail\"",
        "\"tests\":[\"shows_origin\"]",
        "\"module\":\"geo\"",
        "\"signature\":\"fn geo.origin() geo.P\"",
        "\"requires\":[\"n > 0\"]",
        "\"dropped\":[]",
    ] {
        assert!(text.contains(want), "missing {want} in\n{text}");
    }
}

/// api.card.4: a program the checker rejects has no card; a bad option is a usage error.
#[test]
fn rejected_programs_and_bad_options() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("api_card").join("bad");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("main.ail"), "fn main()\n    x := nope()\n").unwrap();
    let out = ail().args(["api", "--card"]).arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let out = ail().args(["api", "--card", "--budget", "many"]).arg(&dir).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

/// api.card.5: the cards under docs/design/cards are what the command prints.
/// Regenerate with the command in each file's name below, from the repository root.
#[test]
fn committed_cards_are_fresh() {
    for (target, file) in [
        ("tools/edit", "edit.md"),
        ("bootstrap/src/std", "std.md"),
        ("tools/garlic", "garlic.md"),
        ("tools/garlic/js", "garlic-js.md"),
        ("tools/pdf", "pdf.md"),
        ("compiler", "compiler.md"),
    ] {
        let out = ail().current_dir(repo()).args(["api", "--card", target]).output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
        let want = String::from_utf8(out.stdout).unwrap();
        let path = repo().join("docs/design/cards").join(file);
        let have = fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(have, want, "{file} is stale; run `./ail api --card {target} > docs/design/cards/{file}`");
    }
}
