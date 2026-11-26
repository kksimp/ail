#!/usr/bin/env python3
"""The public corpus: checks every file, imports program sets, exports the JSONL forms (ADR 1174).

  python3 tools/corpus/corpus.py check [--twins] [--why-complete] [--deny FILE]... [NAME...]
  python3 tools/corpus/corpus.py out NAME...
  python3 tools/corpus/corpus.py import SRC_DIR SET [--deny FILE]...
  python3 tools/corpus/corpus.py export OUT_DIR
  python3 tools/corpus/corpus.py self-test

`check` reads files only; it runs no AIL program, so it is the same on every host (the gate suites `corpus` and
`corpus_sets` build and run the programs). `--twins` also runs every Python twin and compares it with NAME.out.
`out` writes NAME.out from the Python twin of each named program (the twin is the oracle).
`--root DIR` (before the command) names the repository root; the corpus is DIR/corpus and the mistakes DIR/tests/mistakes.
The format is corpus/README.md.
"""
import hashlib, json, os, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
NAME_RE = re.compile(r"[a-z][a-z0-9_]{1,39}\Z")
RESERVED = {"sets", "mistakes"}  # directories of corpus/ that are not programs
MIN_LINES, MAX_LINES = 10, 300
MAX_OUT = 65536
MAX_SET = 500
MAX_WHY = 200
# Modules a corpus program may not use: they need a screen, a device, a network or a host process.
BANNED_USES = {
    "std.app", "std.attach", "std.audio", "std.gfx", "std.mcp", "std.media", "std.net", "std.screen", "std.term",
    "std.ui", "std.video",
}
LICENCE = "Apache-2.0 (the LICENSE file of kksimp/ail)"


class Program:
    def __init__(self, set_name, name, path):
        self.set, self.name, self.dir = set_name, name, path

    def file(self, ext):
        return os.path.join(self.dir, self.name + ext)


def read(path):
    # A Windows checkout may hold CRLF (autocrlf): read LF, as the gate compares LF.
    with open(path, encoding="utf-8", newline="") as f:
        return f.read().replace("\r\n", "\n")


def subdirs(path):
    if not os.path.isdir(path):
        return []
    return sorted(n for n in os.listdir(path) if os.path.isdir(os.path.join(path, n)))


def programs(root):
    """Every program directory: the first set, `core`, is corpus/NAME; the others are corpus/sets/SET/NAME."""
    c = os.path.join(root, "corpus")
    out = [Program("core", n, os.path.join(c, n)) for n in subdirs(c) if n not in RESERVED]
    for s in subdirs(os.path.join(c, "sets")):
        out += [Program(s, n, os.path.join(c, "sets", s, n)) for n in subdirs(os.path.join(c, "sets", s))]
    return out


def fingerprint(text):
    """A program's text with comments, blank lines and spacing removed: two programs that differ only there are one."""
    kept = []
    for line in text.splitlines():
        line = line.split("#", 1)[0] if '"' not in line else line
        line = "".join(line.split())
        if line:
            kept.append(line)
    return hashlib.sha256("\n".join(kept).encode()).hexdigest()


def task_text(ail):
    """The task a program states: its leading `##` lines, joined."""
    out = []
    for line in ail.splitlines():
        if not line.startswith("##"):
            break
        out.append(line[2:].strip())
    return " ".join(x for x in out if x)


def denied(root, extra_files):
    """Names and fingerprints no corpus program may have: the agent benchmark's tasks, and the lines of each --deny
    file (the sealed task names live only on the designer's Mac, so the Mac job passes that file)."""
    names, prints = set(), set()
    bench = os.path.join(root, "tools", "agentbench")
    for n in subdirs(os.path.join(bench, "tasks")):
        names.add(n.replace("-", "_"))
        names.add(n.split("-")[0])
    for d, _, files in os.walk(bench):
        for f in files:
            if f.endswith(".ail"):
                prints.add(fingerprint(read(os.path.join(d, f))))
    for path in extra_files:
        for line in read(path).splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                names.add(line.split()[0].replace("-", "_"))
    return names, prints


