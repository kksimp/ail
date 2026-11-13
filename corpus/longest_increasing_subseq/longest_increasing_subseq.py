def lis(xs):
    tails = []
    parent = [-1] * len(xs)
    for i in range(len(xs)):
        lo, hi = 0, len(tails)
        while lo < hi:
            mid = lo + (hi - lo) // 2
            if xs[tails[mid]] < xs[i]:
                lo = mid + 1
            else:
                hi = mid
        if lo > 0:
            parent[i] = tails[lo - 1]
        if lo == len(tails):
            tails.append(i)
        else:
            tails[lo] = i
    cur = tails[-1] if tails else -1
    out = []
    while cur != -1:
        out.append(xs[cur])
        cur = parent[cur]
    out.reverse()
    return out


def make_data(n, seed):
    s = seed
    out = []
    for _ in range(n):
        s = (s * 1103515245 + 12345) % 2147483648
        out.append(s // 65536 % 100)
    return out


def main():
    inputs = [
        [10, 9, 2, 5, 3, 7, 101, 18],
        [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15],
        [5, 4, 3, 2, 1],
        [7, 7, 7, 7],
        [1, 2, 3, 4, 5],
        [],
        make_data(20, 99),
    ]
    for xs in inputs:
        seq = lis(xs)
        print(f"{' '.join(str(x) for x in xs)} => length {len(seq)}: {' '.join(str(x) for x in seq)}")


main()
