#!/usr/bin/env python3
"""Native coverage tracker (ADR 0583, ADR 0584). Measurement only; run run.sh.

For every program in examples/, tests/cases/ and tools/perfbench/programs/
it runs `ail build`, then classifies the program:

  built-match      built, and its output, exit status and trap line equal `ail run`
  built-differs    built, but one of them differs (stdout, exit or trap line)
  not-subset       native.subset.1 problems; the constructs are counted
  check-rejected   `ail build` stopped on front-end errors (negative tests included)
  backend-error    the backend reported an internal error (native.build.2)
  build-error      any other build failure (write, target, timeout)

Writes docs/design/native-coverage.md and tools/native-coverage/results.json, and
prints the delta against the previous results.json when one exists.
"""
import argparse
import collections
import concurrent.futures
import datetime
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
AIL = os.path.join(ROOT, "ail")
RESULTS = os.path.join(ROOT, "tools", "native-coverage", "results.json")
REPORT = os.path.join(ROOT, "docs", "design", "native-coverage.md")
BUILD_TIMEOUT = 120
RUN_TIMEOUT = 60

SOURCES = [
    ("examples", os.path.join(ROOT, "examples", "*.ail")),
    ("cases", os.path.join(ROOT, "bootstrap", "tests", "cases", "*.ail")),
    ("perfbench", os.path.join(ROOT, "tools", "perfbench", "programs", "*.ail")),
]
SUBSET = re.compile(r"error\[native\.subset\.1\]: (.*)")
TRAP = re.compile(r"trap\[[^\]]*\]")
DIAG = re.compile(r"(error|warning)\[[^\]]+\]")
STATUSES = ["built-match", "built-differs", "not-subset", "check-rejected",
            "backend-error", "build-error"]

# The constructs outside the subset and their owners: the one table (ADR 0746), shared
# with `ail explain native.subset.1`. A wording found in a message names its owner.
SUBSET_TABLE = os.path.join(ROOT, "docs", "design", "native-subset-constructs.md")
FOLLOWS = re.compile(r"result type|its type|must end with its result|only locals, parameters and constants")


def subset_rows():
    rows = []
    with open(SUBSET_TABLE, encoding="utf-8") as f:
        for line in f:
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if line.startswith("|") and len(cells) == 3 and cells[0] not in ("Construct",) and not cells[0].startswith("---"):
                rows.append(cells)
    return rows


def blocker_owner(msg):
    text = msg.replace("`", "")
    for _construct, wording, owner in subset_rows():
        if wording in text:
            return owner
    return "unowned" if not FOLLOWS.search(msg) else None


def rel(path):
    return os.path.relpath(path, ROOT)