def check_program(p, problems):
    def red(msg):
        problems.append(f"{p.set}/{p.name}: {msg}")

    if not NAME_RE.match(p.name):
        red("the name must be lower snake case, 2 to 40 characters")
    want = {p.name + e for e in (".ail", ".py", ".out")}
    have = set(os.listdir(p.dir))
    for f in sorted(want - have):
        red(f"{f} is missing")
    for f in sorted(have - want):
        red(f"{f} does not belong here (a program is NAME.ail, NAME.py and NAME.out)")
    if want - have:
        return None
    ail, out = read(p.file(".ail")), read(p.file(".out"))
    for ext, text in ((".ail", ail), (".py", read(p.file(".py"))), (".out", out)):
        if "\r" in text:
            red(f"{p.name}{ext} has CR line ends")
        if text and not text.endswith("\n"):
            red(f"{p.name}{ext} does not end with a newline")
    if "\t" in ail:
        red("the AIL file has a tab")
    if not ail.startswith("## ") or len(task_text(ail)) < 20:
        red("the AIL file must open with `## ` lines that state the task")
    if not re.search(r"^fn main\(", ail, re.M):
        red("no `fn main`")
    if not re.search(r"^@test fn ", ail, re.M):
        red("no `@test`")
    n = ail.count("\n")
    if n < MIN_LINES or n > MAX_LINES:
        red(f"{n} lines; a program has {MIN_LINES} to {MAX_LINES}")
    for m in re.findall(r"^use (\S+)", ail, re.M):
        if m in BANNED_USES:
            red(f"uses {m}; a corpus program runs with no screen, device or network")
    if not out:
        red("the expected output is empty")
    if len(out.encode()) > MAX_OUT:
        red(f"the expected output is over {MAX_OUT} bytes")
    return ail


def run_twin(p):
    """A problem with the Python twin, or None: it must exit 0 and print NAME.out."""
    try:
        r = subprocess.run(
            [sys.executable, "-I", p.file(".py")], capture_output=True, text=True, timeout=60, cwd=p.dir
        )
    except subprocess.TimeoutExpired:
        return "the Python twin ran over 60 seconds"
    if r.returncode != 0:
        return f"the Python twin exits {r.returncode}: {r.stderr.strip()[-200:]}"
    if r.stdout != read(p.file(".out")):
        return "the Python twin's output differs from the .out"
    return None


def mistakes(root):
    """(name, rules, wrong, right) for every pair of tests/mistakes: NAME.ail with a NAME.fixed."""
    d = os.path.join(root, "tests", "mistakes")
    out = []
    for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
        if f.endswith(".fixed"):
            n = f[: -len(".fixed")]
            rules = read(os.path.join(d, n + ".rules")).split()
            out.append((n, rules, read(os.path.join(d, n + ".ail")), read(os.path.join(d, f))))
    return out


