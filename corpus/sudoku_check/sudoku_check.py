def parse_board(rows):
    cells = []
    for line in rows:
        for ch in line:
            cells.append(int(ch) if ch.isdigit() else 0)
    return cells


def row_cells(r):
    return [r * 9 + k for k in range(9)]


def col_cells(c):
    return [k * 9 + c for k in range(9)]


def box_cells(b):
    return [(b // 3 * 3 + k // 3) * 9 + (b % 3 * 3 + k % 3) for k in range(9)]


def first_repeat(cells, idxs):
    seen = set()
    for i in idxs:
        d = cells[i]
        if d != 0 and d in seen:
            return d
        if d != 0:
            seen.add(d)
    return 0


def empties(cells):
    return sum(1 for v in cells if v == 0)


def problems(cells):
    out = []
    for r in range(9):
        a = first_repeat(cells, row_cells(r))
        if a != 0:
            out.append(f"row {r + 1} repeats {a}")
    for c in range(9):
        b = first_repeat(cells, col_cells(c))
        if b != 0:
            out.append(f"column {c + 1} repeats {b}")
    for x in range(9):
        e = first_repeat(cells, box_cells(x))
        if e != 0:
            out.append(f"box {x + 1} repeats {e}")
    return out


def verdict(cells):
    found = problems(cells)
    if found:
        return "invalid"
    if empties(cells) > 0:
        return f"valid so far, {empties(cells)} cells empty"
    return "valid and complete"


def report(title, rows):
    cells = parse_board(rows)
    print(f"{title}: {verdict(cells)}")
    for p in problems(cells):
        print(f"  {p}")


def main():
    solved = [
        "534678912",
        "672195348",
        "198342567",
        "859761423",
        "426853791",
        "713924856",
        "961537284",
        "287419635",
        "345286179",
    ]
    partial = [
        "53..7....",
        "6..195...",
        ".98....6.",
        "8...6...3",
        "4..8.3..1",
        "7...2...6",
        ".6....28.",
        "...419..5",
        "....8..79",
    ]
    broken = list(solved)
    broken[0] = "535678912"
    report("solved", solved)
    report("partial", partial)
    report("broken", broken)


main()
