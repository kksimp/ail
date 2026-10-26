"""Multi-core drag race: native AIL (`par for`), C (OpenMP) and Python (multiprocessing)
on the programs in this directory (docs/design/native-performance.md, ADRs 0599-0601).
Usage: python3 -I run.py [--cores 1,4] [--runs 3] [--python] [NAME...]
Each program runs pinned to the first N cores with `taskset` (Linux); macOS has no pinning,
so it runs with all cores only, and C uses OpenMP when libomp is installed, else serial; the pool of AIL, the
OpenMP team and the Python workers all follow that count. The three outputs must be
identical or the row says DIFF. Measurement only (ADR 0070)."""
import hashlib, os, platform, shutil, subprocess, sys, tempfile, time

MAC = platform.system() == "Darwin"

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
SIZES = {"parsum": 3000000, "autopar": 3000000, "mandelbrot": 1000, "spectralnorm": 1500, "nbody": 1500}


def args():
    cores, runs, py, names = ([os.cpu_count()] if MAC else [1, os.cpu_count()]), 3, False, []
    a = sys.argv[1:]
    while a:
        x = a.pop(0)
        if x == "--cores":
            cores = [int(c) for c in a.pop(0).split(",")]
        elif x == "--runs":
            runs = int(a.pop(0))
        elif x == "--python":
            py = True
        else:
            names.append(x)
    return cores, runs, py, names or list(SIZES)


def run(cmd, n, env=None):
    """Best wall time of several runs, and the hash of the output."""
    pinned = cmd if MAC else ["taskset", "-c", f"0-{n - 1}"] + cmd
    best, digest = None, None
    for _ in range(RUNS):
        t = time.perf_counter()
        p = subprocess.run(pinned, stdout=subprocess.PIPE, env=env)
        dt = time.perf_counter() - t
        if p.returncode != 0:
            return None, None
        best = dt if best is None or dt < best else best
        digest = hashlib.sha256(p.stdout).hexdigest()[:12]
    return best, digest


def compile_c(src, out):
    if not MAC:
        subprocess.run(["gcc", "-std=c11", "-O2", "-fopenmp", "-ffp-contract=off", "-o", out, src, "-lm"], check=True)
        return
    # macOS clang has no OpenMP unless libomp is installed (Homebrew); without it the pragmas
    # are ignored and the C column is the serial program.
    base = ["clang", "-std=c11", "-O2", "-ffp-contract=off", "-o", out, src, "-lm"]
    for omp in ("/opt/homebrew/opt/libomp", "/usr/local/opt/libomp"):
        if os.path.isdir(omp):
            cmd = base[:4] + ["-Xpreprocessor", "-fopenmp", f"-I{omp}/include", f"-L{omp}/lib", "-lomp"] + base[4:]
            if subprocess.run(cmd).returncode == 0:
                return
    print("note: no libomp, so the C column is serial", file=sys.stderr)
    subprocess.run(base, check=True)


cores, RUNS, with_python, names = args()
tmp = tempfile.mkdtemp(prefix="ail-par-")
print("| program | size | cores | native AIL | C OpenMP | Python | AIL vs C | AIL vs Python |")
print("|---|---:|---:|---:|---:|---:|---:|---:|")
for name in names:
    size = str(SIZES[name])
    exe = os.path.join(tmp, name)
    subprocess.run([os.path.join(ROOT, "ail"), "build", f"{HERE}/{name}.ail", "-o", exe], check=True, cwd=ROOT)
    cexe = os.path.join(tmp, "c-" + name)
    compile_c(f"{HERE}/{name}.c", cexe)
    for n in cores:
        a, ha = run([exe, size], n)
        env = dict(os.environ, OMP_NUM_THREADS=str(n))
        c, hc = run([cexe, size], n, env)
        py, hp = (None, None)
        if with_python:
            env = dict(os.environ, WORKERS=str(n))
            py, hp = run(["python3", "-I", f"{HERE}/{name}.py", size], n, env)
        same = ha == hc and (not with_python or ha == hp)
        f = lambda t: "-" if t is None else "%.3f" % t
        r = lambda t: "-" if t is None or a is None else "%.2f" % (a / t)
        print(f"| {name} | {size} | {n} | {f(a)}{'' if same else ' DIFF'} | {f(c)} | {f(py)} | {r(c)} | {r(py)} |", flush=True)