def why_lines(root, problems):
    """corpus/mistakes/why.tsv: `NAME<TAB>one line that explains the mistake`, sorted by name."""
    path = os.path.join(root, "corpus", "mistakes", "why.tsv")
    why, order = {}, []
    if not os.path.exists(path):
        return why
    for i, line in enumerate(read(path).splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 2 or not parts[1].strip():
            problems.append(f"mistakes/why.tsv:{i}: write NAME, a tab, one line")
            continue
        n, text = parts
        if n in why:
            problems.append(f"mistakes/why.tsv:{i}: {n} is listed twice")
        if len(text) > MAX_WHY:
            problems.append(f"mistakes/why.tsv:{i}: {n} has {len(text)} characters; the limit is {MAX_WHY}")
        why[n] = text.strip()
        order.append(n)
    if order != sorted(order):
        problems.append("mistakes/why.tsv: the rows are not sorted by name")
    return why


def cmd_check(root, args):
    twins = "--twins" in args
    why_complete = "--why-complete" in args
    deny_files, only = [], []
    it = iter(a for a in args if a not in ("--twins", "--why-complete"))
    for a in it:
        if a == "--deny":
            deny_files.append(next(it))
        else:
            only.append(a)
    if os.environ.get("AIL_CORPUS_DENY"):
        deny_files.append(os.environ["AIL_CORPUS_DENY"])
    problems = []
    progs = programs(root)
    deny_names, deny_prints = denied(root, deny_files)
    seen_name, seen_print, per_set = {}, {}, {}
    for p in progs:
        per_set[p.set] = per_set.get(p.set, 0) + 1
        if p.name in seen_name:
            problems.append(f"{p.set}/{p.name}: the name is also in set {seen_name[p.name]}; names are unique")
        seen_name[p.name] = p.set
        if p.name in deny_names:
            problems.append(f"{p.set}/{p.name}: the name is a benchmark or sealed task; it may not be published")
        if only and p.name not in only:
            continue
        ail = check_program(p, problems)
        if ail is None:
            continue
        fp = fingerprint(ail)
        if fp in deny_prints:
            problems.append(f"{p.set}/{p.name}: the text is a benchmark program; it may not be published")
        if fp in seen_print:
            problems.append(f"{p.set}/{p.name}: the same program as {seen_print[fp]}")
        seen_print[fp] = f"{p.set}/{p.name}"
        if twins:
            bad = run_twin(p)
            if bad:
                problems.append(f"{p.set}/{p.name}: {bad}")
    for s, n in sorted(per_set.items()):
        if s != "core" and not NAME_RE.match(s):
            problems.append(f"set {s}: the name must be lower snake case, 2 to 40 characters")
        if s != "core" and n > MAX_SET:
            problems.append(f"set {s}: {n} programs; a set holds at most {MAX_SET}")
    wf = os.path.join(root, "corpus", "waiting.txt")
    if os.path.exists(wf):
        for line in read(wf).splitlines():
            if line.strip() and line.split()[0] not in seen_name:
                problems.append(f"waiting.txt: {line.split()[0]} is not a corpus program")
    pairs = mistakes(root)
    why = why_lines(root, problems)
    pair_names = {n for n, _, _, _ in pairs}
    for n in sorted(set(why) - pair_names):
        problems.append(f"mistakes/why.tsv: {n} is not a pair of tests/mistakes (NAME.ail with NAME.fixed)")
    missing = sorted(pair_names - set(why))
    if why_complete:
        for n in missing:
            problems.append(f"mistakes/why.tsv: {n} has no line")
    for line in problems:
        print("RED", line)
    sets = ", ".join(f"{s} {n}" for s, n in sorted(per_set.items()))
    print(f"corpus: {len(progs)} programs ({sets}){', twins run' if twins else ''}; "
          f"mistakes: {len(pairs)} pairs, {len(pairs) - len(missing)} explained; {len(problems)} red")
    return 1 if problems else 0


def cmd_out(root, names):
    by_name = {p.name: p for p in programs(root)}
    bad = 0
    for n in names:
        p = by_name.get(n)
        if p is None:
            print(f"RED {n}: not a corpus program")
            bad += 1
            continue
        r = subprocess.run([sys.executable, "-I", p.file(".py")], capture_output=True, text=True, timeout=60, cwd=p.dir)
        if r.returncode != 0:
            print(f"RED {n}: the Python twin exits {r.returncode}: {r.stderr.strip()[-200:]}")
            bad += 1
            continue
        with open(p.file(".out"), "w", encoding="utf-8", newline="\n") as f:
            f.write(r.stdout)
    print(f"out: {len(names) - bad} of {len(names)} written")
    return 1 if bad else 0


def cmd_import(root, args):
    deny_files = []
    rest = []
    it = iter(args)
    for a in it:
        if a == "--deny":
            deny_files.append(next(it))
        else:
            rest.append(a)
    if len(rest) != 2:
        print("usage: corpus.py import SRC_DIR SET [--deny FILE]...", file=sys.stderr)
        return 2
    src, set_name = rest
    if not NAME_RE.match(set_name) or set_name == "core":
        print(f"bad set name `{set_name}`", file=sys.stderr)
        return 2
    deny_names, deny_prints = denied(root, deny_files)
    have = {p.name: p.set for p in programs(root)}
    prints = {}
    for p in programs(root):
        if os.path.exists(p.file(".ail")):
            prints[fingerprint(read(p.file(".ail")))] = f"{p.set}/{p.name}"
    dest = os.path.join(root, "corpus", "sets", set_name)
    took = 0
    for n in subdirs(src):
        p = Program(set_name, n, os.path.join(src, n))
        problems = []
        ail = check_program(p, problems)
        if n in have:
            problems.append(f"{n}: the name is taken by {have[n]}/{n}; rename it")
        if n in deny_names:
            problems.append(f"{n}: the name is a benchmark or sealed task")
        if ail is not None:
            fp = fingerprint(ail)
            if fp in deny_prints:
                problems.append(f"{n}: the text is a benchmark program")
            if fp in prints:
                problems.append(f"{n}: the same program as {prints[fp]}")
        if problems:
            for line in problems:
                print("SKIP", line)
            continue
        os.makedirs(os.path.join(dest, n), exist_ok=True)
        for e in (".ail", ".py", ".out"):
            shutil.copyfile(p.file(e), os.path.join(dest, n, n + e))
        have[n] = set_name
        prints[fingerprint(ail)] = f"{set_name}/{n}"
        took += 1
    print(f"import: {took} of {len(subdirs(src))} programs copied to corpus/sets/{set_name}")
    return 0


def cmd_export(root, args):
    if len(args) != 1:
        print("usage: corpus.py export OUT_DIR", file=sys.stderr)
        return 2
    out = args[0]
    os.makedirs(out, exist_ok=True)
    waiting = set()
    wf = os.path.join(root, "corpus", "waiting.txt")
    if os.path.exists(wf):
        waiting = {l.split()[0] for l in read(wf).splitlines() if l.strip()}
    count = 0
    with open(os.path.join(out, "programs.jsonl"), "w", encoding="utf-8", newline="\n") as f:
        for p in programs(root):
            if p.name in waiting:
                continue
            ail = read(p.file(".ail"))
            row = {
                "name": p.name, "set": p.set, "task": task_text(ail), "ail": ail, "py": read(p.file(".py")),
                "out": read(p.file(".out")), "licence": LICENCE,
            }
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
            count += 1
    why = why_lines(root, [])
    pairs = mistakes(root)
    with open(os.path.join(out, "mistakes.jsonl"), "w", encoding="utf-8", newline="\n") as f:
        for n, rules, wrong, right in pairs:
            row = {"name": n, "rules": rules, "why": why.get(n, ""), "wrong": wrong, "right": right, "licence": LICENCE}
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"export: {count} programs, {len(pairs)} mistakes pairs written to {out}")
    return 0


