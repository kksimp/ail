# Python twin of parsum.ail with multiprocessing: WORKERS processes (default: all cores).
import os, sys
from multiprocessing import Pool


def steps(n):
    x = n
    k = 0
    while x != 1:
        x = x // 2 if x % 2 == 0 else 3 * x + 1
        k += 1
    return k


def block(span):
    lo, hi = span
    return [steps(i + 1) for i in range(lo, hi)]


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 3000000
    workers = int(os.environ.get("WORKERS", os.cpu_count()))
    size = max(1, n // (workers * 8))
    spans = [(lo, min(lo + size, n)) for lo in range(0, n, size)]
    with Pool(workers) as pool:
        lengths = [s for part in pool.map(block, spans) for s in part]
    total = sum(lengths)
    longest = 0
    for i in range(n):
        if lengths[i] > lengths[longest]:
            longest = i
    print(f"total steps: {total}")
    print(f"longest: {longest + 1} with {lengths[longest]} steps")


main()
