# Python twin of nbody.ail with multiprocessing: WORKERS processes (default: all cores).
import math, os, sys
from multiprocessing import Pool


def accel(span):
    lo, hi, px, py, pz, mass = span
    n = len(px)
    out = []
    for i in range(lo, hi):
        sx = sy = sz = 0.0
        for j in range(n):
            if j != i:
                dx = px[j] - px[i]
                dy = py[j] - py[i]
                dz = pz[j] - pz[i]
                d2 = dx * dx + dy * dy + dz * dz + 0.01
                inv = mass[j] / (d2 * math.sqrt(d2))
                sx += dx * inv
                sy += dy * inv
                sz += dz * inv
        out.append((sx, sy, sz))
    return out


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1500
    workers = int(os.environ.get("WORKERS", os.cpu_count()))
    px = [0.0] * n
    py = [0.0] * n
    pz = [0.0] * n
    vx = [0.0] * n
    vy = [0.0] * n
    vz = [0.0] * n
    mass = [0.0] * n
    for i in range(n):
        t = i * 0.37
        px[i] = 100.0 * (i % 17) + t
        py[i] = 90.0 * (i % 13) - t
        pz[i] = 80.0 * (i % 11) + 0.5 * t
        mass[i] = 1.0 + (i % 5)
    size = max(1, n // (workers * 4))
    spans = [(lo, min(lo + size, n)) for lo in range(0, n, size)]
    with Pool(workers) as pool:
        for _ in range(10):
            parts = pool.map(accel, [(lo, hi, px, py, pz, mass) for lo, hi in spans])
            acc = [a for part in parts for a in part]
            for i in range(n):
                ax, ay, az = acc[i]
                vx[i] = vx[i] + 0.01 * ax
                vy[i] = vy[i] + 0.01 * ay
                vz[i] = vz[i] + 0.01 * az
                px[i] = px[i] + 0.01 * vx[i]
                py[i] = py[i] + 0.01 * vy[i]
                pz[i] = pz[i] + 0.01 * vz[i]
    total = 0.0
    for i in range(n):
        total += px[i] + py[i] + pz[i]
    print("%.6f" % total)


main()
