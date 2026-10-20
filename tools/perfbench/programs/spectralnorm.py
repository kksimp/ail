# Python twin of spectralnorm.ail (CPython 3, one thread).
import math
import sys


def eval_a(i, j):
    d = (i + j) * (i + j + 1) // 2 + i + 1
    return 1.0 / float(d)


def mul_av(v, n):
    out = []
    for i in range(n):
        total = 0.0
        for j in range(n):
            total += eval_a(i, j) * v[j]
        out.append(total)
    return out


def mul_atv(v, n):
    out = []
    for i in range(n):
        total = 0.0
        for j in range(n):
            total += eval_a(j, i) * v[j]
        out.append(total)
    return out


def mul_ata(v, n):
    return mul_atv(mul_av(v, n), n)


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    u = [1.0] * n
    v = [0.0] * n
    for _ in range(10):
        v = mul_ata(u, n)
        u = mul_ata(v, n)
    vbv = 0.0
    vv = 0.0
    for i in range(n):
        vbv += u[i] * v[i]
        vv += v[i] * v[i]
    print(f"{math.sqrt(vbv / vv):.9f}")


main()
