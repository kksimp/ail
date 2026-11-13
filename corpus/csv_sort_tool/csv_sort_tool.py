import os, tempfile, shutil

DATA = """name,dept,salary,age
ann,eng,120,34
bob,ops,95,45
cy,eng,150,29
dee,ops,95,38
eve,hr,70,51
fay,eng,120,41
"""


def num(s):
    try:
        return int(s)
    except ValueError:
        return 0


def parse(text):
    return [line.split(",") for line in text.splitlines()]


def sort_rows(rows, col, numeric, desc):
    if numeric:
        sign = -1 if desc else 1
        return sorted(rows, key=lambda r: sign * num(r[col]))
    out = sorted(rows, key=lambda r: r[col])
    if desc:
        return out[::-1]
    return out


def column(header, name):
    return header.index(name) if name in header else 0


def render(header, rows):
    lines = [",".join(header)]
    for r in rows:
        lines.append(",".join(r))
    return "\n".join(lines)


def show(title, rows):
    print(f"{title}: {' '.join(r[0] for r in rows)}")


def main():
    table = parse(DATA)
    header = table[0]
    body = table[1:]
    show("by name", sort_rows(body, column(header, "name"), False, False))
    show("by age desc", sort_rows(body, column(header, "age"), True, True))
    show("by salary", sort_rows(body, column(header, "salary"), True, False))
    by_name = sort_rows(body, column(header, "name"), False, False)
    two_key = sort_rows(sort_rows(by_name, column(header, "salary"), True, True), column(header, "dept"), False, False)
    show("dept, then salary desc", two_key)
    d = tempfile.mkdtemp(prefix="corpus_csv_sort_tool")
    out = os.path.join(d, "sorted.csv")
    with open(out, "w", newline="") as f:
        f.write(render(header, two_key))
    with open(out, newline="") as f:
        back = f.read()
    print(f"wrote {len(back.splitlines())} lines, {len(back.encode())} bytes")
    print(f"first row: {back.splitlines()[1]}")
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
