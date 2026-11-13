def data():
    return [
        "name,dept,salary",
        "ada,eng,120",
        "bob,ops,80",
        "cy,eng,100",
        "dee,sales,70",
        "eve,ops,90",
        "oops,eng",
        "fay,sales,abc",
        "gus,eng,110",
    ]


def parse_row(line):
    cells = line.split(",")
    if len(cells) != 3:
        return None
    try:
        return (cells[0], cells[1], int(cells[2]))
    except ValueError:
        return None


def median(xs):
    s = sorted(xs)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    return (s[n // 2 - 1] + s[n // 2]) // 2


def main():
    rows = []
    bad = 0
    for line in data()[1:]:
        row = parse_row(line)
        if row is None:
            bad += 1
            continue
        rows.append(row)
    salaries = [r[2] for r in rows]
    total = sum(salaries)
    print(f"rows: {len(rows)}, skipped: {bad}")
    print(f"min {min(salaries)}, max {max(salaries)}")
    print(f"mean {total // len(rows)}, median {median(salaries)}")
    by_dept = {}
    for r in rows:
        by_dept[r[1]] = by_dept.get(r[1], 0) + r[2]
    for d in sorted(by_dept):
        print(f"{d:<6}{by_dept[d]:>5}")


main()
