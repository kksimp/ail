def levenshtein(a, b):
    xs, ys = a, b
    g = [[0] * (len(ys) + 1) for _ in range(len(xs) + 1)]
    for i in range(len(xs) + 1):
        g[i][0] = i
    for j in range(len(ys) + 1):
        g[0][j] = j
    for i in range(1, len(xs) + 1):
        for j in range(1, len(ys) + 1):
            cost = 0 if xs[i - 1] == ys[j - 1] else 1
            g[i][j] = min(g[i - 1][j] + 1, g[i][j - 1] + 1, g[i - 1][j - 1] + cost)
    return g[len(xs)][len(ys)]


def osa(a, b):
    xs, ys = a, b
    g = [[0] * (len(ys) + 1) for _ in range(len(xs) + 1)]
    for i in range(len(xs) + 1):
        g[i][0] = i
    for j in range(len(ys) + 1):
        g[0][j] = j
    for i in range(1, len(xs) + 1):
        for j in range(1, len(ys) + 1):
            cost = 0 if xs[i - 1] == ys[j - 1] else 1
            best = min(g[i - 1][j] + 1, g[i][j - 1] + 1, g[i - 1][j - 1] + cost)
            if i > 1 and j > 1 and xs[i - 1] == ys[j - 2] and xs[i - 2] == ys[j - 1]:
                best = min(best, g[i - 2][j - 2] + 1)
            g[i][j] = best
    return g[len(xs)][len(ys)]


def hamming(a, b):
    if len(a) != len(b):
        return -1
    return sum(1 for x, y in zip(a, b) if x != y)


def lcs_len(a, b):
    g = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            if a[i - 1] == b[j - 1]:
                g[i][j] = g[i - 1][j - 1] + 1
            else:
                g[i][j] = max(g[i - 1][j], g[i][j - 1])
    return g[len(a)][len(b)]


def common_substring(a, b):
    g = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    best = 0
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            if a[i - 1] == b[j - 1]:
                g[i][j] = g[i - 1][j - 1] + 1
                best = max(best, g[i][j])
    return best


def common_prefix(a, b):
    n = 0
    while n < len(a) and n < len(b) and a[n] == b[n]:
        n += 1
    return n


def main():
    pairs = [
        ["kitten", "sitting"],
        ["ab", "ba"],
        ["abcdef", "abdcef"],
        ["karolin", "kathrin"],
        ["flaw", "lawn"],
        ["interview", "internet"],
        ["", "abc"],
        ["same", "same"],
    ]
    print("a          b          lev osa ham lcs sub pre")
    for p in pairs:
        a = p[0]
        b = p[1]
        print(f"{a:<10} {b:<10} {levenshtein(a, b):>3} {osa(a, b):>3} {hamming(a, b):>3} {lcs_len(a, b):>3} {common_substring(a, b):>3} {common_prefix(a, b):>3}")


main()
