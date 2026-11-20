#!/usr/bin/env python3
"""Render docs/benchmarks.md from the raw numbers (measurement only, ADR 0070).

    publish.py [--latency LAT.json]... [--host DIR]... [--text TEXT.json] RACE_SMALL.json RACE_BIG.json TOKENS.json TEMPLATE.md OUT.md ENV_FILE
    publish.py --skeleton TEMPLATE.md OUT.md

tools/bench/publish.sh produces the inputs and calls this. The template holds the prose;
the markers {{ENV}}, {{SPEED}}, {{BIG}}, {{TEXT}}, {{SPREAD}}, {{HOSTS}}, {{GAPS}}, {{TOKENS}}, {{LATENCY}} and {{SUMMARY}} are replaced by
generated text. {{GAPS}} lists the programs whose AIL / gcc ratio is GAP_RATIO or more on some host, with the cause recorded for each
in docs/design/benchmark-gaps.md, so a release candidate changes only that docs file and this page, never the script. Each --host DIR is another host's `publish.sh --keep-dir` directory (small.json, big.json,
env.txt, latency.json): its speed tables go under {{HOSTS}} and its latency run joins the edit-loop table.
Every sentence in {{SUMMARY}} is computed from the numbers, so it cannot disagree
with the tables. --skeleton writes the page with every number replaced by PENDING (no inputs),
for the page that ships before the release run.
"""
import json, math, os, re, sys

PENDING = "PENDING"
args = sys.argv[1:]
lat_files, host_dirs, text_file = [], [], None
while args and args[0] in ("--latency", "--host", "--skeleton", "--text"):
    if args[0] == "--latency":
        lat_files.append(args[1]); args = args[2:]
    elif args[0] == "--text":
        text_file = args[1]; args = args[2:]
    elif args[0] == "--host":
        host_dirs.append(args[1]); args = args[2:]
    else:
        break
skeleton = bool(args) and args[0] == "--skeleton"
if skeleton:
    tmpl, out = args[1:3]
else:
    small, big, tokens, tmpl, out, envf = args[:6]
    small, big, tokens = (json.load(open(p)) for p in (small, big, tokens))
    textrace = json.load(open(text_file)) if text_file and os.path.isfile(text_file) else None
    lats = [json.load(open(p)) for p in lat_files]
    hosts = []
    for d in host_dirs:
        h = {k: json.load(open(os.path.join(d, k + ".json"))) for k in ("small", "big")}
        h["env"] = open(os.path.join(d, "env.txt")).read().strip()
        tp = os.path.join(d, "text.json")
        h["text"] = json.load(open(tp)) if os.path.isfile(tp) else None
        lp = os.path.join(d, "latency.json")
        h["lat"] = json.load(open(lp)) if os.path.isfile(lp) else None
        line = next((l for l in h["env"].splitlines() if l.startswith("- host:")), "- host: " + d)
        h["name"] = line[len("- host:"):].split(",")[0].strip()
        hosts.append(h)
        if h["lat"]:  # a host directory's own latency run replaces a --latency file with the same host label
            lats = [l for l in lats if l["host"] != h["lat"]["host"]] + [h["lat"]]


def stat(c):
    """The number a cell reports: the median of the runs (best-of-N hides the spread); old files without one give the min."""
    return c.get("med", c.get("min")) if c and "min" in c else None


def cell(c):
    return "%.3f" % stat(c) if stat(c) is not None else (c or {}).get("err", "-")


def ratio(a, b):
    if stat(a) is not None and stat(b):
        return "%.2f" % (stat(a) / stat(b))
    return "-"


def spread(c):
    return "%.3f–%.3f" % (c["min"], c.get("max", c["min"])) if c and "min" in c else "-"


# A row ran parallel when AIL_THREADS=1 is this much slower than the default run (the runs' own noise is about 10%).
PARALLEL_FACTOR = 1.15


def parallel(c):
    a, b = stat(c.get("native")), stat(c.get("native1t"))
    return a is not None and b is not None and b > PARALLEL_FACTOR * a


