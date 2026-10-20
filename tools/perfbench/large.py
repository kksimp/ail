"""Large-size race of `ail build` output against C and Python (docs/design/native-performance.md).
Usage: python3 -I large.py PROGRAMS_DIR EXE_DIR, with each NAME built into EXE_DIR/NAME. Measurement only (ADR 0070)."""
import subprocess, time, sys, os
P = sys.argv[1]; T = sys.argv[2]
sizes = {"collatz": 1000000, "nbody": 1000000, "spectralnorm": 1000, "mandelbrot": 1000,
         "fannkuch": 10, "sieve": 30000000, "matmul": 400, "binarytrees": 17, "fasta": 5000000}
def best(cmd, n=3):
    ts = []; out = None
    for _ in range(n):
        t = time.perf_counter(); p = subprocess.run(cmd, stdout=subprocess.PIPE); ts.append(time.perf_counter() - t); out = p.stdout
    return min(ts), out
print("| program | size | native | clang -O2 | gcc -O2 | python | native/clang | native/gcc | native/python |")
print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
for name, n in sizes.items():
    a = [str(n)]
    nt, no = best([f"{T}/{name}"] + a)
    row = [nt]
    for cc in ("clang", "gcc"):
        exe = f"{T}/{cc}-{name}"
        subprocess.run([cc, "-O2", "-ffp-contract=off", "-o", exe, f"{P}/{name}.c", "-lm"], check=True)
        t, o = best([exe] + a)
        row.append(t if o == no else None)
    pt, po = best(["python3", "-I", f"{P}/{name}.py"] + a, 1)
    row.append(pt if po == no else None)
    f = lambda x: "DIFF" if x is None else "%.3f" % x
    r = lambda x: "-" if x is None else "%.2f" % (nt / x)
    print(f"| {name} | {n} | {f(row[0])} | {f(row[1])} | {f(row[2])} | {f(row[3])} | {r(row[1])} | {r(row[2])} | {r(row[3])} |", flush=True)
