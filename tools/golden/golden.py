#!/usr/bin/env python3
"""Frozen goldens for the tools that were checked against the Rust binary (D0).

The parity scripts (tools/explain/parity.sh, tools/fmt/parity.sh, tools/fix/run-parity.py,
tools/native-cli/api-parity.sh and api-diff-parity.sh) run every case twice, the Rust `ail`
and the native one. Here the Rust side is run once (`freeze`) and its exit status and
output hashes are kept in tests/golden/TOOL.tsv; `check` runs the native `ail` alone and
compares. No Rust binary is needed to check.

  tools/golden/golden.py freeze TOOL... [--bin CMD] [--jobs N] [--limit N]
  tools/golden/golden.py compare TOOL --ref CMD --bin CMD [--match TEXT] [--limit N]
  tools/golden/golden.py check  TOOL... [--bin CMD] [--jobs N] [--limit N] [--strict]

A case named in tests/golden/known.txt is a known divergence (the native side does something the
frozen Rust one can't) and is skipped. `check` also takes `--match TEXT`.
TOOL is explain, fmt, fix, api, edit or all. `compare` runs the matching cases with two binaries (`--ref`, `--bin`) and prints the text
of each difference, to see why a `check` failed.
`--bin` is the `ail` to run (default `./ail`); refresh a golden on purpose by freezing with it. A row is `case<TAB>input hash<TAB>exit
status<TAB>output hash`. The input hash is of the files the case reads, so a row whose
input has changed since the freeze is stale: it is skipped and counted, and it is a
failure only with `--strict` (refreeze to bring it up to date). Last line:
`GOLDEN TOOL GREEN (N checked, M stale)` or `GOLDEN TOOL RED (N failures)`.
"""
import difflib, hashlib, os, re, shutil, subprocess, sys, tempfile, threading
from concurrent.futures import ThreadPoolExecutor

# ADR 1130: the `ail check` runs here must not apply their certain fixes to the inputs.
os.environ.setdefault("AIL_FIX", "0")

ROOT = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip()
GOLDEN = os.path.join(ROOT, "tests", "golden")
TOOLS = ["explain", "fmt", "fix", "api", "edit"]


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
    # Cases run in scratch directories, so a relative program path (`./ail`) must not follow them.
    if words and "/" in words[0]:
        words[0] = os.path.abspath(words[0])
    return words, env


BIN_ENV = {}
LAST = {}


def run(binary, args, cwd=None, tmp=None, stdin=None, stderr=True):
    """(exit status, hash of stdout + stderr) with the scratch directory's name normalised."""
    env = dict(os.environ, AIL_ROOT=ROOT, **BIN_ENV)
    r = subprocess.run(binary + args, cwd=cwd or ROOT, capture_output=True, text=True, env=env, input=stdin)
    out = r.stdout + "\x00" + (r.stderr if stderr else "")
    if tmp:
        out = out.replace(tmp, "T")
    out = re.sub(r"/(?:tmp|var/folders)/[^\"/ ]*/", "TMP/", out)
    # The native `api --ast` ends with `"asks":[]` (the `uhwhatnow?` positions, ADR 0872), which the Rust
    # tree has no field for; an empty list says nothing, so it is dropped before comparing.
    out = out.replace(',"asks":[]', "")
    # The Rust diagnostics for generated code print the line with the second file's base added
    # (`f.ail:1048580:44`, `T:1048580:44` once the scratch path is normalised); the native ones print the line (`4:44`), which is right.
    out = re.sub(r":(\d{7,10}):(\d+)", lambda m: ":%d:%s" % (int(m.group(1)) - 1048576, m.group(2)) if 1048576 <= int(m.group(1)) < 1073741824 else m.group(0), out)
    out = re.sub(r'"line":(\d{7,10})', lambda m: '"line":%d' % (int(m.group(1)) - 1048576) if 1048576 <= int(m.group(1)) < 1073741824 else m.group(0), out)
    # Likewise the parameter flag `"observe":false` (`@observe`, ADR 0879).
    out = out.replace(',"observe":false', "").replace('"observe":false,', "")
    LAST[threading.get_ident()] = out
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
        cases.append((f"fmt plain {f}", ins, (lambda b, f=f: run(b, ["fmt", "--stdout", f], stderr=False))))
        cases.append((f"fmt long {f}", ins, (lambda b, f=f: fmt_long(b, f))))
    return cases


