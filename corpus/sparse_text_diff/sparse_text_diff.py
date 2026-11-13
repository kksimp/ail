def edit_script(a, b):
    g = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    for r in range(len(a)):
        for c in range(len(b)):
            if a[r] == b[c]:
                g[r + 1][c + 1] = g[r][c] + 1
            else:
                g[r + 1][c + 1] = max(g[r][c + 1], g[r + 1][c])
    rev = []
    i = len(a)
    j = len(b)
    while i > 0 or j > 0:
        if i > 0 and j > 0 and a[i - 1] == b[j - 1]:
            rev.append((" ", a[i - 1]))
            i -= 1
            j -= 1
        elif j > 0 and (i == 0 or g[i][j - 1] >= g[i - 1][j]):
            rev.append(("+", b[j - 1]))
            j -= 1
        else:
            rev.append(("-", a[i - 1]))
            i -= 1
    return rev[::-1]


def keep_flags(edits, ctx):
    n = len(edits)
    keep = [False] * n
    for i in range(n):
        if edits[i][0] != " ":
            lo = max(i - ctx, 0)
            hi = min(i + ctx, n - 1)
            for j in range(lo, hi + 1):
                keep[j] = True
    return keep


def hunks(edits, ctx):
    keep = keep_flags(edits, ctx)
    out = []
    old_seen = 0
    new_seen = 0
    i = 0
    while i < len(edits):
        if not keep[i]:
            old_seen += 1
            new_seen += 1
            i += 1
            continue
        o0 = old_seen
        n0 = new_seen
        body = []
        while i < len(edits) and keep[i]:
            kind, text = edits[i]
            body.append(f"{kind}{text}")
            if kind != "+":
                old_seen += 1
            if kind != "-":
                new_seen += 1
            i += 1
        out.append(f"@@ -{o0 + 1},{old_seen - o0} +{n0 + 1},{new_seen - n0} @@")
        out.extend(body)
    return out


def show(label, a, b, ctx):
    print(f"{label} (context {ctx}):")
    out = hunks(edit_script(a, b), ctx)
    if not out:
        print("  no differences")
    for line in out:
        print(f"  {line}")


def main():
    before = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta",
        "eta", "theta", "iota", "kappa", "lambda", "mu",
    ]
    after = list(before)
    after[1] = "BETA"
    del after[7]
    after.insert(10, "nu")
    show("two changes", before, after, 1)
    show("two changes", before, after, 0)
    show("wide", before, after, 3)
    show("same", before, before, 2)
    show("from empty", [], ["one", "two"], 1)


main()
