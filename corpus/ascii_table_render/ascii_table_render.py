def to_base(n, base, width):
    digits = "0123456789abcdef"
    out = ""
    v = n
    while v > 0:
        out = digits[v % base] + out
        v = v // base
    return out.rjust(width, "0")


def widths(headers, rows):
    w = [len(h) for h in headers]
    for row in rows:
        for i in range(len(row)):
            w[i] = max(w[i], len(row[i]))
    return w


def rule(w):
    out = "+"
    for n in w:
        out = out + "-".ljust(n + 2, "-") + "+"
    return out


def row_line(cells, w, right):
    out = "|"
    for i in range(len(cells)):
        cell = cells[i].rjust(w[i]) if right[i] else cells[i].ljust(w[i])
        out = out + " " + cell + " |"
    return out


def render(headers, rows, right):
    w = widths(headers, rows)
    lines = [rule(w), row_line(headers, w, [False for h in headers]), rule(w)]
    for row in rows:
        lines.append(row_line(row, w, right))
    lines.append(rule(w))
    return lines


def glyph(n, names):
    if n < 32:
        return names[n]
    if n == 32:
        return "SP"
    if n == 127:
        return "DEL"
    return chr(n)


def classify(n):
    if n < 32 or n == 127:
        return "control"
    if n == 32:
        return "space"
    if 48 <= n <= 57:
        return "digit"
    if 65 <= n <= 90:
        return "upper"
    if 97 <= n <= 122:
        return "lower"
    return "punct"


def main():
    names = "NUL SOH STX ETX EOT ENQ ACK BEL BS HT LF VT FF CR SO SI DLE DC1 DC2 DC3 DC4 NAK SYN ETB CAN EM SUB ESC FS GS RS US".split(" ")
    rows = []
    for n in [0, 7, 9, 10, 13, 27, 32, 33, 48, 57, 65, 90, 97, 122, 126, 127]:
        rows.append(["%d" % n, "0x" + to_base(n, 16, 2), to_base(n, 8, 3), to_base(n, 2, 8), glyph(n, names), classify(n)])
    for line in render(["Dec", "Hex", "Oct", "Bin", "Char", "Class"], rows, [True, True, True, True, False, False]):
        print(line)
    print("")
    for line in render(["Item", "Qty"], [["apple", "3"], ["watermelon", "12"]], [False, True]):
        print(line)


main()
