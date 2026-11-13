def parse_rows(lines):
    rows = []
    for raw in lines:
        t = raw.strip()
        if t == "":
            continue
        cells = [part.strip() for part in t.split("|")]
        if cells[0] == "" and len(cells) > 1:
            cells = cells[1:]
        if cells[-1] == "" and len(cells) > 1:
            cells = cells[:-1]
        rows.append(cells)
    return rows


def is_int(s):
    if s == "":
        return False
    start = 1 if s[0] == "-" else 0
    if start >= len(s):
        return False
    for ch in s[start:]:
        if not ch.isdigit():
            return False
    return True


def cell(row, col):
    if col < len(row):
        return row[col]
    return ""


def align(lines):
    rows = parse_rows(lines)
    if not rows:
        return []
    ncols = 0
    for r in rows:
        ncols = max(ncols, len(r))
    widths = []
    numeric = []
    for col in range(ncols):
        w = 3
        all_int = len(rows) > 1
        for ri in range(len(rows)):
            text = cell(rows[ri], col)
            w = max(w, len(text))
            if ri > 0 and text != "" and not is_int(text):
                all_int = False
        widths.append(w)
        numeric.append(all_int)
    out = []
    for i in range(len(rows)):
        parts = []
        for c in range(ncols):
            cv = cell(rows[i], c)
            if numeric[c] and i > 0:
                parts.append(cv.rjust(widths[c]))
            else:
                parts.append(cv.ljust(widths[c]))
        out.append("| " + " | ".join(parts) + " |")
        if i == 0:
            seps = []
            for d in range(ncols):
                dashes = "-" * (widths[d] - 1)
                seps.append(f"{dashes}:" if numeric[d] else f":{dashes}")
            out.append("| " + " | ".join(seps) + " |")
    return out


def main():
    raw = [
        "item | qty | price | note",
        "| apple | 3 | 120 | fresh |",
        "banana | 12 | 45",
        "",
        "dragon fruit | 1 | -7 | imported, rare |",
        "kiwi | 30 | 8 | ",
    ]
    for line in align(raw):
        print(line)
    print("")
    for line in align(["name | role", "ann | lead", "bo | dev"]):
        print(line)


main()
