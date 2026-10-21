# Python twin of mandelbrot.ail with multiprocessing: WORKERS processes (default: all cores).
import os, sys
from multiprocessing import Pool

CHARS = " .:-=+*#%@"
MAX_ITER = 500


def escape(cr, ci):
    zr = 0.0
    zi = 0.0
    k = 0
    while k < MAX_ITER:
        if zr * zr + zi * zi > 4.0:
            break
        t = zr * zr - zi * zi + cr
        zi = 2.0 * zr * zi + ci
        zr = t
        k += 1
    return k


def row(args):
    y, rows = args
    cols = rows * 2
    ci = -1.2 + 2.4 * y / rows
    out = []
    for x in range(cols):
        cr = -2.0 + 3.0 * x / cols
        out.append(CHARS[escape(cr, ci) * 9 // MAX_ITER])
    return "".join(out)


def main():
    rows = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
    workers = int(os.environ.get("WORKERS", os.cpu_count()))
    with Pool(workers) as pool:
        lines = pool.map(row, [(y, rows) for y in range(rows)], chunksize=max(1, rows // (workers * 8)))
    sys.stdout.write("\n".join(lines) + "\n")


main()
