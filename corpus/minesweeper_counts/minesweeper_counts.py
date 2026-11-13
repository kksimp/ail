class Field:
    def __init__(self, cols, rows, cells):
        self.cols, self.rows, self.cells = cols, rows, cells

    @staticmethod
    def parse(lines):
        cells = []
        for line in lines:
            for ch in line:
                cells.append(1 if ch == '*' else 0)
        return Field(len(lines[0]), len(lines), cells)

    def inside(self, x, y):
        return 0 <= x < self.cols and 0 <= y < self.rows

    def near(self, x, y):
        n = 0
        for dy in range(-1, 2):
            for dx in range(-1, 2):
                if (dx != 0 or dy != 0) and self.inside(x + dx, y + dy):
                    n += self.cells[(y + dy) * self.cols + x + dx]
        return n

    def glyph(self, x, y):
        if self.cells[y * self.cols + x] == 1:
            return "*"
        n = self.near(x, y)
        if n == 0:
            return "."
        return str(n)

    def hints(self):
        return ["".join(self.glyph(x, y) for x in range(self.cols)) for y in range(self.rows)]

    def reveal(self, sx, sy):
        seen = [0] * len(self.cells)
        queue = [sy * self.cols + sx]
        seen[sy * self.cols + sx] = 1
        head = 0
        while head < len(queue):
            cur = queue[head]
            head += 1
            x = cur % self.cols
            y = cur // self.cols
            if self.cells[cur] == 1 or self.near(x, y) > 0:
                continue
            for dy in range(-1, 2):
                for dx in range(-1, 2):
                    spot = (y + dy) * self.cols + x + dx
                    if self.inside(x + dx, y + dy) and seen[spot] == 0:
                        seen[spot] = 1
                        queue.append(spot)
        return seen

    def show_open(self, seen):
        return ["".join(self.glyph(x, y) if seen[y * self.cols + x] == 1 else "#" for x in range(self.cols)) for y in range(self.rows)]


def main():
    f = Field.parse([
        "..*.....",
        "........",
        ".*..*...",
        "........",
        "......*.",
        "..*.....",
    ])
    print(f"mines: {sum(f.cells)} on {f.cols}x{f.rows}")
    for line in f.hints():
        print(line)
    seen = f.reveal(7, 0)
    print(f"click at 7,0 opens {sum(seen)} cells")
    for line in f.show_open(seen):
        print(line)
    boom = f.reveal(2, 0)
    print(f"click at 2,0 opens {sum(boom)} cell(s)")


def tests():
    f = Field.parse(["*.", ".."])
    assert f.near(1, 1) == 1 and f.near(0, 0) == 0 and f.glyph(0, 0) == "*"
    f = Field.parse(["*..", "...", "..*"])
    assert f.hints() == ["*1.", "121", ".1*"]
    f = Field.parse(["....", "....", "...*"])
    assert sum(f.reveal(0, 0)) == 11


tests()
main()
