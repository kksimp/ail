SIZE = 16
MAX_GENS = 12


def empty():
    return [0] * (SIZE * SIZE)


def place(lines, ox, oy):
    b = empty()
    for y, line in enumerate(lines):
        for x, ch in enumerate(line):
            if ch == '#':
                b[(oy + y) * SIZE + ox + x] = 1
    return b


def at(b, x, y):
    if x < 0 or x >= SIZE or y < 0 or y >= SIZE:
        return 0
    return b[y * SIZE + x]


def step(b):
    nxt = empty()
    for y in range(SIZE):
        for x in range(SIZE):
            n = 0
            for dy in range(-1, 2):
                for dx in range(-1, 2):
                    if dx != 0 or dy != 0:
                        n += at(b, x + dx, y + dy)
            if n == 3 or (n == 2 and at(b, x, y) == 1):
                nxt[y * SIZE + x] = 1
    return nxt


def corner(b):
    lo_x = lo_y = SIZE
    for i in range(len(b)):
        if b[i] == 1:
            lo_x = min(lo_x, i % SIZE)
            lo_y = min(lo_y, i // SIZE)
    return [lo_x, lo_y]


def shape(b):
    c = corner(b)
    return ";".join(f"{i % SIZE - c[0]},{i // SIZE - c[1]}" for i in range(len(b)) if b[i] == 1)


def classify(name, lines):
    start = place(lines, 2, 2)
    b = start
    for gen in range(1, MAX_GENS + 1):
        b = step(b)
        if sum(b) == 0:
            print(f"{name}: dies out at generation {gen}")
            return
        if shape(b) == shape(start):
            c0, c1 = corner(start), corner(b)
            print(f"{name}: period {gen}, shift ({c1[0] - c0[0]},{c1[1] - c0[1]}), population {sum(b)}")
            return
    print(f"{name}: no period within {MAX_GENS} generations, population {sum(b)}")


def main():
    classify("block", ["##", "##"])
    classify("beehive", [".##.", "#..#", ".##."])
    classify("blinker", ["###"])
    classify("toad", [".###", "###."])
    classify("beacon", ["##..", "##..", "..##", "..##"])
    classify("glider", [".#.", "..#", "###"])
    classify("r-pentomino", [".##", "##.", ".#."])
    classify("lonely pair", ["##"])


def tests():
    s = place(["###"], 2, 2)
    assert shape(step(s)) != shape(s)
    assert shape(step(step(s))) == shape(s)
    s = place(["##", "##"], 5, 5)
    assert step(s) == s
    s = place([".#.", "..#", "###"], 2, 2)
    b = s
    for i in range(4):
        b = step(b)
    assert shape(b) == shape(s) and corner(b) == [3, 3]


tests()
main()
