"""Peak resident memory of the perfbench programs: AIL against C, Rust and Go (Linux).

    python3 -I tools/perfbench/mem.py [-o OUT.md] [--ail ./ail] [--only a,b] [--threads N]

Builds each program with every compiler, runs it once (AIL_THREADS=1 unless
--threads), and reports ru_maxrss (peak resident set, MB) of the process from
wait4, plus wall seconds. Measurement only (ADR 0070). Sizes are the sizes of
docs/benchmarks.md; text/ programs run at the sizes fixed in their source.
"""
import argparse, os, subprocess, sys, tempfile, time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
P = os.path.join(ROOT, "tools", "perfbench")
# (name, dir, args)
PROGS = [
    ("collatz", "programs", "3200000"), ("nbody", "programs", "12800000"),
    ("spectralnorm", "programs", "4000"), ("mandelbrot", "programs", "3840"),
    ("fannkuch", "programs", "11"), ("binarytrees", "programs", "19"),
    ("fasta", "programs", "64000000"), ("wordfreq", "programs", "32000000"),
    ("matmul", "programs", "800"), ("sieve", "programs", "96000000"),
    ("lzr", "big", "12000000"), ("vm", "big", "700000"), ("raytrace", "big", "4000"),
    ("matmul1400", "big", "1400"),
    ("splitline", "text", ""), ("floatparse", "text", ""), ("charat", "text", ""),
    ("charwalk", "text", ""), ("gcchurn", "text", ""), ("gcints", "text", ""),
]


def run(cmd, env=None, stdin=None):
    t = time.time()
    p = subprocess.Popen(cmd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, stdin=stdin)
    _, st, ru = os.wait4(p.pid, 0)
    return ru.ru_maxrss / 1024.0, time.time() - t


def sh(cmd, **kw):
    return subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, **kw)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("-o")
    ap.add_argument("--ail", default=os.path.join(ROOT, "ail"))
    ap.add_argument("--only")
    ap.add_argument("--threads", default="1")
    ap.add_argument("--runs", type=int, default=1)
    a = ap.parse_args()
    only = set(a.only.split(",")) if a.only else None
    tmp = tempfile.mkdtemp(prefix="mem")
    env = dict(os.environ, AIL_THREADS=a.threads)
    rows = []
    for name, d, args in PROGS:
        if only and name not in only:
            continue
        base = os.path.join(P, d, name)
        argv = args.split() if args else []
        cell = {}
        exes = {}
        out = os.path.join(tmp, name + "_ail")
        try:
            sh([a.ail, "build", base + ".ail", "-o", out])
            exes["AIL"] = out
        except Exception as e:
            print("ail build failed", name, e, file=sys.stderr)
        if os.path.exists(base + ".c"):
            o = os.path.join(tmp, name + "_c")
            try:
                sh(["gcc", "-std=c11", "-O2", "-ffp-contract=off", "-o", o, base + ".c", "-lm"])
                exes["C"] = o
            except Exception:
                pass
        if os.path.exists(base + ".go"):
            o = os.path.join(tmp, name + "_go")
            try:
                sh(["go", "build", "-o", o, base + ".go"], cwd=tmp)
                exes["Go"] = o
            except Exception:
                pass
        if os.path.exists(base + ".rs"):
            o = os.path.join(tmp, name + "_rs")
            try:
                sh(["rustc", "-C", "opt-level=3", "--edition", "2021", "-o", o, base + ".rs"])
                exes["Rust"] = o
            except Exception:
                pass
        for lang, exe in exes.items():
            best = None
            for _ in range(a.runs):
                m, t = run([exe] + argv, env=env)
                if best is None or m < best[0]:
                    best = (m, t)
            cell[lang] = best
        rows.append((name, args, cell))
        print(name, {k: "%.0f MB %.2fs" % v for k, v in cell.items()}, flush=True)
    langs = ["AIL", "C", "Rust", "Go"]
    lines = ["| program | size | " + " | ".join(l + " MB" for l in langs) + " | AIL / C | AIL s | C s |",
             "|---|---:|" + "---:|" * (len(langs) + 3)]
    for name, args, cell in rows:
        cs = [("%.1f" % cell[l][0]) if l in cell else "" for l in langs]
        ratio = "%.2f" % (cell["AIL"][0] / cell["C"][0]) if "AIL" in cell and "C" in cell else ""
        ts = ["%.2f" % cell[l][1] if l in cell else "" for l in ("AIL", "C")]
        lines.append("| %s | %s | %s | %s | %s | %s |" % (name, args, " | ".join(cs), ratio, ts[0], ts[1]))
    text = "\n".join(lines) + "\n"
    print(text)
    if a.o:
        open(a.o, "w").write(text)


main()
