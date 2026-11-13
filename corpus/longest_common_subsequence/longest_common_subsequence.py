def lcs_table(a, b):
    width = len(b) + 1
    table = [0] * ((len(a) + 1) * width)
    for i in range(1, len(a) + 1):
        for j in range(1, len(b) + 1):
            if a[i - 1] == b[j - 1]:
                table[i * width + j] = table[(i - 1) * width + j - 1] + 1
            else:
                table[i * width + j] = max(table[(i - 1) * width + j], table[i * width + j - 1])
    return table


def lcs(x, y):
    a = list(x)
    b = list(y)
    width = len(b) + 1
    table = lcs_table(a, b)
    out = []
    i = len(a)
    j = len(b)
    while i > 0 and j > 0:
        if a[i - 1] == b[j - 1]:
            out.append(a[i - 1])
            i -= 1
            j -= 1
        elif table[(i - 1) * width + j] >= table[i * width + j - 1]:
            i -= 1
        else:
            j -= 1
    out.reverse()
    return "".join(out)


def similarity(x, y):
    total = len(x) + len(y)
    if total == 0:
        return 100
    return 2 * len(lcs(x, y)) * 100 // total


def main():
    pairs = [
        ["ABCBDAB", "BDCABA"],
        ["AGGTAB", "GXTXAYB"],
        ["kitten", "sitting"],
        ["dynamic programming", "dramatic programs"],
        ["same", "same"],
        ["abc", "xyz"],
        ["", "abc"],
    ]
    for p in pairs:
        common = lcs(p[0], p[1])
        print(f"'{p[0]}' vs '{p[1]}': '{common}' length {len(common)} similarity {similarity(p[0], p[1])}%")


main()