def cmd_self_test():
    """The checker against a small made-up tree: a clean program passes and each planted fault is reported."""
    good_ail = "## Doubles the numbers one to three and prints each.\n\nfn twice(n int) int\n    n * 2\n\n" \
        "fn main()\n    for i in 1..=3\n        print(twice(i))\n\n@test fn two()\n    assert_eq(twice(1), 2)\n"
    good_py = "for i in range(1, 4):\n    print(i * 2)\n"
    good_out = "2\n4\n6\n"

    def put(root, rel, name, ail=good_ail, py=good_py, out=good_out):
        d = os.path.join(root, rel, name)
        os.makedirs(d)
        for e, t in ((".ail", ail), (".py", py), (".out", out)):
            with open(os.path.join(d, name + e), "w", newline="") as f:
                f.write(t)

    def run(root, *args):
        import contextlib, io
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            code = cmd_check(root, list(args))
        return code, buf.getvalue()

    fails = 0

    def expect(label, got, want_code, needle):
        nonlocal fails
        code, text = got
        if code != want_code or needle not in text:
            fails += 1
            print(f"self-test RED {label}: exit {code}, wanted {want_code} and `{needle}`\n{text}")

    with tempfile.TemporaryDirectory() as root:
        os.makedirs(os.path.join(root, "tests", "mistakes"))
        os.makedirs(os.path.join(root, "tools", "agentbench", "tasks", "till-bugfix"))
        os.makedirs(os.path.join(root, "corpus", "mistakes"))
        for e, t in ((".ail", "fn main()\n    print(abs(1))\n"), (".fixed", "fn main()\n    print(1.abs())\n"),
                     (".rules", "expr.name.1\n")):
            with open(os.path.join(root, "tests", "mistakes", "abs_fn" + e), "w") as f:
                f.write(t)
        put(root, "corpus", "doubles")
        other = good_ail.replace("twice", "double").replace("Doubles", "Doubles again:")
        put(root, "corpus/sets/basics", "doubles_two", ail=other)
        expect("clean", run(root, "--twins"), 0, "2 programs (basics 1, core 1), twins run; mistakes: 1 pairs, 0 explained")
        expect("why missing", run(root, "--why-complete"), 1, "abs_fn has no line")
        with open(os.path.join(root, "corpus", "mistakes", "why.tsv"), "w") as f:
            f.write("abs_fn\t`abs` is a method: write `n.abs()`.\nnot_there\tx\n")
        expect("why unknown", run(root), 1, "not_there is not a pair")
        with open(os.path.join(root, "corpus", "mistakes", "why.tsv"), "w") as f:
            f.write("abs_fn\t`abs` is a method: write `n.abs()`.\n")
        expect("why complete", run(root, "--why-complete"), 0, "1 explained")
        put(root, "corpus/sets/basics", "doubles")
        expect("same name", run(root), 1, "names are unique")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "doubles"))
        put(root, "corpus/sets/basics", "copy_cat", ail=good_ail.replace("## Doubles", "##  Doubles"))
        expect("same text", run(root), 1, "the same program as core/doubles")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "copy_cat"))
        put(root, "corpus/sets/basics", "till", ail=other.replace("double", "triple"))
        expect("benchmark name", run(root), 1, "benchmark or sealed task")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "till"))
        deny = os.path.join(root, "deny.txt")
        with open(deny, "w") as f:
            f.write("doubles_two\n")
        expect("deny file", run(root, "--deny", deny), 1, "doubles_two: the name is a benchmark or sealed task")
        put(root, "corpus/sets/basics", "no_tests", ail=other.replace("double", "quad").split("@test")[0] + "\n" * 3)
        expect("no test", run(root), 1, "no `@test`")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "no_tests"))
        put(root, "corpus/sets/basics", "bad_twin", ail=other.replace("double", "quint"), py="print(1)\n")
        expect("twin", run(root, "--twins"), 1, "twin's output differs")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "bad_twin"))
        put(root, "corpus/sets/basics", "net_user", ail=other.replace("double", "sext").replace("\nfn sext", "\nuse std.net\n\nfn sext", 1))
        expect("banned use", run(root), 1, "uses std.net")
        shutil.rmtree(os.path.join(root, "corpus", "sets", "basics", "net_user"))
        out = os.path.join(root, "export")
        import contextlib, io
        with contextlib.redirect_stdout(io.StringIO()):
            cmd_export(root, [out])
        rows = [json.loads(l) for l in open(os.path.join(out, "programs.jsonl"))]
        pairs = [json.loads(l) for l in open(os.path.join(out, "mistakes.jsonl"))]
        if [r["name"] for r in rows] != ["doubles", "doubles_two"] or rows[0]["task"] != "Doubles the numbers one to three and prints each." \
                or pairs[0]["rules"] != ["expr.name.1"] or not pairs[0]["why"]:
            fails += 1
            print("self-test RED export")
        src = os.path.join(root, "incoming")
        put(src, "", "doubles")
        put(src, "", "halves", ail=other.replace("double", "half"))
        with contextlib.redirect_stdout(io.StringIO()) as buf:
            cmd_import(root, [src, "more"])
        if "1 of 2 programs" not in buf.getvalue() or not os.path.exists(os.path.join(root, "corpus", "sets", "more", "halves", "halves.ail")):
            fails += 1
            print("self-test RED import\n" + buf.getvalue())
    print(f"corpus self-test: {fails} red")
    return 1 if fails else 0


def main():
    args = sys.argv[1:]
    root = os.path.normpath(os.path.join(HERE, "..", ".."))
    if len(args) >= 2 and args[0] == "--root":
        root, args = args[1], args[2:]
    if not args:
        print(__doc__, file=sys.stderr)
        return 2
    cmd, rest = args[0], args[1:]
    if cmd == "check":
        return cmd_check(root, rest)
    if cmd == "out":
        return cmd_out(root, rest)
    if cmd == "import":
        return cmd_import(root, rest)
    if cmd == "export":
        return cmd_export(root, rest)
    if cmd == "self-test":
        return cmd_self_test()
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
