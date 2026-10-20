#!/usr/bin/env python3
"""Before/after benchmark table (ADR 0492). Called by tools/bench/run.sh.

Tokens: tools/tokbench's mean over the offline tokenizers, for every program in
examples/, the agent-tool corpus and tools/perfbench/programs.

Speed: min wall time of `ail run` for the "before" and "after" bootstrap
binaries (bytecode tier), `ail run --no-bytecode` on the after binary (tree
walker), and the Python and C twins where they exist. Lower is better; a ratio
below 1 is a win for the after binary.

Standard library only. Output is the table, or JSON with --format json.
"""
import argparse
import json
import math
import re
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# examples/ programs that are also speed programs (the perfbench set).
PERF_EXAMPLES = {"interp", "json", "sheet", "logscan", "tasks", "build"}


def programs(root):
    """(key, path relative to root, is_speed_program) for every benchmark program."""
    found = []
    for p in sorted((root / "examples").glob("*.ail")):
        found.append(("examples/" + p.stem, p.relative_to(root), p.stem in PERF_EXAMPLES))
    for p in sorted((root / "tools/tokbench/corpus/programs").glob("*/*.ail")):
        found.append(("agent/" + p.stem, p.relative_to(root), True))
    for p in sorted((root / "tools/perfbench/programs").glob("*.ail")):
        found.append(("perf/" + p.stem, p.relative_to(root), True))
    return found


def count_tokens(tokbench, root, paths):
    """Map absolute file path -> mean token count, from tokbench --json."""
    if not paths:
        return {}
    out = subprocess.run([tokbench, "--json", *paths], cwd=root, capture_output=True,
                         text=True, check=True).stdout
    # The wrapper may print setup lines before the JSON array.
    start = re.search(r"^\[", out, re.M)
    rows = [] if start is None else json.loads(out[start.start():])
    return {r["file"]: r["mean"] for r in rows}