def fmt_long(binary, f):
    d = tempfile.mkdtemp()
    try:
        p = os.path.join(d, "long.ail")
        lengthened(f, p)
        return run(binary, ["fmt", "--stdout", p], tmp=d, stderr=False)
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
        (f"api {' '.join(fl)} {f}", file_hash(f), (lambda b, f=f, fl=fl: run(b, ["api"] + fl + [f], stderr=False)))
        for f in files
        for fl in API_FLAGS
    ]
    for old, new in zip(files, files[1:]):
        ins = h(file_hash(old) + file_hash(new))
        for fl in ([], ["--json"]):
            cases.append((f"api --diff {' '.join(fl)} {old} {new}", ins, (lambda b, o=old, n=new, fl=fl: run(b, ["api", "--diff", o, n] + fl))))
    return cases


EDIT_FLAGS = [["--dry-run"], ["--dry-run", "--json"], ["--dry-run", "--diff"]]


def edit_one(binary, pkg, name, flags):
    cwd = os.path.join(ROOT, pkg)
    with open(os.path.join(cwd, name)) as f:
        return run(binary, ["edit"] + flags + ["main.ail"], cwd=cwd, stdin=f.read())


def edit_cases(binary):
    """Each edit input of a tests/cases/packages/edit_* directory, dry-run against its main.ail."""
    cases = []
    for pkg in sorted({os.path.dirname(f) for f in listing("tests/cases/packages/edit_*/main.ail")}):
        names = [os.path.basename(f) for f in listing(pkg + "/*.json", pkg + "/*.txt") if os.path.basename(f) != "expected.out"]
        ins = tree_hash(listing(pkg + "/*"))
        for n in names:
            for fl in EDIT_FLAGS:
                cases.append((f"edit {' '.join(fl)} {pkg} {n}", ins, (lambda b, pkg=pkg, n=n, fl=fl: edit_one(b, pkg, n, fl))))
    return cases


BUILDERS = {"explain": explain_cases, "fmt": fmt_cases, "fix": fix_cases, "api": api_cases, "edit": edit_cases}


def load(tool):
    rows = {}
    path = os.path.join(GOLDEN, tool + ".tsv")
    if os.path.exists(path):
        for line in open(path).read().splitlines():
            name, ins, rc, out = line.split("\t")
            rows[name] = (ins, int(rc), out)
    return rows


def known_cases():
    """tests/golden/known.txt: a case whose name has one of these lines is a known divergence, not checked."""
    path = os.path.join(GOLDEN, "known.txt")
    if not os.path.exists(path):
        return []
    return [l.strip() for l in open(path).read().splitlines() if l.strip() and not l.startswith("#")]


def main():
    args = sys.argv[1:]
    if not args or args[0] not in ("freeze", "check", "compare"):
        print(__doc__)
        return 2
    mode = args.pop(0)
    binary, jobs, limit, strict, tools = [os.path.abspath("./ail")], 3, 0, False, []
    ref, match = None, ""
    while args:
        a = args.pop(0)
        if a == "--bin":
            binary, extra = split_bin(args.pop(0))
            BIN_ENV.update(extra)
        elif a == "--jobs":
            jobs = int(args.pop(0))
        elif a == "--limit":
            limit = int(args.pop(0))
        elif a == "--ref":
            ref = split_bin(args.pop(0))
        elif a == "--match":
            match = args.pop(0)
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
        if mode == "compare":
            if ref is None:
                print("compare needs --ref")
                return 2
            shown = 0
            bin_env = dict(BIN_ENV)
            for name, _, fn in cases:
                if match not in name:
                    continue
                texts = []
                for b, extra in ((ref[0], ref[1]), (binary, bin_env)):
                    saved = dict(BIN_ENV)
                    BIN_ENV.clear()
                    BIN_ENV.update(extra)
                    rc, _ = fn(b)
                    texts.append((rc, LAST[threading.get_ident()]))
                    BIN_ENV.clear()
                    BIN_ENV.update(saved)
                if texts[0] != texts[1]:
                    shown += 1
                    print(f"=== {name}: exit ref {texts[0][0]}, bin {texts[1][0]}")
                    a = texts[0][1].replace("\x00", "\n--stderr--\n").splitlines()
                    b = texts[1][1].replace("\x00", "\n--stderr--\n").splitlines()
                    for line in list(difflib.unified_diff(a, b, "ref", "bin", lineterm="", n=1))[:40]:
                        print(line)
                    if limit and shown >= limit:
                        break
            print(f"{shown} differing")
            continue
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
        cases = [c for c in cases if match in c[0]]
        known = [c for c in cases if any(k in c[0] for k in known_cases())]
        cases = [c for c in cases if c not in known]
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
            print(f"GOLDEN {tool} GREEN ({len(todo)} checked, {stale} stale, {len(known)} known divergences)")
    return status


sys.exit(main())
