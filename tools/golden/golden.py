#!/usr/bin/env python3
"""Frozen goldens for the tools that were checked against the Rust binary (D0).

The parity scripts (tools/explain/parity.sh, tools/fmt/parity.sh, tools/fix/run-parity.py,
tools/native-cli/api-parity.sh and api-diff-parity.sh) run every case twice, the Rust `ail`
and the native one. Here the Rust side is run once (`freeze`) and its exit status and
output hashes are kept in tests/golden/TOOL.tsv; `check` runs the native `ail` alone and
compares. No Rust binary is needed to check.

  tools/golden/golden.py freeze TOOL... [--bin CMD] [--jobs N] [--limit N]
  tools/golden/golden.py check  TOOL... [--bin CMD] [--jobs N] [--limit N] [--strict]

TOOL is explain, fmt, fix, api or all. `--bin` is the `ail` to run (default `./ail`;
freeze it with `--bin "AIL_RUST=1 ./ail"` until the Rust binary is gone, afterwards with the
native one, to refresh a golden on purpose). A row is `case<TAB>input hash<TAB>exit
status<TAB>output hash`. The input hash is of the files the case reads, so a row whose
input has changed since the freeze is stale: it is skipped and counted, and it is a
failure only with `--strict` (refreeze to bring it up to date). Last line:
`GOLDEN TOOL GREEN (N checked, M stale)` or `GOLDEN TOOL RED (N failures)`.
"""
import hashlib, os, re, shutil, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

ROOT = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip()
GOLDEN = os.path.join(ROOT, "tests", "golden")
TOOLS = ["explain", "fmt", "fix", "api"]


def h(data):
    if isinstance(data, str):
        data = data.encode()
    return hashlib.sha256(data).hexdigest()[:16]


def file_hash(path):
    with open(os.path.join(ROOT, path), "rb") as f:
        return h(f.read())


def tree_hash(paths):
    return h("".join(file_hash(p) for p in paths))


def listing(*patterns):
    out = subprocess.run(["git", "ls-files", *patterns], cwd=ROOT, capture_output=True, text=True).stdout
    return [p for p in out.splitlines() if p]


def split_bin(text):
    """`AIL_RUST=1 ./ail` as (command words, environment additions)."""
    words, env = text.split(), {}
    while words and re.match(r"^[A-Z_]+=", words[0]):
        k, v = words.pop(0).split("=", 1)
        env[k] = v
    return words, env


BIN_ENV = {}


def run(binary, args, cwd=None, tmp=None):
    """(exit status, hash of stdout + stderr) with the scratch directory's name normalised."""
    env = dict(os.environ, AIL_ROOT=ROOT, **BIN_ENV)
    r = subprocess.run(binary + args, cwd=cwd or ROOT, capture_output=True, text=True, env=env)
    out = r.stdout + "\x00" + r.stderr
    if tmp:
        out = out.replace(tmp, "T")
    out = re.sub(r"/(?:tmp|var/folders)/[^\"/ ]*/", "TMP/", out)
    return r.returncode, h(out)


# Each case is (name, input hash, function binary -> (rc, output hash)).


def explain_cases(binary):
    spec = listing("docs/spec/*.md", "compiler/explain/*")
    ins = tree_hash(spec)
    rules = []
    for line in subprocess.run(binary + ["explain"], cwd=ROOT, capture_output=True, text=True).stdout.splitlines():
        parts = line.split()
        if len(parts) >= 2 and "." in parts[0]:
            rules.append(parts[0])
    names = [[], ["--json"]]
    for r in rules + ["bootstrap.unsupported", "no.such.rule", "expr.bind.99", "zzz.yy.1", "--foo"]:
        names += [[r], [r, "--json"]]
    return [("explain " + " ".join(a), ins, (lambda b, a=a: run(b, ["explain"] + a))) for a in names]


def lengthened(path, out):
    subprocess.run(["python3", os.path.join(ROOT, "tools/fmt/lengthen.py"), path, out], cwd=ROOT, check=True)


def fmt_cases(binary):
    cases = []
    for f in listing("*.ail"):
        ins = file_hash(f)
        cases.append((f"fmt plain {f}", ins, (lambda b, f=f: run(b, ["fmt", "--stdout", f]))))
        cases.append((f"fmt long {f}", ins, (lambda b, f=f: fmt_long(b, f))))
    return cases


def fmt_long(binary, f):
    d = tempfile.mkdtemp()
    try:
        p = os.path.join(d, "long.ail")
        lengthened(f, p)
        return run(binary, ["fmt", "--stdout", p], tmp=d)
    finally:
        shutil.rmtree(d, ignore_errors=True)


