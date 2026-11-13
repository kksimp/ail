class Maze:
    def __init__(self, width, height, walls, start, goal):
        self.width = width
        self.height = height
        self.walls = walls
        self.start = start
        self.goal = goal


def parse(rows):
    walls = []
    start = 0
    goal = 0
    for line in rows:
        for ch in line:
            if ch == "S":
                start = len(walls)
            if ch == "E":
                goal = len(walls)
            walls.append(1 if ch == "#" else 0)
    return Maze(len(rows[0]), len(rows), walls, start, goal)


def explore(m):
    prev = [-2] * len(m.walls)
    dxs = [0, 1, 0, -1]
    dys = [-1, 0, 1, 0]
    queue = [m.start]
    prev[m.start] = -1
    head = 0
    while head < len(queue):
        cur = queue[head]
        head += 1
        if cur == m.goal:
            break
        for d in range(4):
            nx = cur % m.width + dxs[d]
            ny = cur // m.width + dys[d]
            if nx < 0 or nx >= m.width or ny < 0 or ny >= m.height:
                continue
            spot = ny * m.width + nx
            if m.walls[spot] == 0 and prev[spot] == -2:
                prev[spot] = cur
                queue.append(spot)
    return prev


def path_to_goal(m, prev):
    path = []
    if prev[m.goal] == -2:
        return path
    here = m.goal
    while here != -1:
        path.append(here)
        here = prev[here]
    path.reverse()
    return path


def reached(prev):
    n = 0
    for p in prev:
        if p != -2:
            n += 1
    return n


def draw(m, path):
    cells = ["#" if w == 1 else "." for w in m.walls]
    for step in path:
        cells[step] = "*"
    cells[m.start] = "S"
    cells[m.goal] = "E"
    rows = []
    for r in range(m.height):
        lo = r * m.width
        hi = lo + m.width
        rows.append("".join(cells[lo:hi]))
    return rows


def report(name, rows):
    m = parse(rows)
    prev = explore(m)
    path = path_to_goal(m, prev)
    if not path:
        print(f"{name}: no way out, {reached(prev)} cells reachable")
        return
    print(f"{name}: {len(path) - 1} steps, {reached(prev)} cells reached")
    for line in draw(m, path):
        print(f"  {line}")


def main():
    report("maze A", [
        "##########",
        "#S.#.....#",
        "#..#.###.#",
        "#..#...#.#",
        "#..###.#.#",
        "#......#E#",
        "##########",
    ])
    report("maze B", [
        "#########",
        "#S..#...#",
        "#.#.#.#.#",
        "#.#...#E#",
        "###.###.#",
        "#.......#",
        "#########",
    ])
    report("maze C", [
        "#######",
        "#S..#.#",
        "###.#E#",
        "#...###",
        "#######",
    ])


main()