# The other compiled languages, in column order: (race column, heading).
OTHERS = [("rustc", "Rust"), ("go", "Go")]


def others(race):
    return [(k, h) for k, h in OTHERS if any(k in r["cells"] for r in race["rows"])]


def gcc_is_clang(env):
    """macOS: `gcc` is Apple clang, so the gcc and clang columns are one compiler."""
    return any(l.startswith("- gcc:") and "clang" in l for l in env.splitlines())


# Runs under this many seconds are within the noise band (drag.size.1); calibration aims above it.
NOISE_S = 0.3


def speed(race, py, one_cc=False):
    """one_cc: show the single C compiler once, as `clang -O2` (gcc_is_clang)."""
    ot = others(race)
    cc = ["clang -O2"] if one_cc else ["gcc -O2", "clang -O2"]
    t1 = any("native1t" in r["cells"] for r in race["rows"])
    ref = "clang" if one_cc else "gcc"
    head = (["program", "size", "AIL"] + (["AIL 1T"] if t1 else []) + cc + [h for _, h in ot] + (["Python"] if py else [])
            + ["AIL / " + ref] + (["AIL 1T / " + ref] if t1 else []) + ["AIL / " + h for _, h in ot] + (["AIL / Python"] if py else []))
    rows = ["| " + " | ".join(head) + " |", "|---|" + "---:|" * (len(head) - 1)]
    noisy = any_par = False
    for r in race["rows"]:
        c = r["cells"]
        quick = num(c.get("gcc")) is not None and num(c.get("gcc")) < NOISE_S
        noisy = noisy or quick
        par = parallel(c)
        any_par = any_par or par
        line = ([r["program"] + (" †" if quick else "") + (" ‡" if par else ""), r.get("size") or "-", cell(c.get("native"))]
                + ([cell(c.get("native1t"))] if t1 else []) + [cell(c.get("gcc"))] + ([] if one_cc else [cell(c.get("clang"))]))
        for k, _ in ot:
            line.append(cell(c.get(k)))
        if py:
            line.append(cell(c.get("python")))
        line.append(ratio(c.get("native"), c.get("gcc")))
        if t1:
            line.append(ratio(c.get("native1t"), c.get("gcc")))
        for k, _ in ot:
            line.append(ratio(c.get("native"), c.get(k)))
        if py:
            line.append(ratio(c.get("native"), c.get("python")))
        rows.append("| " + " | ".join(line) + " |")
    if any_par:
        rows += ["", "‡ This row ran on several cores: with `AIL_THREADS=1` it is more than %d%% slower. Read it against the "
                 "`AIL 1T` column, since the C, Rust, Go and Python twins are single-threaded." % round((PARALLEL_FACTOR - 1) * 100)]
    if noisy:
        rows += ["", "† The C column ran under %.1f s on this row (its size did not calibrate), so the row's ratios are within noise." % NOISE_S]
    return "\n".join(rows)


def spread_table(race, py):
    """Min–max of the runs behind each median, one row per program, for every column the race measured."""
    cols = [(k, h) for k, h in (("native", "AIL"), ("native1t", "AIL 1T"), ("gcc", "gcc -O2"), ("clang", "clang -O2"),
                                ("rustc", "Rust"), ("go", "Go"), ("python", "Python"))
            if any(k in r["cells"] for r in race["rows"])]
    rows = ["| program | " + " | ".join(h for _, h in cols) + " |", "|---|" + "---:|" * len(cols)]
    for r in race["rows"]:
        rows.append("| %s | %s |" % (r["program"], " | ".join(spread(r["cells"].get(k)) for k, _ in cols)))
    return "\n".join(rows)


def tok(name):
    return next((r["mean"] for r in tokens if os.path.basename(r["file"]) == name), None)



SMALL_NAMES = "collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve".split()
BIG_NAMES = "lzr vm raytrace matmul1400".split()
TEXT_NAMES = "readline fields splitline floatparse charat charwalk gcchurn gcints".split()


