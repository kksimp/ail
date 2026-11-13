class Life:
    def __init__(self, c, r):
        self.cols = c
        self.rows = r
        self.cells = [0] * (c * r)

    def set(self, x, y):
        self.cells[y * self.cols + x] = 1

    def alive(self, x, y):
        return self.cells[y * self.cols + x]

    def neighbors(self, x, y):
        n = 0
        for dy in range(-1, 2):
            for dx in range(-1, 2):
                if dx != 0 or dy != 0:
                    n += self.alive((x + dx + self.cols) % self.cols, (y + dy + self.rows) % self.rows)
        return n

    def step(self):
        nxt = Life(self.cols, self.rows)
        for y in range(self.rows):
            for x in range(self.cols):
                n = self.neighbors(x, y)
                if n == 3 or (n == 2 and self.alive(x, y) == 1):
                    nxt.set(x, y)
        return nxt

    def population(self):
        return sum(self.cells)

    def show(self):
        for y in range(self.rows):
            print("".join("#" if self.alive(x, y) == 1 else "." for x in range(self.cols)))


def main():
    g = Life(8, 8)
    for x, y in [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2), (5, 5), (6, 5), (7, 5)]:
        g.set(x, y)
    print("generation 0")
    g.show()
    pops = [g.population()]
    for i in range(1, 9):
        g = g.step()
        pops.append(g.population())
        if i == 4:
            print("generation 4")
            g.show()
    print(f"population by generation: {pops}")


main()
