#!/usr/bin/env python3
"""Edit-loop latency: how long an agent waits on each round (measurement only, ADR 0070).

    latency.py --out FILE.json [--runs N] [--ail COMMAND] [--fresh-clone] [--label HOST]

Wall milliseconds, best of --runs (default 5) and the median, for `ail check`, `ail test`,
`ail context`, `ail build` (cold: empty build cache; cached) and `ail run` (cached) on three
programs, from the repository root. Then `ail_check` through a warm `ail mcp` session (the first
call, then the best of the repeats of the same call). With --fresh-clone, the first `./ail run` in
a new clone of HEAD (the seed is fetched there only if this machine does not already hold it).
Standard library only; works on macOS, Linux and, with --ail ail.ps1 wrapped by the caller, Windows.
tools/bench/publish.sh calls this; publish.py renders the JSON.
"""
import argparse, json, os, platform, shlex, shutil, statistics, subprocess, sys, tempfile, threading, time

PROGRAMS = [("hello", "examples/hello.ail"), ("json", "examples/json.ail"), ("ailtest", "tools/ailtest/ailtest.ail")]
# (row key, programs it applies to). `run` is only timed where the program runs to completion with no arguments.
ROWS = [("check", None), ("context", None), ("test", None), ("build cold", None), ("build cached", None),
        ("run cached", ("hello", "json"))]


def lines(path):
    with open(path, "rb") as f:
        return sum(1 for _ in f)


def timed(argv, env, timeout, cwd=None):
    t0 = time.perf_counter()
    try:
        p = subprocess.run(argv, env=env, cwd=cwd, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL, timeout=timeout)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        rc = "timeout"
    return (time.perf_counter() - t0) * 1000.0, rc


def cell(argv_fn, runs, timeout, cold_cache=False):
    """Best and median of `runs` runs; ok only when every run exits 0."""
    ms, rcs = [], []
    for _ in range(runs):
        env = dict(os.environ)
        tmp = None
        if cold_cache:
            tmp = tempfile.mkdtemp(prefix="ail-lat-")
            env["AIL_CACHE_DIR"] = tmp
        t, rc = timed(argv_fn(tmp), env, timeout)
        if tmp:
            shutil.rmtree(tmp, ignore_errors=True)
        ms.append(t)
        rcs.append(rc)
    bad = [r for r in rcs if r != 0]
    return {"min": round(min(ms), 1), "median": round(statistics.median(ms), 1), "ok": not bad,
            "exit": bad[0] if bad else 0}


def mcp_session(ail, path, calls, timeout):
    """First `ail_check` call, then the repeats of the same call, through one `ail mcp` process."""
    d, base = os.path.dirname(os.path.abspath(path)), os.path.basename(path)
    p = subprocess.Popen(ail + ["mcp"], cwd=d, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.DEVNULL, text=True)
    killer = threading.Timer(timeout, p.kill)
    killer.start()
    try:
        def call(msg):
            t0 = time.perf_counter()
            p.stdin.write(json.dumps(msg) + "\n")
            p.stdin.flush()
            line = p.stdout.readline()
            return (time.perf_counter() - t0) * 1000.0, line
        call({"jsonrpc": "2.0", "id": 0, "method": "ping"})  # server start-up is not a call
        times, clean = [], True
        for i in range(1, calls + 1):
            t, line = call({"jsonrpc": "2.0", "id": i, "method": "tools/call",
                            "params": {"name": "ail_check", "arguments": {"program": base}}})
            if not line or '"isError":false' not in line.replace(" ", ""):
                clean = False
            times.append(t)
        return {"first": round(times[0], 1), "repeat": round(min(times[1:]), 1) if len(times) > 1 else None, "ok": clean}
    except Exception as e:  # a dead server, a closed pipe
        return {"ok": False, "err": str(e)}
    finally:
        killer.cancel()
        try:
            p.stdin.close()
        except Exception:
            pass
        p.kill()
        p.wait()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--ail", default="ail.cmd" if os.name == "nt" else "./ail")  # Python can't start the bash wrapper on Windows
    ap.add_argument("--label", default="")
    ap.add_argument("--timeout", type=int, default=900)
    ap.add_argument("--fresh-clone", action="store_true")
    a = ap.parse_args()
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    os.chdir(root)
    ail = shlex.split(a.ail, posix=(os.name != "nt"))  # a command, e.g. "powershell -NoProfile -File ail.ps1" on Windows
    if os.sep in ail[0] or "/" in ail[0] or os.path.isfile(ail[0]):
        ail[0] = os.path.abspath(ail[0])  # the mcp session runs in the program's directory: a relative path would not resolve there
    subprocess.run(ail + ["run", "examples/hello.ail"], stdout=subprocess.DEVNULL, check=True)  # builds the native CLI
    out = {"host": a.label or "%s %s" % (platform.system(), platform.machine()), "runs": a.runs,
           "commit": subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip(),
           "programs": {k: lines(p) for k, p in PROGRAMS}, "rows": [], "mcp": {}}
    for key, only in ROWS:
        for name, path in PROGRAMS:
            if only and name not in only:
                continue
            sys.stderr.write("latency: %s %s\n" % (key, name))
            if key == "check":
                fn = lambda t, p=path: ail + ["check", p]
            elif key == "context":
                fn = lambda t, p=path: ail + ["context", p, "main"]
            elif key == "test":
                fn = lambda t, p=path: ail + ["test", p]
            elif key == "run cached":
                timed(ail + ["run", path], None, a.timeout)  # warm the build cache
                fn = lambda t, p=path: ail + ["run", p]
            else:
                exe = os.path.join(tempfile.gettempdir(), "ail-lat-%s-%d%s" % (name, os.getpid(), ".exe" if os.name == "nt" else ""))
                if key == "build cached":
                    timed(ail + ["build", path, "-o", exe], None, a.timeout)
                fn = lambda t, p=path, e=exe: ail + ["build", p, "-o", e]
            c = cell(fn, a.runs, a.timeout, cold_cache=(key == "build cold"))
            c.update({"cmd": key, "program": name})
            out["rows"].append(c)
    for name, path in PROGRAMS:
        sys.stderr.write("latency: mcp %s\n" % name)
        out["mcp"][name] = mcp_session(ail, path, a.runs + 1, a.timeout)
    if a.fresh_clone:
        sys.stderr.write("latency: fresh clone\n")
        tmp = tempfile.mkdtemp(prefix="ail-clone-")
        try:
            subprocess.run(["git", "clone", "-q", root, os.path.join(tmp, "c")], check=True)
            clone = os.path.join(tmp, "c")
            start = [os.path.join(clone, "ail.cmd")] if os.name == "nt" else ["./ail"]
            t, rc = timed(start + ["run", "examples/hello.ail"], None, a.timeout, cwd=clone)
            out["fresh_clone"] = {"ms": round(t), "ok": rc == 0}
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    with open(a.out, "w") as f:
        json.dump(out, f, indent=1)
    bad = [r for r in out["rows"] if not r["ok"]] + [m for m in out["mcp"].values() if not m["ok"]]
    if bad:
        sys.stderr.write("latency: %d cell(s) did not exit 0; see %s\n" % (len(bad), a.out))


if __name__ == "__main__":
    main()
