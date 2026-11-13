class SegTree:
    def __init__(self, data):
        n = len(data)
        self.size = n
        self.tree = [0] * (2 * n)
        for i in range(n):
            self.tree[n + i] = data[i]
        for j in range(n - 1, 0, -1):
            self.tree[j] = self.tree[2 * j] + self.tree[2 * j + 1]

    def update(self, pos, val):
        p = pos + self.size
        self.tree[p] = val
        while p > 1:
            p //= 2
            self.tree[p] = self.tree[2 * p] + self.tree[2 * p + 1]

    def query(self, l, r):
        lo = l + self.size
        hi = r + self.size
        total = 0
        while lo < hi:
            if lo % 2 == 1:
                total += self.tree[lo]
                lo += 1
            if hi % 2 == 1:
                hi -= 1
                total += self.tree[hi]
            lo //= 2
            hi //= 2
        return total


def brute(data, l, r):
    return sum(data[l:r])


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


seed = 2024
data = []
for _ in range(13):
    seed = next_rand(seed)
    data.append(seed // 65536 % 100)
print(f"data: {data}")
st = SegTree(data)
print(f"total: {st.query(0, 13)}")
for a, b in [(0, 5), (3, 9), (7, 8), (4, 4), (10, 13)]:
    print(f"sum[{a}, {b}) = {st.query(a, b)}")
for a, b in [(2, 500), (12, 0), (6, 77)]:
    st.update(a, b)
    data[a] = b
    print(f"set {a} = {b}: total {st.query(0, 13)}")
mismatches = 0
for l in range(13):
    for r in range(l, 14):
        if st.query(l, r) != brute(data, l, r):
            mismatches += 1
print(f"mismatches against brute force: {mismatches}")