FIX_FLAGS = [[], ["--dry-run"], ["--dry-run", "--json"], ["--json"]]


def fix_one(binary, f, flags):
    d = tempfile.mkdtemp()
    try:
        p = os.path.join(d, "t.ail")
        shutil.copy(os.path.join(ROOT, f), p)
        rc, out = run(binary, ["fix", p] + flags, tmp=p)
        written = "" if "--dry-run" in flags else open(p).read()
        return rc, h(out + written)
    finally:
        shutil.rmtree(d, ignore_errors=True)


def fix_cases(binary):
    files = [f for f in listing("tests/mistakes/*.ail", "tests/fix/*.ail")]
    return [
        (f"fix {' '.join(fl)} {f}", file_hash(f), (lambda b, f=f, fl=fl: fix_one(b, f, fl)))
        for f in files
        for fl in FIX_FLAGS
    ]


API_FLAGS = [[], ["--text"], ["--pretty"], ["--symbols"], ["--tokens"], ["--ast"]]


def api_cases(binary):
    files = listing("examples/*.ail", "tests/cases/*.ail")
    cases = [
        (f"api {' '.join(fl)} {f}", file_hash(f), (lambda b, f=f, fl=fl: run(b, ["api"] + fl + [f])))
        for f in files
        for fl in API_FLAGS
    ]
    for old, new in zip(files, files[1:]):
        ins = h(file_hash(old) + file_hash(new))
        for fl in ([], ["--json"]):
            cases.append((f"api --diff {' '.join(fl)} {old} {new}", ins, (lambda b, o=old, n=new, fl=fl: run(b, ["api", "--diff", o, n] + fl))))
    return cases


BUILDERS = {"explain": explain_cases, "fmt": fmt_cases, "fix": fix_cases, "api": api_cases}


def load(tool):
    rows = {}
    path = os.path.join(GOLDEN, tool + ".tsv")
    if os.path.exists(path):
        for line in open(path).read().splitlines():
            name, ins, rc, out = line.split("\t")
            rows[name] = (ins, int(rc), out)
    return rows


def main():
    args = sys.argv[1:]
    if not args or args[0] not in ("freeze", "check"):
        print(__doc__)
        return 2
    mode = args.pop(0)
    binary, jobs, limit, strict, tools = ["./ail"], 3, 0, False, []
    while args:
        a = args.pop(0)
        if a == "--bin":
            binary, extra = split_bin(args.pop(0))
            BIN_ENV.update(extra)
        elif a == "--jobs":
            jobs = int(args.pop(0))
        elif a == "--limit":
            limit = int(args.pop(0))
        elif a == "--strict":
            strict = True
        else:
            tools += TOOLS if a == "all" else [a]
    if not tools:
        tools = TOOLS
    status = 0
    for tool in tools:
        if tool not in BUILDERS:
            print("unknown tool", tool)
            return 2
        cases = BUILDERS[tool](binary)
        if limit:
            cases = cases[:limit]
        if mode == "freeze":
            with ThreadPoolExecutor(jobs) as ex:
                results = list(ex.map(lambda c: c[2](binary), cases))
            os.makedirs(GOLDEN, exist_ok=True)
            with open(os.path.join(GOLDEN, tool + ".tsv"), "w") as f:
                for (name, ins, _), (rc, out) in zip(cases, results):
                    f.write(f"{name}\t{ins}\t{rc}\t{out}\n")
            print(f"froze {tool}: {len(cases)} cases")
            continue
        rows = load(tool)
        if not rows:
            print(f"GOLDEN {tool} RED (no golden: tools/golden/golden.py freeze {tool})")
            status = 1
            continue
        todo = [c for c in cases if c[0] in rows and rows[c[0]][0] == c[1]]
        stale = len(cases) - len(todo)
        with ThreadPoolExecutor(jobs) as ex:
            results = list(ex.map(lambda c: c[2](binary), todo))
        fails = 0
        for (name, _, _), (rc, out) in zip(todo, results):
            want = rows[name]
            if (rc, out) != (want[1], want[2]):
                fails += 1
                print(f"DIFF {name} (exit {rc}, golden {want[1]})")
        if strict:
            fails += stale
        if fails:
            print(f"GOLDEN {tool} RED ({fails} failures, {len(todo)} checked, {stale} stale)")
            status = 1
        else:
            print(f"GOLDEN {tool} GREEN ({len(todo)} checked, {stale} stale)")
    return status


sys.exit(main())
