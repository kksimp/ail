def count_paths(rows):
    h = len(rows)
    w = len(rows[0])
    ways = [0 for _ in range(h * w)]
    for r in range(h):
        cells = list(rows[r])
        for c in range(w):
            if cells[c] == '#':
                continue
            if r == 0 and c == 0:
                ways[0] = 1
                continue
            total = 0
            if r > 0:
                total += ways[(r - 1) * w + c]
            if c > 0:
                total += ways[r * w + c - 1]
            ways[r * w + c] = total
    return ways[h * w - 1]


def digit_grid(rows):
    out = []
    for line in rows:
        for ch in line:
            out.append(int(ch))
    return out


def cheapest(rows):
    h = len(rows)
    w = len(rows[0])
    cost = digit_grid(rows)
    best = [0 for _ in range(h * w)]
    for r in range(h):
        for c in range(w):
            i = r * w + c
            if r == 0 and c == 0:
                best[i] = cost[i]
            elif r == 0:
                best[i] = best[i - 1] + cost[i]
            elif c == 0:
                best[i] = best[i - w] + cost[i]
            else:
                best[i] = min(best[i - 1], best[i - w]) + cost[i]
    return best


def moves(rows, best):
    w = len(rows[0])
    r = len(rows) - 1
    c = w - 1
    steps = []
    while r > 0 or c > 0:
        if r == 0:
            steps.append("R")
            c -= 1
        elif c == 0:
            steps.append("D")
            r -= 1
        elif best[(r - 1) * w + c] <= best[r * w + c - 1]:
            steps.append("D")
            r -= 1
        else:
            steps.append("R")
            c -= 1
    steps.reverse()
    return "".join(steps)


def open_row(w):
    return "".join(["." for _ in range(w)])


def main():
    open_map = ["....", "....", "...."]
    print(f"open 3x4: {count_paths(open_map)} routes")
    walled = ["...#.", ".#...", "...#.", "#....", "....."]
    print(f"walled 5x5: {count_paths(walled)} routes")
    sealed = ["..#", "##.", "..."]
    print(f"sealed 3x3: {count_paths(sealed)} routes")
    big = [open_row(12) for _ in range(12)]
    print(f"open 12x12: {count_paths(big)} routes")
    costs = ["131", "425", "122", "913"]
    best = cheapest(costs)
    print(f"cheapest cost: {best[len(best) - 1]}")
    print(f"moves: {moves(costs, best)}")
    wide = ["19999", "11199", "99119", "99911"]
    wb = cheapest(wide)
    print(f"wide cost: {wb[len(wb) - 1]} via {moves(wide, wb)}")


main()