def run(cmd, timeout, cwd=ROOT):
    """Return (returncode or None on timeout, stdout, stderr)."""
    try:
        p = subprocess.run(cmd, cwd=cwd, stdin=subprocess.DEVNULL, capture_output=True,
                           text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return None, "", ""


def trap_line(err):
    for line in err.splitlines():
        if TRAP.search(line):
            return line
    return ""


def construct(msg):
    """Group a native.subset.1 message into a construct name."""
    m = re.search(r"not in the native subset yet: (.+)$", msg)
    key = m.group(1) if m else msg
    key = re.sub(r"`([^`]*)`", lambda g: g.group(1) if m else "`…`", key)
    key = re.sub(r"\d+", "N", key)
    return key.strip()


def classify(path, workdir):
    exe = os.path.join(workdir, "prog")
    rec = {"file": rel(path), "status": None, "problems": [], "first": "", "detail": ""}
    rc, _, err = run([AIL, "build", path, "-o", exe], BUILD_TIMEOUT)
    if rc is None:
        rec.update(status="build-error", detail="build timed out")
        return rec
    if rc != 0:
        problems = [m.group(1) for m in SUBSET.finditer(err)]
        if problems:
            rec.update(status="not-subset", problems=problems, first=problems[0])
        elif "internal" in err.lower() or TRAP.search(err):
            # A trap while the native compiler runs on the interpreter is a backend bug.
            rec.update(status="backend-error", detail=trap_line(err) or first_line(err))
        elif DIAG.search(err):
            rec.update(status="check-rejected", detail=first_line(err))
        else:
            rec.update(status="build-error", detail=first_line(err))
        return rec
    # `env.program()` is the executable's path, and programs find their repo by it
    # (`path.dir(env.program())/../ail`), so run a copy next to the source, as
    # `ail run` sees the source path; the tests directory doesn't change the result.
    beside = os.path.join(os.path.dirname(os.path.abspath(path)), ".nc-" + os.path.basename(workdir))
    try:
        shutil.copy2(exe, beside)
    except OSError:
        beside = exe
    try:
        rc_n, out_n, err_n = run([beside], RUN_TIMEOUT, cwd=workdir)
    finally:
        if beside != exe:
            try:
                os.remove(beside)
            except OSError:
                pass
    rc_i, out_i, err_i = run([AIL, "run", path], RUN_TIMEOUT)
    if rc_i is None:
        rec.update(status="built-match", detail="interpreter timed out; not compared")
        return rec
    diffs = []
    if rc_n is None:
        diffs.append("native timeout")
    else:
        if rc_n != rc_i:
            diffs.append("exit")
        if out_n != out_i:
            diffs.append(f"stdout (native `{clip(first_diff(out_n, out_i)[0])}`, "
                         f"interpreter `{clip(first_diff(out_n, out_i)[1])}`)")
        if trap_line(err_n) != trap_line(err_i):
            diffs.append("trap")
    rec["status"] = "built-differs" if diffs else "built-match"
    rec["detail"] = "; ".join(diffs)
    return rec


def first_diff(a, b):
    """The first pair of lines where two outputs differ."""
    la, lb = a.splitlines(), b.splitlines()
    for x, y in zip(la, lb):
        if x != y:
            return x, y
    if len(la) != len(lb):
        longer = la if len(la) > len(lb) else lb
        extra = longer[min(len(la), len(lb))] if longer else ""
        return (extra, "") if longer is la else ("", extra)
    return "", ""


def clip(text, width=80):
    return text if len(text) <= width else text[: width - 1] + "…"


def first_line(text):
    for line in text.splitlines():
        if line.strip():
            return line.strip()
    return ""


def collect(pattern, name, only):
    rows = []
    for path in sorted(glob.glob(pattern)):
        if only and only not in path:
            continue
        rows.append((name, path))
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--only", default="", help="only programs whose path contains this text")
    ap.add_argument("--no-write", action="store_true", help="print only; keep the files")
    args = ap.parse_args()

    jobs = []
    for name, pattern in SOURCES:
        jobs.extend(collect(pattern, name, args.only))
    if not jobs:
        sys.exit("native-coverage: no programs matched")

    tmproot = tempfile.mkdtemp(prefix="native-coverage-")
    results = []
    try:
        def work(job):
            name, path = job
            workdir = tempfile.mkdtemp(prefix="p-", dir=tmproot)
            try:
                rec = classify(path, workdir)
            finally:
                shutil.rmtree(workdir, ignore_errors=True)
            rec["source"] = name
            # The sibling file says whether a case is a negative test (ADR 0584).
            stem = os.path.splitext(path)[0]
            rec["negative"] = any(os.path.exists(stem + ext) for ext in (".err", ".rules"))
            return rec

        with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
            for i, rec in enumerate(pool.map(work, jobs), 1):
                results.append(rec)
                print(f"[{i}/{len(jobs)}] {rec['status']:14} {rec['file']}", file=sys.stderr)
    finally:
        shutil.rmtree(tmproot, ignore_errors=True)

    results.sort(key=lambda r: r["file"])
    previous = load_previous()
    text = render(results, previous)
    print(text)
    if not args.no_write and not args.only:
        with open(RESULTS, "w") as f:
            json.dump({"date": today(), "commit": commit(), "programs": results}, f, indent=1)
            f.write("\n")
        with open(REPORT, "w") as f:
            f.write(text + "\n")
    return 0


def load_previous():
    if not os.path.exists(RESULTS):
        return None
    with open(RESULTS) as f:
        data = json.load(f)
    return {r["file"]: r for r in data["programs"]}


def today():
    return datetime.date.today().isoformat()


def commit():
    p = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT,
                       capture_output=True, text=True)
    return p.stdout.strip() or "unknown"


def pct(n, d):
    return f"{100.0 * n / d:.1f}%" if d else "n/a"


