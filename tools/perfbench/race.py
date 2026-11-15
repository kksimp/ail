#!/usr/bin/env python3
"""Drag-race harness (drag.*, ADR 0460). Measurement only (ADR 0070).

Times the same program in `ail run` (native, through the build cache), AIL native
(`ail build`, once lane 120 lands it), C (`clang -O2` and `gcc -O2`), Rust
(`rustc -C opt-level=3`, with `--rust`), Go (`go build`, with `--go`) and Python, then prints one table: min and median wall seconds per column and
ratios to C and to Python (below 1 beats it).

A program NAME.ail in tools/perfbench/programs/ (or the interpreter set in
run.sh) has twins NAME.c, NAME.rs, NAME.go and NAME.py beside it. The programs must print the
same bytes: a column whose output differs from NAME.out (or, without one, the AIL interpreter's) is
reported as DIFF and left out of the ratios (drag.out.1).
"""
import argparse, os, platform, shutil, statistics, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
# Windows: Python can't start the bash wrapper; ail.cmd hands on to ail.ps1. Executables need .exe there.
WINDOWS = os.name == "nt"
AIL = os.path.join(ROOT, "ail.cmd" if WINDOWS else "ail")
EXE = ".exe" if WINDOWS else ""
CORPUS = os.path.join(ROOT, "tools", "tokbench", "corpus", "programs")
DEFAULT_SET = [
    "examples/interp.ail", "examples/json.ail", "examples/sheet.ail",
    "tools/perfbench/programs/loops.ail", "examples/logscan.ail",
    "examples/tasks.ail", "examples/build.ail",
    "tools/perfbench/programs/parsum.ail", "tools/perfbench/programs/tensor.ail",
    "tools/perfbench/programs/tensor_loops.ail", "tools/perfbench/programs/tensor_ops.ail",
    "tools/perfbench/programs/tensor_ops_loops.ail", "tools/perfbench/programs/race_smoke.ail",
] + ["tools/perfbench/programs/%s.ail" % n for n in (
    "collatz", "nbody", "spectralnorm", "mandelbrot", "fannkuch", "binarytrees",
    "fasta", "wordfreq", "matmul", "sieve")]


def run_once(cmd, timeout, env=None):
    """Wall seconds and stdout of one run; (None, reason) when it fails. env: extra environment variables."""
    t0 = time.perf_counter()
    try:
        p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                           stdin=subprocess.DEVNULL, timeout=timeout,
                           env=dict(os.environ, **env) if env else None)
    except subprocess.TimeoutExpired:
        return None, "timeout"
    dt = time.perf_counter() - t0
    if p.returncode != 0:
        return None, "exit %d" % p.returncode
    return dt, p.stdout


def measure(cmd, runs, timeout, env=None):
    """-> dict(min, med, max, out) or dict(err=...). The published tables report the median and the min-max spread."""
    times, out = [], None
    for i in range(runs):
        dt, o = run_once(cmd, timeout, env)
        if dt is None:
            return {"err": o}
        times.append(dt)
        out = o
    return {"min": min(times), "med": statistics.median(times), "max": max(times), "out": out}


def twin(prog, ext):
    name = os.path.splitext(os.path.basename(prog))[0]
    for cand in (os.path.join(os.path.dirname(prog), name + ext),
                 os.path.join(CORPUS, name, name + ext)):
        if os.path.isfile(cand):
            return cand
    return None


def size_args(prog):
    """The size argument in NAME.args beside the program, passed to every language."""
    f = os.path.splitext(prog)[0] + ".args"
    return open(f).read().split() if os.path.isfile(f) else []


def expected_out(prog):
    f = os.path.splitext(prog)[0] + ".out"
    return open(f, "rb").read() if os.path.isfile(f) else None


# How the size argument grows when the C twin is too quick (drag.size.1): most programs double;
# these are factorial or exponential in the argument, so one step is enough.
GROW_BY_ONE = ("fannkuch", "binarytrees")


def calibrate(prog, tmp, secs, timeout):
    """Smallest size in NAME.args, doubled (or +1) until gcc -O2 runs for at least `secs` seconds.

    Returns (size args, gcc output at that size), or (None, None) without a gcc, C twin or size argument.
    Runs under 0.3 s sit inside the 10% noise band, so a ratio there says nothing (drag.size.1)."""
    args, src = size_args(prog), twin(prog, ".c")
    exe = build_c(src, tmp, "gcc") if src and shutil.which("gcc") else None
    if not exe or len(args) != 1 or not args[0].isdigit():
        return None, None
    n, name = int(args[0]), os.path.splitext(os.path.basename(prog))[0]
    for _ in range(40):
        dt, out = run_once([exe, str(n)], timeout)
        if dt is None:
            return None, None
        if dt >= secs:
            return [str(n)], out
        n = n + 1 if name in GROW_BY_ONE else n * 2
    return None, None


