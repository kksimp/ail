def lcs_table(a, b):
    g = [[0] * (len(b) + 1) for _ in range(len(a) + 1)]
    for i in range(len(a)):
        for j in range(len(b)):
            if a[i] == b[j]:
                g[i + 1][j + 1] = g[i][j] + 1
            else:
                g[i + 1][j + 1] = max(g[i][j + 1], g[i + 1][j])
    return g


def diff(a, b):
    g = lcs_table(a, b)
    rev = []
    i = len(a)
    j = len(b)
    while i > 0 or j > 0:
        if i > 0 and j > 0 and a[i - 1] == b[j - 1]:
            rev.append("  " + a[i - 1])
            i -= 1
            j -= 1
        elif j > 0 and (i == 0 or g[i][j - 1] >= g[i - 1][j]):
            rev.append("+ " + b[j - 1])
            j -= 1
        else:
            rev.append("- " + a[i - 1])
            i -= 1
    return rev[::-1]


def apply_edits(edits):
    return [e[2:] for e in edits if not e.startswith("- ")]


def count_prefix(edits, prefix):
    return sum(1 for e in edits if e.startswith(prefix))


def main():
    old = ["fn main()", "    x := 1", "    y := 2", "    print(x + y)", "", "fn helper()", "    todo"]
    new = ["fn main()", "    x := 1", "    z := 5", "    y := 2", "    print(x + y + z)", "", "fn helper()"]
    edits = diff(old, new)
    for e in edits:
        print(e)
    print("added %d, removed %d, kept %d" % (count_prefix(edits, "+ "), count_prefix(edits, "- "), count_prefix(edits, "  ")))
    print("patch reproduces new: %s" % ("true" if apply_edits(edits) == new else "false"))
    same = diff(old, old)
    print("identical inputs changed: %d" % (count_prefix(same, "+ ") + count_prefix(same, "- ")))
    print("from empty: %d lines" % len(diff([], new)))


main()
