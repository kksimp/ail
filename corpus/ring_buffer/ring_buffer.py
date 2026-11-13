class Ring:
    def __init__(self, capacity):
        self.cap = capacity
        self.buf = [0] * capacity
        self.head = 0
        self.size = 0
        self.dropped = 0

    def push(self, v):
        if self.size == self.cap:
            self.buf[self.head] = v
            self.head = (self.head + 1) % self.cap
            self.dropped += 1
        else:
            self.buf[(self.head + self.size) % self.cap] = v
            self.size += 1

    def pop(self):
        if self.size == 0:
            return None
        v = self.buf[self.head]
        self.head = (self.head + 1) % self.cap
        self.size -= 1
        return v

    def items(self):
        return [self.buf[(self.head + i) % self.cap] for i in range(self.size)]

    def average(self):
        if self.size == 0:
            return 0
        return sum(self.items()) // self.size


def main():
    ring = Ring(4)
    readings = [10, 12, 9, 15, 20, 18, 7, 30, 25, 11]
    for r in readings:
        ring.push(r)
        print(f"push {r:>2} -> {ring.items()} avg {ring.average()}")
    print(f"dropped {ring.dropped}")
    drained = []
    while True:
        v = ring.pop()
        if v is None:
            break
        drained.append(v)
    print(f"drained {drained}, empty: {'true' if ring.size == 0 else 'false'}")


main()
