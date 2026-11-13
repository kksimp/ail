SIZE = 24


class Ant:
    def __init__(self, rules):
        self.cells = [0] * (SIZE * SIZE)
        self.rules = list(rules)
        self.x, self.y, self.heading = 12, 12, 0

    def step(self):
        spot = self.y * SIZE + self.x
        colour = self.cells[spot]
        if self.rules[colour] == 'R':
            self.heading = (self.heading + 1) % 4
        else:
            self.heading = (self.heading + 3) % 4
        self.cells[spot] = (colour + 1) % len(self.rules)
        dxs = [0, 1, 0, -1]
        dys = [-1, 0, 1, 0]
        self.x = (self.x + dxs[self.heading] + SIZE) % SIZE
        self.y = (self.y + dys[self.heading] + SIZE) % SIZE

    def coloured(self):
        return sum(1 for c in self.cells if c != 0)

    def render(self):
        arrows = ["^", ">", "v", "<"]
        out = []
        for r in range(SIZE):
            row = []
            for c in range(SIZE):
                if r == self.y and c == self.x:
                    row.append(arrows[self.heading])
                elif self.cells[r * SIZE + c] != 0:
                    row.append("#")
                else:
                    row.append(".")
            out.append("".join(row))
        return out


def run(rules, checkpoints):
    a = Ant(rules)
    steps = 0
    for upto in checkpoints:
        while steps < upto:
            a.step()
            steps += 1
        print(f"  {rules}: step {steps:>4} coloured {a.coloured():>3} at {a.x},{a.y}")


def main():
    for rules in ["RL", "LLRR", "RLR"]:
        print(f"rule {rules}")
        run(rules, [10, 100, 400, 1000])
    a = Ant("RL")
    for i in range(120):
        a.step()
    print("classic ant after 120 steps")
    for line in a.render():
        print(line)


def tests():
    a = Ant("RL")
    a.step()
    assert a.coloured() == 1 and a.x == 13 and a.y == 12
    a.step()
    assert a.x == 13 and a.y == 13
    a = Ant("RRRR")
    for i in range(4):
        a.step()
    assert a.x == 12 and a.y == 12
    a = Ant("RL")
    a.x = 23
    a.heading = 0
    a.step()
    assert a.x == 0


tests()
main()
