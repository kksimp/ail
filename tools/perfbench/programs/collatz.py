# Python twin of collatz.ail (CPython 3, one thread).
import sys


def steps(start):
    n = start
    k = 0
    while n != 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        k += 1
    return k


def main():
    limit = int(sys.argv[1]) if len(sys.argv) > 1 else 30000
    total = 0
    best = 1
    best_steps = 0
    for i in range(1, limit):
        s = steps(i)
        total += s
        if s > best_steps:
            best = i
            best_steps = s
    print(f"total steps: {total}")
    print(f"longest: {best} with {best_steps} steps")


main()