def num(c):
    return stat(c)


def geomean(xs):
    return math.exp(sum(math.log(x) for x in xs) / len(xs)) if xs else None


def pairs(race, ref="gcc", col="native"):
    """(program, AIL/ref) for every row where both columns have a time; a † row (gcc under NOISE_S) is left out."""
    got = []
    for r in race["rows"]:
        a, g = num(r["cells"].get(col)), num(r["cells"].get(ref))
        q = num(r["cells"].get("gcc"))
        if q is not None and q < NOISE_S:
            continue
        if a and g:
            got.append((r["program"], a / g))
    return got


TOKEN_OTHERS = [(".rs", "Rust"), (".go", "Go")]


def token_table():
    """Rows: AIL, C, the other compiled languages that have twins, Python. tot = [AIL, C, Python, {heading: total}]."""
    ot = [(e, h) for e, h in TOKEN_OTHERS if all(tok(r["program"] + e) is not None for r in small["rows"])]
    head = ["program", "AIL", "C"] + [h for _, h in ot] + ["Python", "AIL / Python"]
    trows = ["| " + " | ".join(head) + " |", "|---|" + "---:|" * (len(head) - 1)]
    tot = [0, 0, 0, {h: 0 for _, h in ot}]
    for r in small["rows"]:
        n = r["program"]
        a, c, p = tok(n + ".ail"), tok(n + ".c"), tok(n + ".py")
        if None in (a, c, p):
            continue
        xs = [tok(n + e) for e, _ in ot]
        tot[0] += a; tot[1] += c; tot[2] += p
        for (_, h), x in zip(ot, xs):
            tot[3][h] += x
        trows.append("| %s | %s | %.2f |" % (n, " | ".join("%.0f" % v for v in [a, c] + xs + [p]), a / p))
    if tot[2]:
        trows.append("| **total** | %s | %.2f |" % (" | ".join("%.0f" % v for v in [tot[0], tot[1]] + [tot[3][h] for _, h in ot] + [tot[2]]), tot[0] / tot[2]))
    return "\n".join(trows), tot


def ms(c):
    return "%.0f" % c["min"] if c and c.get("ok") else "-"


LAT_ORDER = ["check", "context", "test", "build cold", "build cached", "run cached"]


def latency_table(lats):
    """Edit loop: one row per command and program, one column per host (best of the runs, ms)."""
    if not lats:
        return "No latency run was included."
    names = [n for n in ("hello", "json", "ailtest") if n in lats[0]["programs"]]
    head = ["command", "program (lines)"] + [l["host"] for l in lats]
    rows = ["| " + " | ".join(head) + " |", "|---|---|" + "---:|" * len(lats)]
    for cmd in LAT_ORDER:
        for n in names:
            cells = [next((r for r in l["rows"] if r["cmd"] == cmd and r["program"] == n), None) for l in lats]
            if not any(cells):
                continue
            rows.append("| `ail %s` | %s (%d) | %s |" % (cmd.split()[0] + (" --cold" if cmd == "build cold" else ""),
                        n, lats[0]["programs"][n], " | ".join(ms(c) for c in cells)))
    for n in names:
        first = [l["mcp"].get(n) for l in lats]
        rep = first
        rows.append("| `ail mcp` `ail_check`, first call | %s (%d) | %s |" % (n, lats[0]["programs"][n],
                    " | ".join("%.0f" % m["first"] if m and m.get("ok") else "-" for m in first)))
        rows.append("| `ail mcp` `ail_check`, repeat | %s (%d) | %s |" % (n, lats[0]["programs"][n],
                    " | ".join("%.0f" % m["repeat"] if m and m.get("ok") and m.get("repeat") is not None else "-" for m in rep)))
    if any("fresh_clone" in l for l in lats):
        rows.append("| first `./ail run examples/hello.ail` in a fresh clone | hello (2) | %s |" % " | ".join(
            ("%.1f s" % (l["fresh_clone"]["ms"] / 1000.0) if l["fresh_clone"].get("ok") else "-") if "fresh_clone" in l else "n/a" for l in lats))
    return "\n".join(rows)


