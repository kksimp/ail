def split_row(line):
    inner = line.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|"):
        inner = inner[:-1]
    return [c.strip() for c in inner.split("|")]


def is_separator(cells):
    if not cells:
        return False
    for c in cells:
        if "-" not in c:
            return False
        for ch in c:
            if ch != "-" and ch != ":":
                return False
    return True


def alignment(cell):
    if cell.startswith(":") and cell.endswith(":"):
        return "center"
    if cell.endswith(":"):
        return "right"
    return "left"


def parse_table(block):
    if len(block) < 2:
        return None
    sep = split_row(block[1])
    if not is_separator(sep):
        return None
    header = split_row(block[0])
    rows = []
    for i in range(2, len(block)):
        cells = split_row(block[i])
        while len(cells) < len(header):
            cells.append("")
        rows.append(cells[:len(header)])
    aligns = []
    for i in range(len(header)):
        if i < len(sep):
            aligns.append(alignment(sep[i]))
        else:
            aligns.append("left")
    return (header, aligns, rows)


def pad_cell(text, width, how):
    if how == "right":
        return text.rjust(width)
    if how == "center":
        left = (width - len(text)) // 2
        return text.rjust(len(text) + left).ljust(width)
    return text.ljust(width)


def bar(n):
    return "-" * n


def render(t):
    header, aligns, rows = t
    widths = [max(len(h), 3) for h in header]
    for r in rows:
        for i in range(len(r)):
            widths[i] = max(widths[i], len(r[i]))
    out = []
    head = []
    rule = []
    for i in range(len(header)):
        head.append(pad_cell(header[i], widths[i], aligns[i]))
        rule.append(bar(widths[i]))
    out.append("| " + " | ".join(head) + " |")
    out.append("| " + " | ".join(rule) + " |")
    for r in rows:
        cells = []
        for i in range(len(r)):
            cells.append(pad_cell(r[i], widths[i], aligns[i]))
        out.append("| " + " | ".join(cells) + " |")
    return out


def to_int(s):
    t = s[1:] if s.startswith("-") else s
    if t.isdigit() and t.isascii():
        return int(s)
    return None


def column_sum(t, col):
    total = 0
    for r in t[2]:
        n = to_int(r[col])
        if n is None:
            return None
        total += n
    return total


def main():
    doc = [
        "# Report",
        "Some prose before the first table.",
        "| item | qty | note |",
        "|:--|--:|:-:|",
        "| apple | 12 | fresh |",
        "| pear | 7 |",
        "| fig | 130 | dried, sweet | extra |",
        "",
        "Text between tables.",
        "|name|score|",
        "|---|---|",
        "|ann|90|",
        "|bob|x|",
        "",
        "| not | a table |",
        "just a line",
    ]
    block = []
    count = 0
    for line in doc:
        if line.strip().startswith("|"):
            block.append(line)
            continue
        if not block:
            continue
        count += 1
        t = parse_table(block)
        if t is None:
            print(f"block {count}: not a table ({len(block)} lines)")
            block = []
            continue
        print(f"table {count}: {len(t[0])} columns, {len(t[2])} rows")
        for l in render(t):
            print(f"  {l}")
        for i in range(len(t[0])):
            s = column_sum(t, i)
            if s is None:
                continue
            print(f"  sum of {t[0][i]}: {s}")
        block = []


main()
