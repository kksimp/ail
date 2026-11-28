"""A/B of two trees: min wall time and peak RSS of each program, base against head.

    python3 -I tools/perfbench/memab.py BASE_TREE HEAD_TREE [RUNS] [NAMES]

Each tree has its own `./ail`; every program is built by each tree (its own
runtime) and run RUNS times (default 5) with AIL_THREADS=1 and, for the
parallel column, unset. Prints base and head side by side with the ratio.
Measurement only (ADR 0070).
"""
import os, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
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
    ("bt_par", "alloc", "16"), ("objs_par", "alloc", "200000"), ("gc_heap", "alloc", "2000000"),
]


def run(exe, argv, env):
    t = time.time()
    p = subprocess.Popen([exe] + argv, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, _, ru = os.wait4(p.pid, 0)
    return time.time() - t, ru.ru_maxrss / 1048576.0 * 1024


def main():
    base, head = sys.argv[1], sys.argv[2]
    runs = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    only = set(sys.argv[4].split(",")) if len(sys.argv) > 4 else None
    tmp = tempfile.mkdtemp(prefix="memab")
    print("| program | base s | head s | time ratio | base MB | head MB | MB ratio | par base s | par head s |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for name, d, args in PROGS:
        if only and name not in only:
            continue
        src = os.path.join(HERE, d, name + ".ail")
        argv = args.split() if args else []
        exes = []
        for tree in (base, head):
            out = os.path.join(tmp, name + "_" + str(len(exes)))
            subprocess.run([os.path.join(tree, "ail"), "build", "--no-cache", src, "-o", out], cwd=tree,
                           check=True, stdout=subprocess.DEVNULL)
            exes.append(out)
        res = []
        for threads in ("1", None):
            env = dict(os.environ)
            env.pop("AIL_THREADS", None)
            if threads:
                env["AIL_THREADS"] = threads
            best = [None, None]
            # alternate base and head so drift hits both
            for _ in range(runs):
                for k in (0, 1):
                    t, m = run(exes[k], argv, env)
                    if best[k] is None or t < best[k][0]:
                        best[k] = (t, m)
            res.append(best)
        (b, h), (pb, ph) = res
        print("| %s | %.3f | %.3f | %.2f | %.1f | %.1f | %.2f | %.3f | %.3f |" % (
            name, b[0], h[0], h[0] / b[0], b[1], h[1], h[1] / b[1], pb[0], ph[0]), flush=True)


main()