def text_table(race, one_cc=False):
    """Text and allocation micros: AIL (default threads and one thread) against the one gcc column."""
    ref = "clang" if one_cc else "gcc"
    head = ["program", "what it does", "AIL", "AIL 1T", ref + " -O2", "AIL / " + ref, "AIL 1T / " + ref]
    rows = ["| " + " | ".join(head) + " |", "|---|---|---:|---:|---:|---:|---:|"]
    for r in race["rows"]:
        c = r["cells"]
        rows.append("| %s | %s | %s | %s | %s | %s | %s |" % (r["program"], r.get("what", ""), cell(c.get("native")), cell(c.get("native1t")),
                    cell(c.get("gcc")), ratio(c.get("native"), c.get("gcc")), ratio(c.get("native1t"), c.get("gcc"))))
    return "\n".join(rows)


# A program is a known gap when AIL / gcc (default threads) is this much above 1 on some host (rows under NOISE_S left out).
GAP_RATIO = 1.10
CAUSES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "docs", "design", "benchmark-gaps.md")


def causes():
    """`- `name`: text` lines of docs/design/benchmark-gaps.md, the recorded cause of each gap."""
    got = {}
    try:
        for l in open(CAUSES, encoding="utf-8"):
            m = re.match(r"^- `([^`]+)`: (.+)$", l.rstrip())
            if m:
                got[m.group(1)] = m.group(2)
    except OSError:
        pass
    return got


def gaps(main_host, others):
    """Known gaps, computed from the measured rows: every program at GAP_RATIO or more on some host, with its cause."""
    found = {}
    for hname, one, races in [main_host] + others:
        for race in races:
            for r in race["rows"]:
                c = r["cells"]
                a, g = num(c.get("native")), num(c.get("gcc"))
                if not a or not g or g < NOISE_S or a / g < GAP_RATIO:
                    continue
                t = num(c.get("native1t"))
                note = "%s %.2f" % (hname, a / g) + (" (%.2f with `AIL_THREADS=1`)" % (t / g) if t and parallel(c) else "")
                found.setdefault(r["program"], []).append(note)
    if not found:
        return "No program is %.2f or more above gcc on any host (rows whose C column ran under %.1f s are left out)." % (GAP_RATIO, NOISE_S)
    known = causes()
    out = ["Programs whose AIL / gcc ratio is %.2f or more on at least one host (rows whose C column ran under %.1f s are left out; on macOS the C "
           "compiler is Apple clang). The twins are single-threaded and AIL auto-parallelizes, so a row that ran on several cores also shows its "
           "`AIL_THREADS=1` ratio. The causes are recorded by hand in [design/benchmark-gaps.md](design/benchmark-gaps.md); the list of programs and "
           "ratios is computed from the tables above." % (GAP_RATIO, NOISE_S), ""]
    for prog, notes in found.items():
        out.append("- `%s`: %s. Cause: %s" % (prog, "; ".join(notes), known.get(prog, "none is recorded for this program yet.")))
    return "\n".join(out)


