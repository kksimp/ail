NAMES = ["red", "green", "yellow"]
LENGTHS = [6, 8, 3]


class Signal:
    def __init__(self):
        self.phase = 0
        self.remaining = LENGTHS[0]
        self.changes = 0
        self.early = 0
        self.spent = [0, 0, 0]

    def press(self):
        if self.phase == 1 and self.remaining > 2:
            self.remaining = 2
            self.early += 1

    def tick(self, t):
        self.spent[self.phase] += 1
        self.remaining -= 1
        if self.remaining == 0:
            self.phase = (self.phase + 1) % 3
            self.remaining = LENGTHS[self.phase]
            self.changes += 1
            print(f"t={t:>2} -> {NAMES[self.phase]}")


def main():
    presses = [10, 30, 31]
    sig = Signal()
    for t in range(1, 46):
        if t in presses:
            sig.press()
            print(f"t={t:>2} button pressed ({NAMES[sig.phase]}, {sig.remaining} left)")
        sig.tick(t)
    print(f"changes: {sig.changes}")
    print(f"shortened greens: {sig.early}")
    print(f"ticks red/green/yellow: {sig.spent}")
    print(f"final: {NAMES[sig.phase]} with {sig.remaining} left")


main()
