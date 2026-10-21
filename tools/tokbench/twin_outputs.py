#!/usr/bin/env python3
"""Check that each corpus twin pair prints the same output.

Each program under corpus/programs/NAME/ has an AIL file NAME.ail and a Python
twin NAME.py. Many programs find the tools they need by path (`{here}/../ail`,
`{here}/../tools/...`), and the twins find the repository from their own file
location (`parents[5]`). So each pair runs in a mirror tree laid out like the
repository:

  MIRROR/ail                         wrapper for this checkout's launcher
  MIRROR/tools/garlic, pdf           symlinks to the real tool directories
  MIRROR/tools/tokbench/corpus/agent-r5/data   symlink to the fixture data
  MIRROR/run/                        the AIL program's directory (ail run NAME.ail)
  MIRROR/tools/tokbench/corpus/programs/NAME/  the Python twin, at repository depth

No corpus file changes, so token counts stay the same. Pass --fixture DIR to build
the mirror in DIR and keep it, for inspection or re-runs.

    tools/tokbench/twin_outputs.py              # every pair, temporary mirror
    tools/tokbench/twin_outputs.py --only NAME  # one pair (repeatable)
    tools/tokbench/twin_outputs.py --fixture DIR --json

Exit status is 1 when any pair disagrees or fails to run.
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROGRAMS = ROOT / "tools/tokbench/corpus/programs"
AGENT_DATA = ROOT / "tools/tokbench/corpus/agent-r5/data"
TIMEOUT_S = 120
# Programs that need outside network access; reported as skipped, not as mismatches.
NEEDS_NETWORK = {"fetch": "needs network access to example.com"}
# Documented print difference (README, "Known print difference: csv2json"): AIL prints a
# whole float without ".0" (ADR 0631). A pair listed here matches when that is the only difference.
KNOWN_DIFFERENCES = {"csv2json": "whole floats print without .0 (ADR 0631)"}
WHOLE_FLOAT = re.compile(r"(\d)\.0(?=[,\]\}\s]|$)", re.M)


def pairs(only):
    found = []
    for d in sorted(PROGRAMS.iterdir()):
        if not d.is_dir():
            continue
        if not ((d / f"{d.name}.ail").exists() and (d / f"{d.name}.py").exists()):
            continue
        if only and d.name not in only:
            continue
        found.append(d)
    return found


def link(path, target):
    if path.is_symlink() or path.exists():
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.symlink_to(target)


def build_mirror(m):
    m.mkdir(parents=True, exist_ok=True)
    wrapper = m / "ail"
    wrapper.write_text(f'#!/usr/bin/env bash\nexec "{ROOT}/ail" "$@"\n')
    wrapper.chmod(0o755)
    link(m / "tools/garlic", ROOT / "tools/garlic")
    link(m / "tools/pdf", ROOT / "tools/pdf")
    link(m / "tools/tokbench/corpus/agent-r5/data", AGENT_DATA)
    link(m / "examples", ROOT / "examples")


def copy_dir(src, dst):
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def run(cmd, cwd):
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, timeout=TIMEOUT_S)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return None, "", "timed out"


def first_difference(a, b):
    al, bl = a.splitlines(), b.splitlines()
    for i in range(max(len(al), len(bl))):
        x = al[i] if i < len(al) else "<end>"
        y = bl[i] if i < len(bl) else "<end>"
        if x != y:
            return f"line {i + 1}: AIL {x!r} vs Python {y!r}"
    return ""


def provide_data(dir_):
    """Programs read the repository's examples/ (relative to the working directory) and
    examples/agent-data (beside the program) when the program's own folder has none."""
    link(dir_ / "examples", ROOT / "examples")
    if not (dir_ / "agent-data").exists():
        link(dir_ / "agent-data", ROOT / "examples/agent-data")


def check_pair(m, d):
    name = d.name
    if name in NEEDS_NETWORK:
        return {"name": name, "status": "skipped", "detail": NEEDS_NETWORK[name]}
    run_dir = m / "run"
    copy_dir(d, run_dir)
    twin_dir = m / "tools/tokbench/corpus/programs" / name
    copy_dir(d, twin_dir)
    provide_data(run_dir)
    provide_data(twin_dir)

    # Both sides run from the program's own directory, so relative defaults ("tool.ail") resolve.
    a_rc, a_out, a_err = run([str(m / "ail"), "run", str(run_dir / f"{name}.ail")], run_dir)
    p_rc, p_out, p_err = run([sys.executable, str(twin_dir / f"{name}.py")], run_dir)

    row = {"name": name, "ail_rc": a_rc, "py_rc": p_rc}
    if a_rc is None:
        row["status"] = "ail timed out"
    elif p_rc is None:
        row["status"] = "python timed out"
    elif a_rc != 0:
        row["status"] = f"ail exit {a_rc}"
        row["detail"] = a_err.strip().splitlines()[0] if a_err.strip() else ""
    elif p_rc != 0:
        row["status"] = f"python exit {p_rc}"
        row["detail"] = p_err.strip().splitlines()[-1] if p_err.strip() else ""
    elif a_out == p_out:
        row["status"] = "match"
    elif name in KNOWN_DIFFERENCES and WHOLE_FLOAT.sub(r"\1", p_out) == a_out:
        row["status"] = "known difference"
        row["detail"] = KNOWN_DIFFERENCES[name]
    else:
        row["status"] = "MISMATCH"
        row["detail"] = first_difference(a_out, p_out)
    return row


def sweep(only=(), fixture=None):
    """Run every twin pair (or those named in `only`); return one row per pair."""
    todo = pairs(set(only))
    if fixture:
        mirror = Path(fixture).resolve()
        build_mirror(mirror)
        return [check_pair(mirror, d) for d in todo]
    cleanup = tempfile.mkdtemp(prefix="twin-outputs-")
    try:
        build_mirror(Path(cleanup))
        return [check_pair(Path(cleanup), d) for d in todo]
    finally:
        shutil.rmtree(cleanup, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--only", action="append", default=[], help="one pair by name (repeatable)")
    ap.add_argument("--fixture", help="build the mirror in this directory and keep it")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    rows = sweep(args.only, args.fixture)
    bad = [r for r in rows if r["status"] not in ("match", "skipped", "known difference")]
    if args.json:
        print(json.dumps({"pairs": rows, "bad": len(bad)}, indent=2))
    else:
        for r in rows:
            line = f"{r['name']:32} {r['status']}"
            if r.get("detail"):
                line += f"  ({r['detail']})"
            print(line)
        print(f"{len(rows)} pairs, {len(rows) - len(bad)} match, {len(bad)} not")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
