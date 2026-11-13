class Fenwick:
    def __init__(self, n):
        self.size = n
        self.cells = [0] * (n + 1)

    def add(self, pos, delta):
        i = pos
        while i <= self.size:
            self.cells[i] += delta
            i += i & (0 - i)

    def prefix(self, pos):
        i = pos
        total = 0
        while i > 0:
            total += self.cells[i]
            i -= i & (0 - i)
        return total

    def range(self, l, r):
        return self.prefix(r) - self.prefix(l - 1)

    def kth(self, k):
        pos = 0
        left = k
        step = 1
        while step * 2 <= self.size:
            step *= 2
        while step > 0:
            nxt = pos + step
            if nxt <= self.size and self.cells[nxt] < left:
                pos = nxt
                left -= self.cells[nxt]
            step //= 2
        return pos + 1


def inversions(xs, top):
    fw = Fenwick(top)
    count = 0
    seen = 0
    for x in xs:
        count += seen - fw.prefix(x)
        fw.add(x, 1)
        seen += 1
    return count


def inversions_slow(xs):
    count = 0
    for i in range(len(xs)):
        for j in range(i + 1, len(xs)):
            if xs[i] > xs[j]:
                count += 1
    return count


fw = Fenwick(10)
for p in range(1, 11):
    fw.add(p, p * p % 7)
print(f"prefix sums: {[fw.prefix(q) for q in range(1, 11)]}")
print(f"range 3..7 = {fw.range(3, 7)}")
print(f"range 1..10 = {fw.range(1, 10)}")
fw.add(4, 10)
print(f"after add 10 at 4: range 3..7 = {fw.range(3, 7)}")
counts = Fenwick(8)
for v in [3, 1, 4, 1, 5, 8, 2, 6, 5, 3, 5]:
    counts.add(v, 1)
for k in [1, 4, 6, 11]:
    print(f"k-th smallest, k={k}: {counts.kth(k)}")
seq = [5, 3, 8, 1, 9, 2, 7, 4, 6, 3]
print(f"inversions: {inversions(seq, 9)} (slow check {inversions_slow(seq)})")
print(f"sorted input: {inversions([1, 2, 3, 4, 5], 5)}")
print(f"reversed input: {inversions([5, 4, 3, 2, 1], 5)}")
