#!/usr/bin/env python3
"""Before/after benchmark table (ADR 0492). Called by tools/bench/run.sh.

Tokens: tools/tokbench's mean over the offline tokenizers, for every program in
examples/, the agent-tool corpus and tools/perfbench/programs.

Speed: min wall time over --runs runs of each program, with its NAME.args
passed after the file and its working directory set to the program's own
directory. Columns: the "before" build, the "after" build, the Python twin and
the C twin where they exist. Lower is better.

Correctness gate: a program is compared only when every run it takes part in
exits 0 and prints what the after binary prints. A program that fails the gate
is listed as excluded and left out of every ratio it would affect. The after
output is checked against the Python and C twins, and the before output against
the after output.

Groups, reported as separate geometric means:
  agent     the agent-tool corpus (start-up bound, runs in milliseconds)
  compute   tools/perfbench/programs (CPU-bound classics)
  examples  the perfbench programs in examples/ (interp, json, sheet, ...)

Standard library only. Output is the table, or JSON with --format json.
"""
import argparse
import json
import math
import re
import shlex
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# examples/ programs that are also speed programs (the perfbench set).
PERF_EXAMPLES = {"interp", "json", "sheet", "logscan", "tasks", "build"}
# tools/perfbench classics (the CPU-bound programs with Python and C twins);
# the rest of tools/perfbench (loops, parsum, tensor*, race_smoke) is "micro".
CLASSICS = {"binarytrees", "collatz", "fannkuch", "fasta", "mandelbrot", "matmul",
            "nbody", "sieve", "spectralnorm", "wordfreq"}
GROUPS = ["agent", "compute", "micro", "examples"]
# Programs counted for tokens only: their run time is not the AIL program's.
NO_TIMING = {"build"}


def programs(root):
    """Dicts for every benchmark program: key, path, group, and whether it is timed."""
    found = []
    for p in sorted((root / "examples").glob("*.ail")):
        found.append({"key": "examples/" + p.stem, "rel": p.relative_to(root),
                      "group": "examples", "speed": p.stem in PERF_EXAMPLES})
    for p in sorted((root / "tools/tokbench/corpus/programs").glob("*/*.ail")):
        # agent/build runs the repo's own ail binary, so its time is not the program's.
        found.append({"key": "agent/" + p.stem, "rel": p.relative_to(root),
                      "group": "agent", "speed": p.stem not in NO_TIMING})
    for p in sorted((root / "tools/perfbench/programs").glob("*.ail")):
        group = "compute" if p.stem in CLASSICS else "micro"
        found.append({"key": group + "/" + p.stem, "rel": p.relative_to(root),
                      "group": group, "speed": True})
    return found


def program_args(ail):
    """Arguments from NAME.args next to NAME.ail (program arguments, ADR 0175)."""
    args_file = ail.with_suffix(".args")
    return shlex.split(args_file.read_text()) if args_file.exists() else []


def count_tokens(tokbench, root, paths):
    """Map file path -> mean token count, from tokbench --json."""
    if not paths:
        return {}
    p = subprocess.run([tokbench, "--json", *paths], cwd=root, capture_output=True, text=True)
    if p.returncode != 0:
        # No tokenizer files (offline machine): report speed without token counts.
        print("bench: token counts skipped: tokbench exited %d: %s"
              % (p.returncode, p.stderr.strip()[-300:]), file=sys.stderr)
        return {}
    out = p.stdout
    # The wrapper may print setup lines before the JSON array.
    start = re.search(r"^\[", out, re.M)
    rows = [] if start is None else json.loads(out[start.start():])
    return {r["file"]: r["mean"] for r in rows}


