"""A/B timing for the allocator and collector with threads (ADR 0943, 0944).

    python3 tools/perfbench/alloc/run.py BASE_TREE HEAD_TREE [RUNS]

Each tree is a worktree of kksimp/ail with its own `./ail`. Every program here
and ../programs/binarytrees.ail and fasta.ail is built by each tree (its own
runtime), also with AIL_PARALLEL=0, run RUNS times (default 7), and the
minimum wall time printed. Outputs of the two trees must match. Measurement
only (ADR 0070); run on a quiet machine, never in a cloud container.
"""
import os, subprocess, sys, tempfile, time

here = os.path.dirname(os.path.abspath(__file__))
progs = [
    (os.path.join(here, "bt_par.ail"), "16"),
    (os.path.join(here, "objs_par.ail"), "200000"),
    (os.path.join(here, "gc_heap.ail"), "2000000"),
    (os.path.join(here, "..", "programs", "binarytrees.ail"), "16"),
    (os.path.join(here, "..", "programs", "fasta.ail"), "500000"),
]


def build(tree, src, out, par):
    env = dict(os.environ)
    if not par:
        env["AIL_PARALLEL"] = "0"
    subprocess.run([os.path.join(tree, "ail"), "build", src, "-o", out], cwd=tree, env=env, check=True,
                   stdout=subprocess.DEVNULL)


def best(exe, arg, runs):
    ts, out = [], None
    for _ in range(runs):
        t = time.perf_counter()
        r = subprocess.run([exe, arg], capture_output=True)
        ts.append(time.perf_counter() - t)
        if r.returncode != 0:
            return None, "exit %d: %s" % (r.returncode, r.stderr.decode()[-300:])
        out = r.stdout
    return min(ts), out


def main():
    base, head = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
    runs = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    tmp = tempfile.mkdtemp(prefix="allocab-")
    print("program\tmode\tbase s\thead s\thead/base")
    bad = 0
    for src, arg in progs:
        name = os.path.basename(src)[:-4]
        for par in (True, False):
            mode = "pool" if par else "AIL_PARALLEL=0"
            res = []
            for tag, tree in (("base", base), ("head", head)):
                exe = os.path.join(tmp, "%s-%s-%d" % (name, tag, par))
                build(tree, src, exe, par)
                res.append(best(exe, arg, runs))
            (tb, ob), (th, oh) = res
            if tb is None or th is None or ob != oh:
                bad += 1
                print("%s\t%s\tFAIL base=%s head=%s" % (name, mode, ob if tb is None else "ok",
                                                         oh if th is None else ("ok" if ob == oh else "differs")))
                continue
            print("%s\t%s\t%.3f\t%.3f\t%.2f" % (name, mode, tb, th, th / tb))
    sys.exit(1 if bad else 0)


main()
