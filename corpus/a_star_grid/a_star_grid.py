INF = 1000000


def heuristic(cell, goal, width):
    return abs(cell // width - goal // width) + abs(cell % width - goal % width)


def search(cells, width, start, goal):
    total = len(cells)
    height = total // width
    g = [INF] * total
    came = [-1] * total
    closed = [False] * total
    g[start] = 0
    open_ = [start]
    expanded = 0
    dr = [-1, 0, 1, 0]
    dc = [0, 1, 0, -1]
    while open_:
        bi = 0
        for k in range(1, len(open_)):
            fa = g[open_[k]] + heuristic(open_[k], goal, width)
            fb = g[open_[bi]] + heuristic(open_[bi], goal, width)
            ha = heuristic(open_[k], goal, width)
            hb = heuristic(open_[bi], goal, width)
            if fa < fb or (fa == fb and (ha < hb or (ha == hb and open_[k] < open_[bi]))):
                bi = k
        cur = open_[bi]
        del open_[bi]
        if closed[cur]:
            continue
        closed[cur] = True
        expanded += 1
        if cur == goal:
            path = [cur]
            at = cur
            while came[at] != -1:
                at = came[at]
                path.append(at)
            path.reverse()
            return (True, path, expanded)
        row = cur // width
        col = cur % width
        for d in range(4):
            nr = row + dr[d]
            nc = col + dc[d]
            if nr < 0 or nr >= height or nc < 0 or nc >= width:
                continue
            nxt = nr * width + nc
            if cells[nxt] == "#" or closed[nxt]:
                continue
            if g[cur] + 1 < g[nxt]:
                g[nxt] = g[cur] + 1
                came[nxt] = cur
                open_.append(nxt)
    return (False, [], expanded)


def solve(label, rows):
    width = len(rows[0])
    cells = []
    start = 0
    goal = 0
    for r in rows:
        for c in r:
            if c == "S":
                start = len(cells)
            elif c == "G":
                goal = len(cells)
            cells.append(c)
    found, path, expanded = search(cells, width, start, goal)
    print(f"{label}:")
    if not found:
        print(f"  no path ({expanded} cells expanded)")
    else:
        for p in path:
            if p != start and p != goal:
                cells[p] = "*"
        print(f"  length {len(path) - 1}, {expanded} cells expanded")
    for r in range(len(rows)):
        print("  " + "".join(cells[r * width:(r + 1) * width]))


def main():
    solve("open field", [
        "S.......",
        "........",
        "........",
        ".......G",
    ])
    solve("with walls", [
        "S..#......",
        ".#.#.####.",
        ".#...#....",
        ".#####.##.",
        "......#..G",
    ])
    solve("sealed off", [
        "S.#.",
        "..#.",
        "###.",
        "...G",
    ])


main()