def skeleton_tables():
    sp = ["| program | size | AIL | AIL 1T | gcc -O2 | clang -O2 | Python | AIL / gcc | AIL 1T / gcc | AIL / Python |", "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
    sp += ["| %s | %s | %s |" % (n, PENDING, " | ".join([PENDING] * 8)) for n in SMALL_NAMES]
    bg = ["| program | size | AIL | AIL 1T | gcc -O2 | clang -O2 | AIL / gcc | AIL 1T / gcc |", "|---|---|---:|---:|---:|---:|---:|---:|"]
    bg += ["| %s | %s | %s |" % (n, PENDING, " | ".join([PENDING] * 6)) for n in BIG_NAMES]
    tk = ["| program | AIL | C | Python | AIL / Python |", "|---|---:|---:|---:|---:|"]
    tk += ["| %s | %s |" % (n, " | ".join([PENDING] * 4)) for n in SMALL_NAMES]
    tx = ["| program | what it does | AIL | AIL 1T | gcc -O2 | AIL / gcc | AIL 1T / gcc |", "|---|---|---:|---:|---:|---:|---:|"]
    tx += ["| %s | %s | %s |" % (n, PENDING, " | ".join([PENDING] * 5)) for n in TEXT_NAMES]
    lat = ["| command | program (lines) | host |", "|---|---|---:|"]
    for cmd in LAT_ORDER:
        for n in ("hello", "json", "ailtest"):
            if cmd == "run cached" and n == "ailtest":
                continue
            lat.append("| `ail %s` | %s | %s |" % (cmd, n, PENDING))
    return "\n".join(sp), "\n".join(bg), "\n".join(tk), "\n".join(lat), "\n".join(tx)


def geo_line(race, ref, col="native"):
    rp = pairs(race, ref, col)
    return "%.2f" % geomean([r for _, r in rp]) if rp else "-"


def commit_of(env):
    line = next((l for l in env.splitlines() if l.startswith("- AIL commit:")), "- AIL commit: ?")
    return line[len("- AIL commit:"):].split("(")[0].strip()


def hosts_section(hosts):
    """One block per other host: its environment and both speed tables."""
    if not hosts:
        return ""
    out = ["## Speed on the other hosts", "",
           "The same command on each host, with the same calibration rule; every size is that host's own."]
    out[-1] += " Commits: " + "; ".join("%s %s" % (h["name"], commit_of(h["env"])) for h in hosts) + "."
    for h in hosts:
        one = gcc_is_clang(h["env"])
        out += ["", "### %s" % h["name"], "", h["env"], ""]
        if one:
            out += ["`gcc` on this host is Apple clang, so the C column is that one compiler (run as `gcc -O2`, the "
                    "calibration reference).", ""]
        out += ["Small programs:", "", speed(h["small"], True, one), "", "Larger programs:", "", speed(h["big"], False, one)]
        if h.get("text"):
            out += ["", "Text and allocation programs:", "", text_table(h["text"], one)]
        out += [
                "", "Spread (min–max wall seconds of the runs behind each median):", "", spread_table(h["small"], True), "", spread_table(h["big"], False)]
    return "\n".join(out)


def summary(sm, bg, tot, lats, hosts=()):
    out = []
    sp, bp = pairs(sm), pairs(bg)
    if sp:
        faster = [p for p, r in sp if r < 1]
        slower = [p for p, r in sp if r >= 1]
        out.append("- **Small programs.** AIL / gcc is below 1 on %d of %d (%s) and 1 or above on %d (%s); the geometric mean is %.2f."
                   % (len(faster), len(sp), ", ".join(faster) or "none", len(slower), ", ".join(slower) or "none", geomean([r for _, r in sp])))
    s1 = pairs(sm, "gcc", "native1t") + pairs(bg, "gcc", "native1t")
    if s1:
        both = pairs(sm, "gcc") + pairs(bg, "gcc")
        out.append("- **Single thread.** With `AIL_THREADS=1` the geometric mean of AIL 1T / gcc over the %d programs is %.2f; with the default "
                   "thread count it is %.2f. The programs marked ‡ ran on several cores by default (%s)."
                   % (len(s1), geomean([r for _, r in s1]), geomean([r for _, r in both]),
                      ", ".join(r["program"] for race in (sm, bg) for r in race["rows"] if parallel(r["cells"])) or "none"))
    for key, name, how in (("rustc", "Rust", "`rustc -C opt-level=3`"), ("go", "Go", "`go build`")):
        rp = pairs(sm, key) + pairs(bg, key)
        if rp:
            faster = [p for p, r in rp if r < 1]
            out.append("- **%s (%s).** AIL / %s is below 1 on %d of %d programs (%s); the geometric mean over all %d is %.2f."
                       % (name, how, name, len(faster), len(rp), ", ".join(faster) or "none", len(rp), geomean([r for _, r in rp])))
    if bp:
        out.append("- **Larger programs.** AIL / gcc is " + ", ".join("%.2f on %s" % (r, p) for p, r in bp)
                   + "; the geometric mean is %.2f." % geomean([r for _, r in bp]))
    for h in hosts:
        refs = [("gcc", "clang" if gcc_is_clang(h["env"]) else "gcc")] + [(k, n) for k, n in OTHERS if any(k in r["cells"] for r in h["small"]["rows"] + h["big"]["rows"])]
        out.append("- **On %s.** Geometric mean of AIL / X over the small programs, then the larger ones (rows marked † left out): %s."
                   % (h["name"], "; ".join("%s %s and %s" % (n, geo_line(h["small"], k), geo_line(h["big"], k)) for k, n in refs)))
    if tot[2]:
        out.append("- **Tokens.** Across the ten programs AIL source is %.0f%% of the Python source and %.0f%% of the C source by mean token count (%.0f, %.0f, %.0f)."
                   % (100.0 * tot[0] / tot[2], 100.0 * tot[0] / tot[1], tot[0], tot[1], tot[2]))
        for h, t in tot[3].items():
            if t:
                out[-1] += " The %s source is %.0f tokens, so AIL is %.0f%% of it." % (h, t, 100.0 * tot[0] / t)
    for l in lats:
        d = {(r["cmd"], r["program"]): r for r in l["rows"]}
        c = d.get(("check", "hello"))
        m = l["mcp"].get("hello", {})
        t = d.get(("check", "ailtest"))
        if c and c.get("ok"):
            s = "- **Edit loop on %s.** `ail check` of hello takes %.0f ms" % (l["host"], c["min"])
            if t and t.get("ok"):
                s += " and of the %d-line `ailtest` %.0f ms" % (l["programs"]["ailtest"], t["min"])
            if m.get("ok") and m.get("repeat") is not None:
                s += "; a repeated `ail_check` through a warm `ail mcp` takes %.0f ms" % m["repeat"]
            out.append(s + ".")
    return "\n".join(out) or "No summary: the run produced no comparable rows."


text = open(tmpl).read()
if skeleton:
    sp, bg, tk, lat, tx = skeleton_tables()
    repl = (("ENV", "- date: PENDING\n- AIL commit: PENDING\n- host: PENDING\n- gcc, clang, python: PENDING\n- runs per cell: PENDING"),
            ("SPEED", sp), ("BIG", bg), ("TEXT", tx), ("SPREAD", ""), ("HOSTS", ""), ("GAPS", PENDING + ": computed from the measured rows."), ("TOKENS", tk), ("LATENCY", lat),
            ("SUMMARY", "PENDING: the release run fills this in from the measured numbers. This page has not been measured yet."))
else:
    tt, tot = token_table()
    env = open(envf).read().strip()
    one = gcc_is_clang(env)
    gap_hosts = [(h["name"], gcc_is_clang(h["env"]), [h["small"], h["big"]] + ([h["text"]] if h.get("text") else [])) for h in hosts]
    main_name = next((l[len("- host:"):].split(",")[0].strip() for l in env.splitlines() if l.startswith("- host:")), "this host")
    repl = (("ENV", env), ("SPEED", speed(small, True, one)), ("BIG", speed(big, False, one)),
            ("TEXT", text_table(textrace, one) if textrace else "No text run was included."),
            ("GAPS", gaps((main_name, one, [small, big] + ([textrace] if textrace else [])), gap_hosts)),
            ("SPREAD", spread_table(small, True) + "\n\nLarger programs:\n\n" + spread_table(big, False)),
            ("HOSTS", hosts_section(hosts)), ("TOKENS", tt), ("LATENCY", latency_table(lats)),
            ("SUMMARY", summary(small, big, tot, lats, hosts)))
for k, v in repl:
    text = text.replace("{{%s}}" % k, v)
while "\n\n\n" in text:  # an empty marker (no other hosts) leaves no gap
    text = text.replace("\n\n\n", "\n\n")
open(out, "w").write(text)
