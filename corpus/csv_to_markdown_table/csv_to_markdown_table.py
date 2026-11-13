def split_csv(line):
    cells = []
    cur = ""
    quoted = False
    for c in line:
        if c == '"':
            quoted = not quoted
        elif c == "," and not quoted:
            cells.append(cur)
            cur = ""
        else:
            cur += c
    cells.append(cur)
    return cells


def is_number(s):
    if s == "":
        return False
    for c in s:
        if c not in "0123456789":
            return False
    return True


def cell_at(row, j):
    if j < len(row):
        return row[j].strip().replace("|", "/")
    return ""


def to_markdown(text):
    rows = []
    for line in text.split("\n"):
        if line.strip() != "":
            rows.append(split_csv(line))
    if not rows:
        return ""
    ncols = len(rows[0])
    widths = []
    numeric = []
    for j in range(ncols):
        w = 3
        num = len(rows) > 1
        for i in range(len(rows)):
            cell = cell_at(rows[i], j)
            w = max(w, len(cell))
            if i > 0 and not is_number(cell):
                num = False
        widths.append(w)
        numeric.append(num)
    out = []
    for i in range(len(rows)):
        cells = []
        for j in range(ncols):
            cell = cell_at(rows[i], j)
            if numeric[j]:
                cells.append(cell.rjust(widths[j]))
            else:
                cells.append(cell.ljust(widths[j]))
        out.append("| " + " | ".join(cells) + " |")
        if i == 0:
            marks = []
            for j in range(ncols):
                if numeric[j]:
                    marks.append(":".rjust(widths[j], "-"))
                else:
                    marks.append("".rjust(widths[j], "-"))
            out.append("| " + " | ".join(marks) + " |")
    return "\n".join(out)


data = [
    "name,qty,note",
    'apple,12,"red, sweet"',
    "pear,7,green",
    "fig,150,a|b",
    "kiwi,3",
]
print(to_markdown("\n".join(data)))
print("")
print(to_markdown("id,score\n1,90\n22,8\n333,100"))
print(to_markdown("only,header"))
