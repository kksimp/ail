//! The tools written in AIL (`tools/`) are AIL programs too: their checks
//! and `@test` suites run with the bootstrap's own tests.

use std::path::Path;
use std::process::Command;

fn ail() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ail"))
}

fn tool(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel).display().to_string()
}

/// `tools/edit/edit.ail` (ADR 0180) checks without diagnostics and passes
/// its headless `@test` suite.
#[test]
fn editor_checks_clean_and_passes_its_tests() {
    let path = tool("tools/edit/edit.ail");
    let check = ail().args(["check", &path]).output().unwrap();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check.status.success() && stderr.is_empty(), "ail check {path}:\n{stderr}");
    let test = ail().args(["test", &path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test.status.success() && stdout.contains(" 0 failed"), "ail test {path}:\n{stdout}");
}

/// `tools/garlic/garlic.ail`, the text-mode browser
/// (docs/design/garlic-browser.md), checks clean, passes its `@test`s, and
/// prints a page and its page model headless.
#[test]
fn garlic_checks_clean_and_renders_headless() {
    let path = tool("tools/garlic/garlic.ail");
    let check = ail().args(["check", &path]).output().unwrap();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check.status.success() && stderr.is_empty(), "ail check {path}:\n{stderr}");
    let test = ail().args(["test", &path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test.status.success() && stdout.contains(" 0 failed"), "ail test {path}:\n{stdout}");
    let page = tool("tools/garlic/testdata/index.html");
    let run = |flags: &[&str]| {
        let mut args = vec!["run".to_string(), path.clone()];
        args.extend(flags.iter().map(|f| f.to_string()));
        args.push(page.clone());
        let out = ail().args(&args).output().unwrap();
        assert!(out.status.success(), "{flags:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let text = run(&["--width", "60", "--print"]);
    assert!(text.starts_with("Second page[1] | Example[2]\n\n# Garlic\n"), "--print:\n{text}");
    let json = run(&["--width", "60", "--height", "8", "--json"]);
    assert!(json.starts_with("{\"api\":1,") && json.contains("\"role\":\"link\",\"name\":\"Second page\""), "--json:\n{json}");
    let second = run(&["--follow", "1", "--print"]);
    assert!(second.contains("# Second page"), "--follow 1:\n{second}");
    let viewport = run(&["--height", "4", "--viewport", "--do", "find:the end", "--json"]);
    assert!(viewport.contains("\"message\":\"found `the end`") && !viewport.contains("\"Second page\""), "--viewport:\n{viewport}");
}

/// Ctrl-Shift-H previews a replace across the directory's `.ail` files,
/// then Ctrl-Enter applies it to the buffers without saving (ADR 0339).
#[test]
fn editor_replaces_in_files() {
    let dir = std::env::temp_dir().join(format!("ail-edit-replace-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(tool("tools/edit/testdata/agent.ail"), dir.join("a.ail")).unwrap();
    std::fs::write(dir.join("b.ail"), "fn main()\n    print(half(2))\n").unwrap();
    let screen = |more: &[&str]| {
        let mut args = vec!["run".to_string(), tool("tools/edit/edit.ail"), "--print".to_string()];
        for f in ["--at=5:4", "--key=ctrl+shift+h", "--key=enter", "--type=halve", "--key=enter"].iter().chain(more) {
            args.push(f.to_string());
        }
        args.push("a.ail".to_string());
        let out = ail().args(&args).current_dir(&dir).env("AIL", env!("CARGO_BIN_EXE_ail")).output().unwrap();
        assert!(out.status.success(), "{more:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let preview = screen(&[]);
    for want in ["replace `half` with `halve`: 4 of 4 lines, 2 files", " a.ail:12:10: x := half(4)? -> x := halve(4)?", " b.ail:2:11: print(half(2)) -> print(halve(2))"] {
        assert!(preview.contains(want), "preview lacks {want:?}:\n{preview}");
    }
    let skipped = screen(&["--key=down", "--key=space", "--key=ctrl+enter"]);
    assert!(skipped.contains(" 12     x := half(4)?") && skipped.contains("  5 fn halve(n int)"), "skipped row:\n{skipped}");
    let marked = screen(&["--key=down", "--key=space"]);
    assert!(marked.contains("3 of 4 lines, 2 files") && marked.contains("[ ] a.ail:12:10:"), "marked:\n{marked}");
    let applied = screen(&["--key=ctrl+enter"]);
    for want in [" a.ail *   b.ail * ", "  5 fn halve(n int) int ! HalfError", "| replaced `half` in 2 files, not saved"] {
        assert!(applied.contains(want), "applied lacks {want:?}:\n{applied}");
    }
    assert!(std::fs::read_to_string(dir.join("b.ail")).unwrap().contains("half(2)"), "nothing is saved");
    std::fs::remove_dir_all(&dir).ok();
}

/// Drives the editor headless (`--print` with keys) against the real
/// `ail api` and `ail fmt`, so a change to their output breaks this test
/// rather than the editor: hover, completion of locals and members, F12,
/// F2 rename and formatting on a fixture.
#[test]
fn editor_drives_the_real_semantic_api() {
    let dir = std::env::temp_dir().join(format!("ail-edit-smoke-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("smoke.ail");
    std::fs::copy(tool("tools/edit/testdata/smoke.ail"), &file).unwrap();
    let file = file.display().to_string();
    let screen = |flags: &[&str]| {
        let mut args = vec!["run".to_string(), tool("tools/edit/edit.ail"), "--print".to_string()];
        args.extend(flags.iter().map(|f| f.to_string()));
        args.push(file.clone());
        let out = ail().args(&args).env("AIL", env!("CARGO_BIN_EXE_ail")).output().unwrap();
        assert!(out.status.success(), "{flags:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let hover = screen(&["--at=11:15", "--key=ctrl+i"]);
    assert!(hover.contains(" fn add(a int, b int) int\n") && hover.contains(" ## Adds.\n"), "hover:\n{hover}");
    let locals = screen(&["--at=12:5", "--key=end", "--key=enter", "--type=to", "--key=ctrl+space"]);
    assert!(locals.contains(" total int (binding) "), "locals:\n{locals}");
    let members = screen(&["--at=12:5", "--key=end", "--key=enter", "--type=p."]);
    assert!(members.contains(" x x int ") && members.contains(" y y int "), "members:\n{members}");
    let definition = screen(&["--at=12:11", "--key=f12"]);
    assert!(definition.contains("Ln 11, Col 5"), "F12:\n{definition}");
    let rename = screen(&["--at=10:5", "--key=f2", "--type=t", "--key=enter"]);
    assert!(rename.contains("total := add(pt.x, pt.y)") && rename.contains("| renamed 3 `p`"), "F2:\n{rename}");
    // With the file broken `--refs` can't answer, so F2 falls back to words.
    let by_word = screen(&["--at=10:5", "--type=(", "--key=f2", "--type=t", "--key=enter"]);
    assert!(by_word.contains("total := add(pt.x, pt.y)") && !by_word.contains("Ln 10, Col 5"), "F2 by word:\n{by_word}");
    let squeeze = ["--at=7:6", "--key=delete", "--key=right", "--key=delete"];
    let squeezed = screen(&squeeze);
    assert!(squeezed.contains("  7     a+b\n"), "squeeze:\n{squeezed}");
    let format = screen(&[&squeeze[..], &["--key=alt+shift+f"]].concat());
    assert!(format.contains("  7     a + b\n") && format.contains("| formatted"), "format:\n{format}");
    std::fs::remove_dir_all(&dir).ok();
}

/// The README's key table is the editor's `--keys` output, the same list
/// the command palette shows (ADR 0291).
#[test]
fn editor_key_table_matches_the_readme() {
    let out = ail().args(["run", &tool("tools/edit/edit.ail"), "--keys"]).output().unwrap();
    assert!(out.status.success());
    let table = String::from_utf8_lossy(&out.stdout);
    let readme = std::fs::read_to_string(tool("tools/edit/README.md")).unwrap();
    assert!(readme.contains(table.trim_end()), "tools/edit/README.md's key table differs from `--keys`; paste its output:\n{table}");
}

/// The agent panel against the real `ail` outputs (ADRs 0312–0313, 0331):
/// Ctrl-I info, Shift-F5 trace with a trap, Ctrl-Shift-E failure paths,
/// Ctrl-Shift-I program card and Ctrl-Shift-C coverage on a fixture, so an
/// output change breaks a test.
#[test]
fn editor_agent_panel_reads_the_real_outputs() {
    let dir = std::env::temp_dir().join(format!("ail-edit-agent-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("agent.ail");
    std::fs::copy(tool("tools/edit/testdata/agent.ail"), &file).unwrap();
    let file = file.display().to_string();
    let screen = |flags: &[&str]| {
        let mut args = vec!["run".to_string(), tool("tools/edit/edit.ail"), "--print".to_string()];
        args.extend(flags.iter().map(|f| f.to_string()));
        args.push(file.clone());
        let out = ail().args(&args).env("AIL", env!("CARGO_BIN_EXE_ail")).output().unwrap();
        assert!(out.status.success(), "{flags:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let info = screen(&["--at=12:10", "--key=ctrl+i"]);
    for want in [" half: int ! HalfError\n", "     requires n >= 0\n", "     effects: fail\n", "     fails with HalfError\n", " ## Halves an even number.\n", " fn:half: 2 uses in this file"] {
        assert!(info.contains(want), "info lacks {want:?}:\n{info}");
    }
    let trace = screen(&["--key=shift+f5"]);
    for want in ["   call half(n = 4)  12:10", " requires n >= 0 held  6:5", " trap[type.coll.4] index 2 is out of bounds", "   in main 15:12  locals: x = 2, v = [1, 2]", " > 2"] {
        assert!(trace.contains(want), "trace lacks {want:?}:\n{trace}");
    }
    let jumped = screen(&["--key=shift+f5", "--key=down", "--key=enter"]);
    assert!(jumped.contains("Ln 12, Col 10"), "trace jump:\n{jumped}");
    let paths = screen(&["--key=ctrl+shift+e"]);
    for want in [" fn half fails with HalfError", "   8:16 return: HalfError", "   12:17 propagate half(): HalfError", "   18:15 handle (default) half(): HalfError"] {
        assert!(paths.contains(want), "failure paths lack {want:?}:\n{paths}");
    }
    let card = screen(&["--key=ctrl+shift+i"]);
    for want in [" program card ", " entry: main in agent.ail", " tests: halves", "   fn half(n int) int ! HalfError  [fail]", "       requires n >= 0", "       fails with HalfError"] {
        assert!(card.contains(want), "program card lacks {want:?}:\n{card}");
    }
    let card_jump = screen(&["--key=ctrl+shift+i", "--key=down", "--key=down", "--key=down", "--key=down", "--key=down", "--key=enter"]);
    assert!(card_jump.contains("Ln 5, Col 4"), "card jump:\n{card_jump}");
    let problems = screen(&["--at=13:5", "--type=x", "--key=ctrl+shift+m"]);
    for want in [" problems: 1 errors, 0 others", "agent.ail:13:5: error[expr.name.1] unknown name `xprint`"] {
        assert!(problems.contains(want), "problems lack {want:?}:\n{problems}");
    }
    let problem_jump = screen(&["--at=13:5", "--type=x", "--key=end", "--key=ctrl+shift+m", "--key=enter"]);
    assert!(problem_jump.contains("Ln 13, Col 5"), "problem jump:\n{problem_jump}");
    // ADR 0353: the shown list follows edits after an idle pause and keeps
    // the selection on the same diagnostic (`zprint`), so F4 goes on to the
    // new first row.
    let live = screen(&["--at=13:5", "--type=x", "--at=15:5", "--type=z", "--key=ctrl+shift+m", "--key=down", "--at=9:5", "--type=q", "--idle", "--key=f4"]);
    assert!(live.contains(" problems: 3 errors, 0 others") && live.contains("agent.ail:9:5: error[expr.name.1] unknown name `qn`") && live.contains("Ln 9, Col 5"), "live problems:\n{live}");
    let tests_on = screen(&["--key=ctrl+shift+u"]);
    assert!(tests_on.contains("0 warnings, tests 1/1 pass"), "tests on save:\n{tests_on}");
    let covered = screen(&["--key=ctrl+shift+c"]);
    assert!(covered.contains("\n  8!        return Odd(n)\n") && covered.contains("\n  9     n / 2\n") && covered.contains("| coverage: lines 2/7"), "coverage:\n{covered}");
    // Saving with tests on reruns them; this changes the file, so it's last.
    let failing = screen(&["--key=ctrl+shift+u", "--at=9:9", "--type=1", "--key=ctrl+s"]);
    assert!(failing.contains("tests 1 FAILED of 1"), "tests after save:\n{failing}");
    std::fs::remove_dir_all(&dir).ok();
}

/// `tools/pdf` (ADR 0520) checks without diagnostics, passes its `@test`s,
/// and reads its fixtures' text layers: a classic table, a cross-reference
/// stream with an object stream, a LibreOffice PDF with subset fonts, and
/// a damaged table it must rebuild.
#[test]
fn pdf_reader_checks_clean_and_reads_its_fixtures() {
    let path = tool("tools/pdf/pdf.ail");
    let check = ail().args(["check", &path]).output().unwrap();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check.status.success() && stderr.is_empty(), "ail check {path}:\n{stderr}");
    let test = ail().args(["test", &path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test.status.success() && stdout.contains(" 0 failed"), "ail test {path}:\n{stdout}");
    let pdf = |args: &[&str]| {
        let out = ail().arg("run").arg(&path).args(args).output().unwrap();
        (out.status.code(), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
    };
    for (pdf_name, txt) in [("simple", "simple"), ("xrefstream", "xrefstream"), ("report", "report"), ("damaged", "simple")] {
        let (code, out, err) = pdf(&["text", &tool(&format!("tools/pdf/testdata/{pdf_name}.pdf"))]);
        let want = std::fs::read_to_string(tool(&format!("tools/pdf/testdata/{txt}.txt"))).unwrap();
        assert!(code == Some(0) && out == want, "pdf text {pdf_name}.pdf:\n{out}{err}");
    }
    let report = tool("tools/pdf/testdata/report.pdf");
    let (code, out, _) = pdf(&["grep", "LINE BREAK can be tested", &report]);
    assert!(code == Some(0) && out.starts_with("1: enough to wrap onto more than one line"), "pdf grep across lines:\n{out}");
    let (code, out, _) = pdf(&["grep", "--json", "résumé", &report]);
    assert!(code == Some(0) && out.contains("\"boxes\":[{\"x0\":327.56,"), "pdf grep --json:\n{out}");
    let (code, _, _) = pdf(&["grep", "not in the file", &report]);
    assert_eq!(code, Some(1), "pdf grep with no hit exits 1");
    let (_, out, _) = pdf(&["json", &report, "--page", "1"]);
    assert!(out.contains("\"uri\":\"https://example.com/ledger\"") && out.contains("{\"title\":\"Appendix\",\"depth\":0.0,\"page\":2.0}"), "pdf json:\n{out}");
    let (_, out, _) = pdf(&["info", &tool("tools/pdf/testdata/scan.pdf")]);
    assert!(out.contains("image-only pages (scans, read by OCR when shown or searched): 1"), "pdf info on a scan:\n{out}");
    let (_, out, _) = pdf(&["view", &report, "--find", "without OCR", "--print", "--rows", "20"]);
    assert!(out.contains(" report.pdf  page 1/2") && out.contains("Searching a PDF without OCR works") && out.contains(" match 1 of 1"), "pdf view --print:\n{out}");
    // ADR 0511: the viewer reads only the page it shows, and says what is on screen.
    let (_, out, _) = pdf(&["view", &report, "--json", "--page", "2"]);
    assert!(out.contains("\"pages_read\":1.0") && out.contains("\"visible\":[{\"text\":\"Appendix\""), "pdf view --json:\n{out}");
    let (code, out, _) = pdf(&["grep", "--first", "the", &report]);
    assert!(code == Some(0) && !out.contains("2: "), "pdf grep --first stops after page 1:\n{out}");
    // ADR 0512: a Group 4 fax scan is read by OCR when asked for, and not with --no-ocr.
    let scan = tool("tools/pdf/testdata/scan-ocr.pdf");
    let (code, out, err) = pdf(&["text", &scan]);
    let want = std::fs::read_to_string(tool("tools/pdf/testdata/scan-ocr.txt")).unwrap();
    assert!(code == Some(0) && out == want, "pdf text scan-ocr.pdf (OCR):\n{out}{err}");
    let (code, out, _) = pdf(&["grep", "western library cooperative", &scan]);
    assert!(code == Some(0) && out.starts_with("1: the growth came"), "pdf grep on a scan:\n{out}");
    let (code, _, _) = pdf(&["grep", "--no-ocr", "western", &scan]);
    assert_eq!(code, Some(1), "pdf grep --no-ocr finds nothing on a scan");
    let (_, out, _) = pdf(&["json", &scan]);
    assert!(out.contains("\"image_only\":true,\"ocr\":true,\"confidence\":"), "pdf json on a scan:\n{out}");
}
