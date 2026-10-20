# Python twin of binarytrees.ail (CPython 3, one thread).
import sys


def make(left, right, depth):
    node = len(left)
    left.append(-1)
    right.append(-1)
    if depth > 0:
        l = make(left, right, depth - 1)
        r = make(left, right, depth - 1)
        left[node] = l
        right[node] = r
    return node


def check(left, right, node):
    if left[node] < 0:
        return 1
    return 1 + check(left, right, left[node]) + check(left, right, right[node])


def main():
    max_depth = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    min_depth = 4
    d = min_depth
    while d <= max_depth:
        iters = 2 ** (max_depth - d + min_depth)
        total = 0
        for _ in range(iters):
            left = []
            right = []
            root = make(left, right, d)
            total += check(left, right, root)
        print(f"{iters} trees of depth {d} check: {total}")
        d += 2


main()
