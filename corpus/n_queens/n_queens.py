def safe(cols, col):
    row = len(cols)
    for r in range(row):
        d = cols[r] - col
        if d == 0 or abs(d) == row - r:
            return False
    return True


def count(n, cols):
    if len(cols) == n:
        return 1
    total = 0
    for col in range(n):
        if safe(cols, col):
            cols.append(col)
            total += count(n, cols)
            cols.pop()
    return total


def first(n, cols):
    if len(cols) == n:
        return True
    for col in range(n):
        if safe(cols, col):
            cols.append(col)
            if first(n, cols):
                return True
            cols.pop()
    return False


def solutions(n):
    return count(n, [])


def draw(cols):
    lines = []
    for r in range(len(cols)):
        cells = ["Q" if cols[r] == c else "." for c in range(len(cols))]
        lines.append(" ".join(cells))
    return lines


def main():
    for n in range(1, 9):
        print(f"{n} queens: {solutions(n)} solutions")
    cols = []
    if first(6, cols):
        print(f"first 6x6 solution: {cols}")
        for line in draw(cols):
            print(line)
    else:
        print("no 6x6 solution")


main()
