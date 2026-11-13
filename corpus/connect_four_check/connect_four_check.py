COLS = 7
ROWS = 6


def parse(rows):
    cells = []
    for line in rows:
        cells.extend(list(line))
    return cells


def find_win(cells):
    steps = [[0, 1], [1, 0], [1, 1], [1, -1]]
    for r in range(ROWS):
        for c in range(COLS):
            p = cells[r * COLS + c]
            if p == '.':
                continue
            for s in steps:
                run = 1
                for k in range(1, 4):
                    nr = r + s[0] * k
                    nc = c + s[1] * k
                    if 0 <= nr < ROWS and 0 <= nc < COLS:
                        if cells[nr * COLS + nc] == p:
                            run += 1
                if run == 4:
                    return f"{p} from row {r} col {c} step {s[0]},{s[1]}"
    return "none"


def drop(cells, col, piece):
    r = ROWS - 1
    while r >= 0:
        if cells[r * COLS + col] == '.':
            cells[r * COLS + col] = piece
            return r
        r -= 1
    return -1


def show(cells):
    for r in range(ROWS):
        print("  " + "".join(cells[r * COLS:(r + 1) * COLS]))


def replay(columns):
    cells = parse([".......", ".......", ".......", ".......", ".......", "......."])
    turn = 0
    for ch in columns:
        col = int(ch)
        piece = 'X' if turn % 2 == 0 else 'O'
        row = drop(cells, col, piece)
        if row < 0:
            print(f"move {turn + 1}: column {col} is full")
            return
        turn += 1
        verdict = find_win(cells)
        if verdict != "none":
            print(f"move {turn}: {verdict}")
            show(cells)
            return
    print(f"no winner after {turn} moves")
    show(cells)


def main():
    boards = [
        [".......", ".......", ".......", ".......", "..O....", ".XXXX.O"],
        [".......", ".......", "...X...", "..XO...", ".XOO...", "XOXO..."],
        ["XOXOXOX", "XOXOXOX", "OXOXOXO", "OXOXOXO", "XOXOXOX", "XOXOXOX"],
    ]
    for rows in boards:
        print(f"board: {find_win(parse(rows))}")
    replay("3344556")
    replay("3232323")
    replay("0000000")


def tests():
    cells = parse([".......", ".......", ".......", ".......", ".......", "..OOOO."])
    assert find_win(cells) == "O from row 5 col 2 step 0,1"
    cells = parse([".......", ".......", "...X...", "..XO...", ".XOO...", "XOXO..."])
    assert find_win(cells) == "X from row 2 col 3 step 1,-1"
    cells = parse([".......", ".......", ".......", ".......", ".......", "......."])
    assert drop(cells, 2, 'X') == 5
    assert drop(cells, 2, 'O') == 4
    for i in range(4):
        drop(cells, 2, 'X')
    assert drop(cells, 2, 'O') == -1


tests()
main()
