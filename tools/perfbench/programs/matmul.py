# Python twin of matmul.ail (CPython 3, one thread).
import sys


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    a = []
    b = []
    for i in range(n):
        a.append([(i * j + i + j) % 7 for j in range(n)])
        b.append([(i + 2 * j) % 5 for j in range(n)])
    c = []
    for i in range(n):
        row = []
        for j in range(n):
            s = 0
            for k in range(n):
                s += a[i][k] * b[k][j]
            row.append(s)
        c.append(row)
    trace = 0
    total = 0
    for i in range(n):
        trace += c[i][i]
        for j in range(n):
            total += c[i][j]
    print(f"trace: {trace}")
    print(f"sum: {total}")


main()