def run(cmd, cwd):
    """One run: (seconds, return code, stdout bytes)."""
    t0 = time.perf_counter()
    p = subprocess.run(cmd, cwd=cwd, stdin=subprocess.DEVNULL,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return time.perf_counter() - t0, p.returncode, p.stdout


def timed(cmd, cwd, runs):
    """(min seconds, median seconds, return code, stdout) over `runs` runs.

    The return code and stdout come from the first run; a later run that
    disagrees on either marks the timing unstable."""
    times, first, stable = [], None, True
    for _ in range(runs):
        dt, rc, out = run(cmd, cwd)
        times.append(dt)
        if first is None:
            first = (rc, out)
        elif (rc, out) != first:
            stable = False
    return min(times), statistics.median(times), first[0], first[1], stable


def compile_c(src, workdir):
    """Compile a C twin with cc -O2; None when there is no compiler or it fails."""
    exe = Path(workdir) / (src.stem + "_c")
    try:
        p = subprocess.run(["cc", "-O2", "-o", str(exe), str(src), "-lm"], capture_output=True)
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


def measure_speed(prog, ctx):
    """One speed row. Each ratio is only kept when its inputs passed the gate."""
    ail = ctx["root"] / prog["rel"]
    before_ail = ctx["before_root"] / prog["rel"]
    work = ail.parent
    pargs = program_args(ail)
    row = {"program": prog["key"], "group": prog["group"], "notes": [],
           "before_s": None, "after_s": None, "py_s": None, "c_s": None,
           "output": "-"}

    # After: the reference output every other column is checked against.
    amin, _, arc, aout, astable = timed([str(ctx["after_bin"]), "run", str(ail), *pargs],
                                        work, ctx["runs"])
    if arc != 0 or not astable:
        row["notes"].append(f"after exits {arc}" if arc != 0 else "after is unstable")
        row["excluded"] = True
        return row
    row["after_s"] = amin

    # Before: must exit 0 and print what after prints.
    if before_ail.exists():
        bmin, _, brc, bout, _ = timed([str(ctx["before_bin"]), "run", str(before_ail), *pargs],
                                      work, ctx["runs"])
        if brc != 0:
            row["output"] = "before err"
            row["notes"].append(f"before exits {brc}")
        elif bout != aout:
            row["output"] = "DIFF"
            row["notes"].append("before output differs from after")
        else:
            row["output"] = "same"
            row["before_s"] = bmin

    # Python twin: must print what after prints.
    py = ail.with_suffix(".py")
    if py.exists():
        pmin, _, prc, pout, _ = timed([sys.executable, str(py), *pargs], work, ctx["runs"])
        if prc == 0 and pout == aout:
            row["py_s"] = pmin
        else:
            row["notes"].append(f"python twin exits {prc} or prints different output")

    # C twin: must print what after prints.
    c_src = ail.with_suffix(".c")
    if c_src.exists():
        c_exe = compile_c(c_src, ctx["workdir"])
        if c_exe is None:
            row["notes"].append("C twin does not compile")
        else:
            cmin, _, crc, cout, _ = timed([str(c_exe), *pargs], work, ctx["runs"])
            if crc == 0 and cout == aout:
                row["c_s"] = cmin
            else:
                row["notes"].append(f"C twin exits {crc} or prints different output")

    row["excluded"] = False
    return row


def group_summary(rows, group):
    """Geometric means of each ratio over the rows in one group (passing rows only)."""
    rs = [r for r in rows if r["group"] == group and not r["excluded"]]

    def ratio(num, den):
        vals = [r[num] / r[den] for r in rs if r[num] and r[den]]
        return geomean(vals), len(vals)

    return {
        "after_before": ratio("after_s", "before_s"),
        "after_python": ratio("after_s", "py_s"),
        "after_c": ratio("after_s", "c_s"),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", required=True, type=Path, help="repository at the working tree")
    ap.add_argument("--before-root", required=True, type=Path, help="worktree at the before revision")
    ap.add_argument("--before-label", required=True)
    ap.add_argument("--after-label", required=True)
    ap.add_argument("--tokbench", required=True, help="path to tools/tokbench/run.sh")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--format", choices=["table", "json"], default="table")
    ap.add_argument("--card-ail", type=Path, help="AIL language card, for the card-plus-program line")
    ap.add_argument("--card-py", type=Path, help="Python reference card, for the card-plus-program line")
    args = ap.parse_args()
    if args.runs < 1:
        sys.exit("bench: --runs must be at least 1")

    root = args.root.resolve()
    before_root = args.before_root.resolve()
    progs = programs(root)

    # Tokens: the same file name in each revision (missing before -> None).
    after_paths = {p["key"]: str(root / p["rel"]) for p in progs}
    before_paths = {p["key"]: str(before_root / p["rel"]) if (before_root / p["rel"]).exists() else None
                    for p in progs}
    # Python twins of the agent corpus and perfbench programs, for the card line.
    twin_paths = {p["key"]: str((root / p["rel"]).with_suffix(".py")) for p in progs
                  if (root / p["rel"]).with_suffix(".py").exists()}
    after_tok = count_tokens(args.tokbench, root, list(after_paths.values()) + list(twin_paths.values()))
    before_tok = count_tokens(args.tokbench, root, [v for v in before_paths.values() if v])
    cards = [c for c in (args.card_ail, args.card_py) if c and c.exists()]
    card_tok = count_tokens(args.tokbench, root, [str(c) for c in cards])
    card_ail = card_tok.get(str(args.card_ail)) if args.card_ail and args.card_ail.exists() else None
    card_py = card_tok.get(str(args.card_py)) if args.card_py and args.card_py.exists() else None
    token_rows = []
    for p in progs:
        k = p["key"]
        b = before_tok.get(before_paths[k]) if before_paths[k] else None
        a = after_tok.get(after_paths[k])
        change = (a - b) / b * 100 if b and a is not None else None
        py_tok = after_tok.get(twin_paths[k]) if k in twin_paths else None
        token_rows.append({"program": k, "before": b, "after": a, "change_pct": change,
                           "python_twin": py_tok})

    # Speed.
    after_bin = root / "ail"
    before_bin = before_root / "ail"
    speed_progs = [p for p in progs if p["speed"]]
    speed_rows = []
    with tempfile.TemporaryDirectory() as workdir:
        ctx = {"root": root, "before_root": before_root, "after_bin": after_bin,
               "before_bin": before_bin, "runs": args.runs, "workdir": workdir}
        for p in speed_progs:
            speed_rows.append(measure_speed(p, ctx))

    group_stats = {g: group_summary(speed_rows, g) for g in GROUPS}
    # Card-plus-program: the AIL card plus AIL programs, against the Python
    # reference card plus the Python twins, over programs with a twin.
    twinned = [r for r in token_rows if r["python_twin"] is not None and r["after"] is not None]
    card_line = None
    if card_ail is not None and card_py is not None and twinned:
        ail_total = sum(r["after"] for r in twinned)
        py_total = sum(r["python_twin"] for r in twinned)
        card_line = {"programs": len(twinned), "ail_card": card_ail, "ail_programs": ail_total,
                     "ail_total": card_ail + ail_total, "py_card": card_py, "py_programs": py_total,
                     "py_total": card_py + py_total}
    both = [r for r in token_rows if r["before"] is not None and r["after"] is not None]
    tok_before = sum(r["before"] for r in both)
    tok_after = sum(r["after"] for r in both)
    excluded = [r for r in speed_rows if r["excluded"]]
    noted = [r for r in speed_rows if r["notes"] and not r["excluded"]]

    if args.format == "json":
        print(json.dumps({
            "before": args.before_label, "after": args.after_label, "runs": args.runs,
            "tokens": token_rows, "speed": speed_rows,
            "tokens_total": {"before": tok_before, "after": tok_after,
                             "change_pct": (tok_after - tok_before) / tok_before * 100 if tok_before else None},
            "groups": {g: {k: {"ratio": v[0], "n": v[1]} for k, v in s.items()}
                       for g, s in group_stats.items()},
            "excluded": [r["program"] for r in excluded],
            "card_plus_program": card_line,
        }, indent=2))
        return

    print(f"Before: {args.before_label}")
    print(f"After:  {args.after_label}")
    print(f"Timing runs per program: {args.runs}")
    print()
    print("Tokens (tokbench mean over the offline tokenizers; lower is better)")
    print_table(["program", "before", "after", "change"],
                [[r["program"], r["before"] or "-", r["after"] or "-", fmt_pct(r["change_pct"])]
                 for r in token_rows])
    print(f"TOTAL (both present)  {tok_before}  {tok_after}  {fmt_pct((tok_after - tok_before) / tok_before * 100 if tok_before else None)}")
    print()
    print("Speed (min wall seconds; ratios are after / other, below 1.00 is faster)")
    print_table(
        ["program", "before", "after", "python", "c", "a/b", "a/py", "a/c", "output"],
        [[r["program"], fmt_s(r["before_s"]), fmt_s(r["after_s"]), fmt_s(r["py_s"]), fmt_s(r["c_s"]),
          fmt_ratio(r["after_s"] / r["before_s"] if r["before_s"] and r["after_s"] else None),
          fmt_ratio(r["after_s"] / r["py_s"] if r["py_s"] and r["after_s"] else None),
          fmt_ratio(r["after_s"] / r["c_s"] if r["c_s"] and r["after_s"] else None),
          r["output"]] for r in speed_rows if not r["excluded"]])
    print()
    print("Geometric means per group (n = programs in that ratio)")
    labels = {"agent": "agent (start-up bound)", "compute": "compute (classics)",
              "micro": "micro (loops, tensor)", "examples": "examples (perf set)"}
    for g in GROUPS:
        s = group_stats[g]
        parts = []
        for key, name in [("after_before", "a/b"), ("after_python", "a/py"), ("after_c", "a/c")]:
            ratio, n = s[key]
            parts.append(f"{name} {fmt_ratio(ratio)} (n={n})")
        print(f"  {labels[g]:<26} " + "   ".join(parts))
    if card_line:
        c = card_line
        print()
        print(f"Card plus program, over {c['programs']} programs with a Python twin (tokbench mean):")
        print(f"  AIL:    card {c['ail_card']:.0f} + programs {c['ail_programs']:.0f} = {c['ail_total']:.0f}")
        print(f"  Python: card {c['py_card']:.0f} + twins {c['py_programs']:.0f} = {c['py_total']:.0f}")
    print("  native                     not measured: no `ail build` on this platform yet (ADR 0492)")
    if excluded:
        print()
        print(f"Excluded from every ratio ({len(excluded)}):")
        for r in excluded:
            print(f"  {r['program']}: " + "; ".join(r["notes"]))
    if noted:
        print()
        print(f"Partly excluded ({len(noted)}): some columns dropped, see notes")
        for r in noted:
            print(f"  {r['program']}: " + "; ".join(r["notes"]))

    # Twin outputs: each corpus pair (tools/tokbench/corpus/programs) must print what its
    # Python twin prints. The sweep runs in a mirror tree with this checkout's ail (ADR 0029).
    sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools/tokbench"))
    import twin_outputs
    twins = twin_outputs.sweep()
    skipped = [r for r in twins if r["status"] == "skipped"]
    known = [r for r in twins if r["status"] == "known difference"]
    bad_twins = [r for r in twins if r["status"] not in ("match", "skipped", "known difference")]
    print()
    print(f"Twin outputs: {len(twins) - len(bad_twins) - len(skipped) - len(known)} of {len(twins)} corpus pairs print what the Python twin prints"
          f" ({len(known)} documented difference, {len(skipped)} skipped)")
    for r in bad_twins:
        detail = f" ({r['detail']})" if r.get("detail") else ""
        print(f"  {r['name']}: {r['status']}{detail}")

if __name__ == "__main__":
    main()
