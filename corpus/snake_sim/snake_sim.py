class Game:
    def __init__(self, cols, rows, foods):
        self.cols, self.rows, self.body, self.foods = cols, rows, [0], foods
        self.eaten, self.alive = 0, True

    def step(self, d):
        last = self.body[-1]
        nx = last % self.cols
        ny = last // self.cols
        if d == 'U':
            ny -= 1
        elif d == 'D':
            ny += 1
        elif d == 'L':
            nx -= 1
        else:
            nx += 1
        if nx < 0 or nx >= self.cols or ny < 0 or ny >= self.rows:
            self.alive = False
            return f"hit the wall at {nx},{ny}"
        spot = ny * self.cols + nx
        eating = self.eaten < len(self.foods) and self.foods[self.eaten] == spot
        blocked = spot in self.body
        if not eating and spot == self.body[0]:
            blocked = False
        if blocked:
            self.alive = False
            return f"bit itself at {nx},{ny}"
        self.body.append(spot)
        if eating:
            self.eaten += 1
            return f"ate food {self.eaten}, length {len(self.body)}"
        self.body.pop(0)
        return ""

    def render(self):
        out = []
        for y in range(self.rows):
            row = []
            for x in range(self.cols):
                spot = y * self.cols + x
                mark = "."
                if self.eaten < len(self.foods) and self.foods[self.eaten] == spot:
                    mark = "*"
                if spot in self.body:
                    mark = "o"
                if spot == self.body[-1]:
                    mark = "@"
                row.append(mark)
            out.append("".join(row))
        return out


def run(moves, foods):
    g = Game(6, 5, foods)
    n = 0
    for ch in moves:
        if not g.alive:
            break
        n += 1
        note = g.step(ch)
        if note != "":
            print(f"  move {n} ({ch}): {note}")
    return g


def fmt(b):
    return "true" if b else "false"


def main():
    print("run 1")
    a = run("RRRDDRDDLLLUUR", [3, 15, 22])
    for line in a.render():
        print(line)
    print(f"alive={fmt(a.alive)} length={len(a.body)} eaten={a.eaten}")
    print("run 2")
    b = run("RRRRRRR", [])
    print(f"alive={fmt(b.alive)} length={len(b.body)}")


def tests():
    g = Game(4, 4, [1])
    note = g.step('R')
    assert len(g.body) == 2 and note.startswith("ate")
    g = Game(4, 4, [])
    g.step('U')
    assert not g.alive
    g = Game(4, 4, [1, 5, 4])
    for c in "RDL":
        g.step(c)
    note = g.step('U')
    assert g.alive and g.eaten == 3 and note == ""


tests()
main()
