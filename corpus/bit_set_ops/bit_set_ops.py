WORD = 32


class BitSet:
    def __init__(self, n):
        self.words = [0] * ((n + WORD - 1) // WORD)
        self.limit = n

    def has(self, i):
        return ((self.words[i // WORD] >> (i % WORD)) & 1) == 1

    def add(self, i):
        if not self.has(i):
            self.words[i // WORD] |= 1 << (i % WORD)

    def remove(self, i):
        if self.has(i):
            self.words[i // WORD] ^= 1 << (i % WORD)

    def count(self):
        return sum(popcount(w) for w in self.words)

    def members(self):
        return [i for i in range(self.limit) if self.has(i)]

    def combine(self, other, mode):
        res = BitSet(self.limit)
        for i in range(len(self.words)):
            res.words[i] = word_op(self.words[i], other.words[i], mode)
        return res


def popcount(w):
    x = w
    c = 0
    while x > 0:
        c += x & 1
        x = x >> 1
    return c


def word_op(a, b, mode):
    if mode == 0:
        return a | b
    if mode == 1:
        return a & b
    return a ^ (a & b)


def sieve(n):
    composite = BitSet(n)
    for i in range(2, n):
        if not composite.has(i):
            j = i * i
            while j < n:
                composite.add(j)
                j += i
    primes = BitSet(n)
    for i in range(2, n):
        if not composite.has(i):
            primes.add(i)
    return primes


def multiples(step, n):
    s = BitSet(n)
    for i in range(n):
        if i % step == 0:
            s.add(i)
    return s


primes = sieve(100)
print(f"primes below 100: {primes.count()}")
print(f"primes below 40: {primes.members()[:12]}")
threes = multiples(3, 100)
fives = multiples(5, 100)
print(f"multiples of 3: {threes.count()}, of 5: {fives.count()}")
print(f"either: {threes.combine(fives, 0).count()}")
print(f"both: {threes.combine(fives, 1).members()}")
print(f"3 but not 5: {threes.combine(fives, 2).count()}")
print(f"prime multiples of 3 or 5: {primes.combine(threes.combine(fives, 0), 1).members()}")
s = BitSet(70)
for i in [0, 31, 32, 33, 63, 64, 69]:
    s.add(i)
print(f"edge bits: {s.members()} in {len(s.words)} words")
s.remove(32)
s.remove(5)
print(f"after removes: {s.count()} members, words {s.words}")
