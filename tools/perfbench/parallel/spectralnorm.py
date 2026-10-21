# Python twin of spectralnorm.ail with multiprocessing: WORKERS processes (default: all cores).
import math, os, sys
from multiprocessing import Pool

V = None


def eval_a(i, j):
    d = (i + j) * (i + j + 1) // 2 + i + 1
    return 1.0 / d


def av_rows(span):
    lo, hi, n, v = span
    out = []
    for i in range(lo, hi):
        s = 0.0
        for j in range(n):
            s += eval_a(i, j) * v[j]
        out.append(s)
    return out


def atv_rows(span):
    lo, hi, n, v = span
    out = []
    for i in range(lo, hi):
        s = 0.0
        for j in range(n):
            s += eval_a(j, i) * v[j]
        out.append(s)
    return out


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1500
    workers = int(os.environ.get("WORKERS", os.cpu_count()))
    size = max(1, n // (workers * 4))
    spans = [(lo, min(lo + size, n)) for lo in range(0, n, size)]
    with Pool(workers) as pool:
        def mul(fn, v):
            parts = pool.map(fn, [(lo, hi, n, v) for lo, hi in spans])
            return [x for part in parts for x in part]

        def mul_ata(v):
            return mul(atv_rows, mul(av_rows, v))

        u = [1.0] * n
        v = [0.0] * n
        for _ in range(10):
            v = mul_ata(u)
            u = mul_ata(v)
    vbv = 0.0
    vv = 0.0
    for i in range(n):
        vbv += u[i] * v[i]
        vv += v[i] * v[i]
    print("%.9f" % math.sqrt(vbv / vv))


main()
