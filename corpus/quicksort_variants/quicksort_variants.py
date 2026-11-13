def lomuto(xs, lo, hi):
    if lo >= hi:
        return 0
    pivot = xs[hi]
    store = lo
    seen = 0
    for j in range(lo, hi):
        seen += 1
        if xs[j] < pivot:
            xs[store], xs[j] = xs[j], xs[store]
            store += 1
    xs[store], xs[hi] = xs[hi], xs[store]
    seen += lomuto(xs, lo, store - 1)
    seen += lomuto(xs, store + 1, hi)
    return seen


def three_way(xs, lo, hi):
    if lo >= hi:
        return 0
    pivot = xs[lo]
    lt = lo
    gt = hi
    i = lo + 1
    seen = 0
    while i <= gt:
        seen += 1
        if xs[i] < pivot:
            xs[lt], xs[i] = xs[i], xs[lt]
            lt += 1
            i += 1
        elif xs[i] > pivot:
            xs[i], xs[gt] = xs[gt], xs[i]
            gt -= 1
        else:
            i += 1
    seen += three_way(xs, lo, lt - 1)
    seen += three_way(xs, gt + 1, hi)
    return seen


def functional(xs):
    if len(xs) <= 1:
        return xs
    pivot = xs[len(xs) // 2]
    lows = [x for x in xs if x < pivot]
    same = [x for x in xs if x == pivot]
    highs = [x for x in xs if x > pivot]
    return functional(lows) + same + functional(highs)


def make_data(n, seed, modulus):
    s = seed
    out = []
    for _ in range(n):
        s = (s * 1103515245 + 12345) % 2147483648
        out.append(s // 65536 % modulus)
    return out


def main():
    ascending = list(range(24))
    descending = [24 - i for i in range(24)]
    equal = [5] * 16
    names = ["random", "ascending", "descending", "all equal", "three values"]
    sets = [make_data(24, 7, 50), ascending, descending, equal, make_data(24, 3, 3)]
    for i in range(len(names)):
        data = sets[i]
        want = sorted(data)
        a = list(data)
        b = list(data)
        ca = lomuto(a, 0, len(a) - 1)
        cb = three_way(b, 0, len(b) - 1)
        c = functional(data)
        ok = "yes" if a == want and b == want and c == want else "no"
        print(f"{names[i]} (n={len(data)}): lomuto={ca} three-way={cb} agree={ok}")
    print("first set sorted: " + " ".join(str(x) for x in sorted(sets[0])))


main()
