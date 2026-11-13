def parse(rows):
    cells = []
    for line in rows:
        for ch in line:
            cells.append(ch)
    return cells


def fill(cells, width, height, start, target, paint):
    if cells[start] != target or target == paint:
        return 0
    stack = [start]
    cells[start] = paint
    filled = 0
    while stack:
        cur = stack.pop()
        filled += 1
        x = cur % width
        y = cur // width
        if x > 0 and cells[cur - 1] == target:
            cells[cur - 1] = paint
            stack.append(cur - 1)
        if x < width - 1 and cells[cur + 1] == target:
            cells[cur + 1] = paint
            stack.append(cur + 1)
        if y > 0 and cells[cur - width] == target:
            cells[cur - width] = paint
            stack.append(cur - width)
        if y < height - 1 and cells[cur + width] == target:
            cells[cur + width] = paint
            stack.append(cur + width)
    return filled


def count_of(cells, what):
    n = 0
    for c in cells:
        if c == what:
            n += 1
    return n


def label_islands(cells, width, height):
    names = "ABCDEFGHIJ"
    sizes = []
    for i in range(len(cells)):
        if cells[i] == "#":
            tag = names[len(sizes)]
            size = fill(cells, width, height, i, "#", tag)
            sizes.append(size)
    return sizes


def show(title, cells, width):
    print(title)
    for r in range(len(cells) // width):
        lo = r * width
        hi = lo + width
        print("  " + "".join(cells[lo:hi]))


def main():
    rows = [
        "............",
        ".####...##..",
        ".#..#..####.",
        ".####...##..",
        "............",
        "..#......#..",
        "............",
    ]
    width = len(rows[0])
    height = len(rows)
    cells = parse(rows)
    show("original:", cells, width)
    outside = fill(cells, width, height, 0, ".", "~")
    print(f"outside paper cells: {outside}")
    print(f"enclosed paper cells: {count_of(cells, '.')}")
    sizes = label_islands(cells, width, height)
    show("labelled:", cells, width)
    names = "ABCDEFGHIJ"
    for k in range(len(sizes)):
        print(f"island {names[k]}: {sizes[k]} cells")
    print(f"ink total: {sum(sizes)}")


main()
