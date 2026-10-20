# Python twin of fasta.ail (CPython 3, one thread).
import sys

WIDTH = 60


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100000
    letters = "acgt"
    seed = 42
    counts = [0, 0, 0, 0]
    line = []
    for _ in range(n):
        seed = (seed * 3877 + 29573) % 139968
        t = seed * 100 // 139968
        base = 3
        if t < 30:
            base = 0
        elif t < 50:
            base = 1
        elif t < 70:
            base = 2
        counts[base] += 1
        line.append(letters[base])
        if len(line) == WIDTH:
            print("".join(line))
            line = []
    if len(line) > 0:
        print("".join(line))
    print(f"a {counts[0]} c {counts[1]} g {counts[2]} t {counts[3]}")


main()
