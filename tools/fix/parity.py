#!/usr/bin/env python3
"""Compare the AIL fix machinery (compiler/fix) with the Rust one on source files.

For each file: the Rust `ail check --json` gives the settled fixes; a Rust build
patched to write the unsettled edits to $AIL_DUMP_EDITS gives their input; the
compiler's `fixes` command settles them and must print the same edits.

  tools/fix/parity.py RUST_AIL_WITH_DUMP COMPILER_BINARY FILE...

RUST_AIL_WITH_DUMP is a Rust `ail` built after `git apply tools/fix/dump-edits.patch`
(copy the binary away, then `git checkout bootstrap/src/diag.rs`). Besides the
settled edits, `compiler fixrun FILE [--dry-run] [--json]` is `ail fix` with the
Rust tool as the checker (AIL_RUST names it): stdout, stderr, exit status and the
written file are compared by tools/fix/run-parity.py.
"""
import json, os, subprocess, sys, tempfile


def esc(s):
    return s.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t")


def main():
    rust, comp, files = sys.argv[1], sys.argv[2], sys.argv[3:]
    checked = bad = fixes = 0
    for f in files:
        with tempfile.NamedTemporaryFile(delete=False) as t:
            dump = t.name
        os.unlink(dump)
        env = dict(os.environ, AIL_DUMP_EDITS=dump)
        r = subprocess.run([rust, "check", f, "--json"], capture_output=True, text=True, env=env)
        if not os.path.exists(dump):
            continue
        want = []
        for line in (r.stdout + r.stderr).splitlines():
            if not line.startswith("{"):
                continue
            d = json.loads(line)
            for fx in d.get("fixes", []):
                es = ";".join(f"{e['line']}:{e['col']}-{e['end_line']}:{e['end_col']}={esc(e['text'])}" for e in fx["edits"])
                want.append(f"{d['line']}:{d['col']}:{es}")
        got = subprocess.run([comp, "fixes", f, dump], capture_output=True, text=True).stdout.splitlines()
        got = [g for g in got if g]
        os.unlink(dump)
        checked += 1
        fixes += len(want)
        if got != want:
            bad += 1
            print("DIFF", f)
            for a, b in zip(got + [None] * 9, want + [None] * 9):
                if a != b:
                    print("  ail :", a)
                    print("  rust:", b)
                    break
    print(f"files with fixes {checked}, fixes {fixes}, differing {bad}")
    return 1 if bad else 0


sys.exit(main())
