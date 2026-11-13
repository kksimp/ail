from collections import deque


class Ring:
    def __init__(self):
        self.buf = [0, 0, 0, 0]
        self.head = 0
        self.size = 0
        self.grows = 0

    def grow(self):
        bigger = [self.buf[(self.head + i) % len(self.buf)] for i in range(self.size)]
        while len(bigger) < len(self.buf) * 2:
            bigger.append(0)
        self.buf = bigger
        self.head = 0
        self.grows += 1

    def push_back(self, x):
        if self.size == len(self.buf):
            self.grow()
        self.buf[(self.head + self.size) % len(self.buf)] = x
        self.size += 1

    def push_front(self, x):
        if self.size == len(self.buf):
            self.grow()
        self.head = (self.head + len(self.buf) - 1) % len(self.buf)
        self.buf[self.head] = x
        self.size += 1

    def pop_front(self):
        if self.size == 0:
            return None
        v = self.buf[self.head]
        self.head = (self.head + 1) % len(self.buf)
        self.size -= 1
        return v

    def pop_back(self):
        if self.size == 0:
            return None
        self.size -= 1
        return self.buf[(self.head + self.size) % len(self.buf)]

    def items(self):
        return [self.buf[(self.head + i) % len(self.buf)] for i in range(self.size)]


def run(script):
    ring = Ring()
    ref = deque()
    lines = []
    for cmd in script.split(";"):
        parts = cmd.split(" ")
        op = parts[0]
        n = int(parts[1]) if len(parts) > 1 else 0
        if op == "pf":
            ring.push_front(n)
            ref.appendleft(n)
        elif op == "pb":
            ring.push_back(n)
            ref.append(n)
        elif op == "xf":
            ring.pop_front()
            if ref:
                ref.popleft()
        else:
            ring.pop_back()
            if ref:
                ref.pop()
    lines.append(f"ring {ring.items()} grows {ring.grows}")
    lines.append(f"std  {list(ref)}")
    lines.append("agree " + str(ring.items() == list(ref)).lower())
    return lines


for line in run("pb 1;pb 2;pf 0;pb 3;pb 4;pf -1;xf;xb;pb 5;pb 6;pf 7;pf 8"):
    print(line)
for line in run("pf 1;pf 2;xb;xb;xb;pb 9"):
    print(line)