def timed(cmd, cwd, runs):
    """(min seconds, median seconds, return code, stdout) over `runs` runs."""
    times, last = [], None
    for _ in range(runs):
        t0 = time.perf_counter()
        last = subprocess.run(cmd, cwd=cwd, stdin=subprocess.DEVNULL,
                              stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        times.append(time.perf_counter() - t0)
    return min(times), statistics.median(times), last.returncode, last.stdout


def supports_no_bytecode(binary, hello, root):
    """True when this binary accepts --no-bytecode (ADR 0361)."""
    p = subprocess.run([str(binary), "run", "--no-bytecode", str(hello)], cwd=root,
                       stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL)
    return p.returncode == 0


def compile_c(src, workdir):
    """Compile a C twin with cc -O2; None when there is no compiler or it fails."""
    exe = Path(workdir) / (src.stem + "_c")
    try:
        p = subprocess.run(["cc", "-O2", "-o", str(exe), str(src)], capture_output=True)
    except FileNotFoundError:
        return None
    return exe if p.returncode == 0 else None


def geomean(values):
    values = [v for v in values if v and v > 0]
    return math.exp(sum(math.log(v) for v in values) / len(values)) if values else None


def fmt_ratio(v):
    return "-" if v is None else f"{v:.2f}"


def fmt_pct(v):
    return "-" if v is None else f"{v:+.1f}%"


def fmt_s(v):
    return "-" if v is None else f"{v:.3f}"


def print_table(headers, rows):
    widths = [max(len(str(h)), *(len(str(r[i])) for r in rows)) for i, h in enumerate(headers)]
    line = lambda cells: "  ".join(
        str(c).ljust(w) if i == 0 else str(c).rjust(w) for i, (c, w) in enumerate(zip(cells, widths)))
    print(line(headers))
    print(line(["-" * w for w in widths]))
    for r in rows:
        print(line(r))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True, type=Path, help="repository at the working tree")
    ap.add_argument("--before-root", required=True, type=Path, help="worktree at the before revision")
    ap.add_argument("--before-label", required=True)
    ap.add_argument("--after-label", required=True)
    ap.add_argument("--tokbench", required=True, help="path to tools/tokbench/run.sh")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--format", choices=["table", "json"], default="table")
    args = ap.parse_args()
    if args.runs < 1:
        sys.exit("bench: --runs must be at least 1")

    root = args.root.resolve()
    before_root = args.before_root.resolve()
    after_bin = root / "bootstrap/target/release/ail"
    before_bin = before_root / "bootstrap/target/release/ail"
    hello = root / "examples/hello.ail"
    progs = programs(root)

    # Tokens: the same file name in each revision (missing before -> None).
    after_paths, before_paths = {}, {}
    for key, rel, _ in progs:
        after_paths[key] = str(root / rel)
        before_paths[key] = str(before_root / rel) if (before_root / rel).exists() else None
    after_tok = count_tokens(args.tokbench, root, list(after_paths.values()))
    before_tok = count_tokens(args.tokbench, root,
                              [p for p in before_paths.values() if p is not None])
    token_rows = []
    for key, _, _ in progs:
        b = before_tok.get(before_paths[key]) if before_paths[key] else None
        a = after_tok.get(after_paths[key])
        change = (a - b) / b * 100 if b and a is not None else None
        token_rows.append({"program": key, "before": b, "after": a, "change_pct": change})

    # Speed.
    walk_ok = supports_no_bytecode(after_bin, hello, root)
    speed_rows = []
    with tempfile.TemporaryDirectory() as workdir:
        for key, rel, is_speed in progs:
            if not is_speed:
                continue
            ail = root / rel
            row = {"program": key, "before_s": None, "after_s": None, "walk_s": None,
                   "py_s": None, "c_s": None, "output": "-"}
            if (before_root / rel).exists():
                bmin, _, brc, bout = timed([str(before_bin), "run", str(before_root / rel)],
                                           before_root, args.runs)
                row["before_s"] = bmin
            amin, _, arc, aout = timed([str(after_bin), "run", str(ail)], root, args.runs)
            row["after_s"] = amin
            if walk_ok:
                row["walk_s"] = timed([str(after_bin), "run", "--no-bytecode", str(ail)],
                                      root, args.runs)[0]
            if row["before_s"] is not None:
                if brc != 0 or arc != 0:
                    row["output"] = "err"
                else:
                    row["output"] = "same" if bout == aout else "DIFF"
            py = ail.with_suffix(".py")
            if py.exists():
                row["py_s"] = timed([sys.executable, str(py)], root, args.runs)[0]
            c_src = ail.with_suffix(".c")
            if c_src.exists():
                c_exe = compile_c(c_src, workdir)
                if c_exe is not None:
                    row["c_s"] = timed([str(c_exe)], root, args.runs)[0]
            speed_rows.append(row)

    # Summaries: total tokens, geometric-mean time ratios.
    both = [r for r in token_rows if r["before"] is not None and r["after"] is not None]
    tok_before = sum(r["before"] for r in both)
    tok_after = sum(r["after"] for r in both)
    summary = {
        "tokens_before": tok_before,
        "tokens_after": tok_after,
        "tokens_change_pct": (tok_after - tok_before) / tok_before * 100 if tok_before else None,
        "speed_after_over_before": geomean([r["after_s"] / r["before_s"] for r in speed_rows
                                            if r["before_s"] and r["after_s"]]),
        "speed_after_over_walk": geomean([r["after_s"] / r["walk_s"] for r in speed_rows
                                          if r["walk_s"] and r["after_s"]]),
        "speed_after_over_python": geomean([r["after_s"] / r["py_s"] for r in speed_rows
                                            if r["py_s"] and r["after_s"]]),
        "speed_after_over_c": geomean([r["after_s"] / r["c_s"] for r in speed_rows
                                       if r["c_s"] and r["after_s"]]),
    }

    if args.format == "json":
        print(json.dumps({"before": args.before_label, "after": args.after_label,
                          "runs": args.runs, "tokens": token_rows, "speed": speed_rows,
                          "summary": summary}, indent=2))
        return

    print(f"Before: {args.before_label}    After: {args.after_label}    runs per program: {args.runs}")
    print()
    print("Tokens (tokbench mean over the offline tokenizers; lower is better)")
    print_table(["program", "before", "after", "change"],
                [[r["program"], r["before"] or "-", r["after"] or "-", fmt_pct(r["change_pct"])]
                 for r in token_rows])
    print(f"TOTAL (both present)  {tok_before}  {tok_after}  {fmt_pct(summary['tokens_change_pct'])}")
    print()
    print("Speed (min wall seconds; ratios are after / other, below 1.00 is faster)")
    print_table(
        ["program", "before", "after", "walker", "python", "c", "a/b", "a/walker", "a/py", "a/c", "output"],
        [[r["program"], fmt_s(r["before_s"]), fmt_s(r["after_s"]), fmt_s(r["walk_s"]),
          fmt_s(r["py_s"]), fmt_s(r["c_s"]),
          fmt_ratio(r["after_s"] / r["before_s"] if r["before_s"] and r["after_s"] else None),
          fmt_ratio(r["after_s"] / r["walk_s"] if r["walk_s"] and r["after_s"] else None),
          fmt_ratio(r["after_s"] / r["py_s"] if r["py_s"] and r["after_s"] else None),
          fmt_ratio(r["after_s"] / r["c_s"] if r["c_s"] and r["after_s"] else None),
          r["output"]] for r in speed_rows])
    print()
    print("Geometric mean over programs present in both columns:")
    print(f"  after / before  {fmt_ratio(summary['speed_after_over_before'])}")
    print(f"  after / walker  {fmt_ratio(summary['speed_after_over_walk'])}")
    print(f"  after / python  {fmt_ratio(summary['speed_after_over_python'])}")
    print(f"  after / c       {fmt_ratio(summary['speed_after_over_c'])}")
    print("  native          not measured: no `ail build` yet (ADR 0492)")
    if any(r["output"] == "DIFF" for r in speed_rows):
        print("warning: some programs print different output before and after", file=sys.stderr)


if __name__ == "__main__":
    main()