def tool_version(cmd):
    try:
        return subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                              text=True).stdout.splitlines()[0].strip()
    except Exception:
        return None


def cpu_name():
    try:
        for line in open("/proc/cpuinfo"):
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    if platform.system() == "Darwin":
        try:
            return subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"],
                                  stdout=subprocess.PIPE, text=True).stdout.strip()
        except Exception:
            pass
    return platform.processor() or platform.machine()


def build_native(prog, tmp, build_args):
    """`ail build` the program; returns the executable path or None (drag.native.1)."""
    out = os.path.join(tmp, "native-" + os.path.basename(prog)[:-4] + EXE)
    cmd = [AIL, "build"] + build_args.split() + [prog, "-o", out]
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if p.returncode != 0 or not os.path.isfile(out):
        return None
    return out


def build_c(src, tmp, cc):
    out = os.path.join(tmp, "%s-%s" % (cc, os.path.basename(src)[:-2]) + EXE)
    # -ffp-contract=off: no fused multiply-adds, so floats match AIL and Python. Windows has no separate libm.
    p = subprocess.run([cc, "-O2", "-ffp-contract=off", "-o", out, src] + ([] if WINDOWS else ["-lm"]),
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return out if p.returncode == 0 else None


def build_rust(src, tmp):
    """`rustc -C opt-level=3` (safe, idiomatic Rust; no target-cpu flag, like the C columns)."""
    out = os.path.join(tmp, "rustc-" + os.path.basename(src)[:-3] + EXE)
    p = subprocess.run(["rustc", "-C", "opt-level=3", "--edition", "2021", "-o", out, src],
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return out if p.returncode == 0 else None


def build_go(src, tmp):
    """`go build` with its default flags (the gc compiler's optimisations are always on)."""
    out = os.path.join(tmp, "go-" + os.path.basename(src)[:-3] + EXE)
    p = subprocess.run(["go", "build", "-o", out, src], stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=tmp)
    return out if p.returncode == 0 else None


# What each column ran with, recorded in the JSON so every host's page says it (tools/bench/publish.py).
FLAGS = {"native": "ail build FILE -o EXE (AIL_THREADS unset: auto-parallel loops use every core)",
         "native1t": "the same executable with AIL_THREADS=1",
         "gcc": "gcc -O2 -ffp-contract=off -lm", "clang": "clang -O2 -ffp-contract=off -lm",
         "rustc": "rustc -C opt-level=3 --edition 2021", "go": "go build (default flags)",
         "python": "python3 -I (CPython, no numpy)"}


def fmt(x):
    return "-" if x is None else "%.3f" % x


def ratio(a, b):
    return "-" if a is None or b is None or b == 0 else "%.2f" % (a / b)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("runs_pos", nargs="?", type=int, help="runs per column (legacy position)")
    ap.add_argument("--runs", type=int, default=None)
    ap.add_argument("--native", action="store_true", help="also time `ail build` output")
    ap.add_argument("--native-1t", action="store_true",
                    help="also time the `ail build` output with AIL_THREADS=1 (column native1t), so a row that "
                         "ran on several cores can be read against the single-threaded twins")
    ap.add_argument("--c", action="store_true", help="also time C twins (clang and gcc -O2)")
    ap.add_argument("--rust", action="store_true", help="also time Rust twins (rustc -C opt-level=3)")
    ap.add_argument("--go", action="store_true", help="also time Go twins (go build)")
    ap.add_argument("--python", action="store_true", help="also time Python twins")
    ap.add_argument("--all", action="store_true", help="--native --c --python")
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--build-args", default=os.environ.get("AIL_BUILD_ARGS", ""))
    ap.add_argument("--no-run", action="store_true", help="skip the `ail run` column (programs too big for it)")
    ap.add_argument("--json", metavar="FILE", help="also write the raw numbers as JSON (tools/bench/publish.sh)")
    ap.add_argument("--md", metavar="FILE", help="also write the table and environment as Markdown")
    ap.add_argument("--calibrate", type=float, metavar="SECS", default=0,
                    help="raise each program's size until gcc -O2 takes at least SECS (drag.size.1); "
                         "the gcc output at that size is the reference, NAME.out is not used")
    ap.add_argument("--programs", nargs="*", help="programs to race (default: the interpreter set)")
    a = ap.parse_args()
    runs = a.runs or a.runs_pos or 5
    if a.all:
        a.native = a.c = a.python = True
    if not (a.native or a.c or a.python):
        a.python = True  # the pre-ADR-0460 default: interpreter vs Python
    progs = [p if os.path.isabs(p) else os.path.join(ROOT, p) for p in (a.programs or DEFAULT_SET)]
    tmp = tempfile.mkdtemp(prefix="perfbench-")

    cols = [] if a.no_run else ["ail"]
    if a.native: cols.append("native")
    if a.native and a.native_1t: cols.append("native1t")
    cols_c = [cc for cc in ("clang", "gcc") if a.c and shutil.which(cc)]
    if a.c and not cols_c:
        print("perfbench: --c needs clang or gcc; neither is installed", file=sys.stderr)
    cols += cols_c
    # "gcc-alloc": the NAME_alloc.c twin, which allocates the way the AIL program does.
    alloc = a.c and shutil.which("gcc") is not None
    if alloc:
        cols.append("gcc-alloc")
    rust = a.rust and shutil.which("rustc") is not None
    if a.rust and not rust:
        print("perfbench: --rust needs rustc; it is not installed", file=sys.stderr)
    if rust:
        cols.append("rustc")
    gocol = a.go and shutil.which("go") is not None
    if a.go and not gocol:
        print("perfbench: --go needs go; it is not installed", file=sys.stderr)
    if gocol:
        cols.append("go")
    py = sys.executable if a.python else None
    if py: cols.append("python")

    native_ok = None
    if a.native:
        probe = build_native(os.path.join(ROOT, "examples", "hello.ail"), tmp, a.build_args)
        native_ok = probe is not None
        if not native_ok:
            print("perfbench: --native skipped: `ail build` isn't available yet (lane 120)", file=sys.stderr)

    rows = []
    try:
        for p in progs:
            name = os.path.splitext(os.path.basename(p))[0]
            extra, cal_ref = size_args(p), None
            if a.calibrate:
                got, cal_ref = calibrate(p, tmp, a.calibrate, a.timeout)
                if got:
                    extra = got
                else:
                    sys.stderr.write("  %s: not calibrated (no gcc, C twin or size); using NAME.args\n" % name)
            res = {} if a.no_run else {"ail": measure([AIL, "run", p] + extra, runs, a.timeout)}
            ref = cal_ref if cal_ref is not None else expected_out(p)
            if ref is None and "ail" in res:
                ref = res["ail"].get("out")
            if a.native and native_ok:
                exe = build_native(p, tmp, a.build_args)
                res["native"] = measure([exe] + extra, runs, a.timeout) if exe else {"err": "build"}
                if a.native_1t:
                    res["native1t"] = measure([exe] + extra, runs, a.timeout, {"AIL_THREADS": "1"}) if exe else {"err": "build"}
            for cc in cols_c:
                src = twin(p, ".c")
                exe = build_c(src, tmp, cc) if src else None
                res[cc] = measure([exe] + extra, runs, a.timeout) if exe else {"err": "no twin" if not src else "build"}
            if alloc:
                src = twin(p, "_alloc.c")
                exe = build_c(src, tmp, "gcc") if src else None
                res["gcc-alloc"] = measure([exe] + extra, runs, a.timeout) if exe else {"err": "no twin" if not src else "build"}
            if rust:
                src = twin(p, ".rs")
                exe = build_rust(src, tmp) if src else None
                res["rustc"] = measure([exe] + extra, runs, a.timeout) if exe else {"err": "no twin" if not src else "build"}
            if gocol:
                src = twin(p, ".go")
                exe = build_go(src, tmp) if src else None
                res["go"] = measure([exe] + extra, runs, a.timeout) if exe else {"err": "no twin" if not src else "build"}
            if py:
                src = twin(p, ".py")
                res["python"] = measure([py, "-I", src] + extra, runs, a.timeout) if src else {"err": "no twin"}
            if ref is None:  # no NAME.out and no interpreter run: the first column that ran sets the reference
                ref = next((r["out"] for r in res.values() if "out" in r), None)
            for c, r in res.items():
                if "out" in r and ref is not None and r["out"].replace(b"\r\n", b"\n") != ref.replace(b"\r\n", b"\n"):
                    res[c] = {"err": "DIFF"}  # drag.out.1
            rows.append((name, res, " ".join(extra)))
            sys.stderr.write("  %s done\n" % name)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    head = ["program"]
    for c in cols:
        head += [c + " min", c + " med"]
    ratio_cols = []
    for ref in ("clang", "gcc", "gcc-alloc", "rustc", "go", "python"):
        if ref in cols:
            for c in cols:
                if c != ref and c in ("ail", "native", "native1t"):
                    ratio_cols.append((c, ref))
    head += ["%s/%s" % rc for rc in ratio_cols]
    table = [head]
    for name, res, _size in rows:
        line = [name]
        for c in cols:
            r = res.get(c, {})
            line += [fmt(r.get("min")), fmt(r.get("med"))] if "min" in r else [r.get("err", "-")] * 2
        for c, ref in ratio_cols:
            line.append(ratio(res.get(c, {}).get("min"), res.get(ref, {}).get("min")))
        table.append(line)

    widths = [max(len(r[i]) for r in table) for i in range(len(head))]
    for r in table:
        print("  ".join(v.ljust(widths[0]) if i == 0 else v.rjust(widths[i]) for i, v in enumerate(r)))

    if os.path.isfile(os.path.join(HERE, "programs", "parsum.ail")) and "parsum" in [r[0] for r in rows]:
        def pool(env):
            e = dict(os.environ, **env)
            ts = []
            for _ in range(runs):
                t0 = time.perf_counter()
                subprocess.run([AIL, "run", os.path.join(HERE, "programs", "parsum.ail")],
                               stdout=subprocess.DEVNULL, env=e)
                ts.append(time.perf_counter() - t0)
            return min(ts)
        print("parsum min_s: 1 thread %.3f, pool %.3f" % (pool({"AIL_THREADS": "1"}), pool({})))

    if a.json:
        import json
        with open(a.json, "w") as f:
            json.dump({"runs": runs, "columns": cols, "flags": FLAGS, "rows": [
                {"program": n, "size": size, "cells": {c: {k: v for k, v in r.items() if k != "out"} for c, r in res.items()}}
                for n, res, size in rows]}, f, indent=1)
    if a.md:
        write_md(a.md, table, cols, runs, cols_c, py)


def write_md(path, table, cols, runs, cols_c, py):
    ail_rev = subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"],
                             stdout=subprocess.PIPE, text=True).stdout.strip()
    env = [
        ("date", time.strftime("%Y-%m-%d", time.gmtime())),
        ("commit", ail_rev),
        ("os", "%s %s" % (platform.system(), platform.machine())),
        ("cpu", "%s, %d logical cores" % (cpu_name(), os.cpu_count() or 0)),
        ("runs per cell", str(runs)),
    ]
    for cc in cols_c:
        env.append((cc, tool_version([cc, "--version"]) or "?"))
    if py:
        env.append(("python", platform.python_version()))
    lines = ["# Drag race: AIL against C and Python", "",
             "Generated by `tools/perfbench/run.sh` (ADR 0460). Seconds of wall time; `min` is the",
             "best of the runs, `med` the median; `X/Y` is X's min over Y's min, so below 1 means X is faster.",
             "A cell that reads `DIFF` printed different bytes from the AIL interpreter and is excluded.", "",
             "## Environment", ""]
    lines += ["- %s: %s" % kv for kv in env]
    lines += ["", "## Results", "", "| " + " | ".join(table[0]) + " |",
              "|" + "|".join(["---"] + ["---:"] * (len(table[0]) - 1)) + "|"]
    for r in table[1:]:
        lines.append("| " + " | ".join(r) + " |")
    lines += ["", "## Notes", "",
              "- Each program gets the size in its `NAME.args` in every language; C is built with `-std=c11 -O2 -ffp-contract=off -lm`.",
              "- `native` is `ail build FILE -o OUT`; a program outside the native subset reads `build` (native.subset.1). The `-` in a ratio means one side has no number.",
              "- `ail` is the bytecode tier (`ail run`); process start and the check pass are inside its time, as start-up is for `python3`.",
              "- `gcc` and `clang` are the best-effort C twins (NAME.c). `gcc-alloc` is NAME_alloc.c, which allocates the way the AIL program does (fresh arrays per tree, per permutation and per line, growing by doubling); only binarytrees, fannkuch and fasta have one. matmul's C twin is one flat array and the AIL program is `List[List[int]]`: a layout difference, not an algorithm one.",
              "- Programs without a C twin read `no twin`. The integrator runs macOS ARM64 with the same command.", ""]
    with open(path, "w") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
