def chunks(s):
    out = []
    cur = ""
    digits = False
    for c in s:
        d = c in "0123456789"
        if cur != "" and d != digits:
            out.append(cur)
            cur = ""
        digits = d
        cur += c
    if cur != "":
        out.append(cur)
    return out


def is_num(s):
    return s != "" and s[0] in "0123456789"


def cmp_chunk(a, b):
    if is_num(a) and is_num(b):
        x = int(a)
        y = int(b)
        if x != y:
            return -1 if x < y else 1
        return 0
    if is_num(a):
        return -1
    if is_num(b):
        return 1
    if a < b:
        return -1
    if a > b:
        return 1
    return 0


def natural_cmp(a, b):
    xs = chunks(a)
    ys = chunks(b)
    n = min(len(xs), len(ys))
    for i in range(n):
        c = cmp_chunk(xs[i], ys[i])
        if c != 0:
            return c
    if len(xs) == len(ys):
        return 0
    if len(xs) < len(ys):
        return -1
    return 1


def natural_sort(items):
    vs = list(items)
    for i in range(1, len(vs)):
        j = i
        while j > 0 and natural_cmp(vs[j - 1], vs[j]) > 0:
            vs[j - 1], vs[j] = vs[j], vs[j - 1]
            j -= 1
    return vs


def product(name):
    return chunks(name)[0]


names = [
    "app-1.10.2", "app-1.2.10", "app-1.2.9", "lib-0.9", "app-1.2", "lib-0.10",
    "app-10.0", "app-2.0", "lib-0.9.1", "tool-v3", "tool-v12", "tool-v4",
]
ordered = natural_sort(names)
print(f"natural: {' '.join(ordered)}")
print(f"plain:   {' '.join(sorted(names))}")
newest = {}
for n in ordered:
    newest[product(n)] = n
for p in sorted(newest):
    print(f"newest {p}: {newest[p]}")
print(f"1.9 vs 1.10: {natural_cmp('1.9', '1.10')}")