def render(results, previous):
    out = []
    out.append("# Native coverage report")
    out.append("")
    out.append(f"Generated by `tools/native-coverage/run.sh` on {today()} at commit {commit()}. "
               "Measurement only; see [ADR 0583](../decisions/0583-native-coverage-tracker.md) "
               "and [ADR 0584](../decisions/0584-native-coverage-classification.md).")
    out.append("")
    out.append("## Summary by source")
    out.append("")
    out.append("| source | programs | built, matches `ail run` | built, differs | not in subset "
               "| check-rejected | backend error | build error | builds and matches (of checked) |")
    out.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    groups = collections.OrderedDict()
    for name in ["examples", "cases", "perfbench"]:
        groups[name] = [r for r in results if r["source"] == name]
    groups["all"] = results
    for name, rows in groups.items():
        c = collections.Counter(r["status"] for r in rows)
        checked = len(rows) - c["check-rejected"]
        out.append(f"| {name} | {len(rows)} | {c['built-match']} | {c['built-differs']} | "
                   f"{c['not-subset']} | {c['check-rejected']} | {c['backend-error']} | "
                   f"{c['build-error']} | {c['built-match']} ({pct(c['built-match'], checked)}) |")
    out.append("")
    out.append("`check-rejected` includes negative tests (a sibling `.err` or `.rules` file); "
               f"{sum(1 for r in results if r['negative'] and r['status'] == 'check-rejected')} "
               "of them are negative.")
    out.append("")

    problems = collections.Counter()
    for r in results:
        for p in r["problems"]:
            problems[construct(p)] += 1
    blocking = collections.Counter(construct(r["first"]) for r in results if r["status"] == "not-subset")
    out.append("## Missing constructs by count")
    out.append("")
    out.append("Every native.subset.1 problem counts once. `first` counts the programs whose first "
               "reported problem is that construct, which is the one to fix first.")
    out.append("")
    out.append("| construct | problems | first | example |")
    out.append("|---|---:|---:|---|")
    examples = {}
    for r in results:
        for p in r["problems"]:
            examples.setdefault(construct(p), p)
    for key, n in problems.most_common(25):
        out.append(f"| {key} | {n} | {blocking.get(key, 0)} | {examples[key]} |")
    out.append("")

    out.append("## What blocks each example")
    out.append("")
    out.append("The first reported problem whose owner is known from the lane briefs. "
               "`follows` means the problem is a type or a use of a construct another blocker "
               "already names, so it clears with that blocker.")
    out.append("")
    out.append("| example | blocker | owner |")
    out.append("|---|---|---|")
    for r in results:
        if r["source"] != "examples" or r["status"] != "not-subset":
            continue
        pick = None
        for p in r["problems"]:
            owner = blocker_owner(p)
            if owner and owner != "unowned":
                pick = (p, owner)
                break
        if pick is None:
            pick = (r["first"], blocker_owner(r["first"]) or "follows")
            if pick[1] is None:
                pick = (r["first"], "follows")
        out.append(f"| {os.path.basename(r['file'])} | {clip(pick[0], 90)} | {pick[1]} |")
    out.append("")

    out.append("## Built but differs")
    out.append("")
    differs = [r for r in results if r["status"] == "built-differs"]
    if differs:
        out.append("| program | differs in |")
        out.append("|---|---|")
        for r in differs:
            out.append(f"| {r['file']} | {r['detail']} |")
    else:
        out.append("None.")
    out.append("")

    out.append("## Other failures")
    out.append("")
    other = [r for r in results if r["status"] in ("backend-error", "build-error")]
    if other:
        for r in other:
            out.append(f"- {r['file']}: {r['status']}: {r['detail']}")
    else:
        out.append("None.")
    out.append("")

    out.append("## Change since the last run")
    out.append("")
    if previous is None:
        out.append("No earlier results.json; this run is the baseline.")
    else:
        out.append(delta(results, previous))
    return "\n".join(out)


def delta(results, previous):
    lines = []
    now = {r["file"]: r for r in results}
    old_built = sum(1 for r in previous.values() if r["status"] == "built-match")
    new_built = sum(1 for r in results if r["status"] == "built-match")
    lines.append(f"Built and matching: {old_built} to {new_built} ({new_built - old_built:+d}).")
    lines.append("")
    changed = []
    for f, r in now.items():
        old = previous.get(f)
        if old is None:
            changed.append(f"- {f}: new, {r['status']}")
        elif old["status"] != r["status"]:
            changed.append(f"- {f}: {old['status']} to {r['status']}")
    lines.extend(changed if changed else ["No program changed status."])
    return "\n".join(lines)


if __name__ == "__main__":
    sys.exit(main())
