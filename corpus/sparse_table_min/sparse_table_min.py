class Sparse:
    def __init__(self, data):
        count = len(data)
        self.logs = [0, 0]
        for i in range(2, count + 1):
            self.logs.append(self.logs[i // 2] + 1)
        self.levels = [data]
        width = 1
        while width * 2 <= count:
            prev = self.levels[-1]
            row = []
            for i in range(count - width * 2 + 1):
                row.append(min(prev[i], prev[i + width]))
            self.levels.append(row)
            width *= 2

    def min_of(self, l, r):
        k = self.logs[r - l]
        row = self.levels[k]
        return min(row[l], row[r - (1 << k)])


def argmin_scan(data, l, r):
    best = l
    for i in range(l, r):
        if data[i] < data[best]:
            best = i
    return best


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


seed = 99
data = []
for _ in range(20):
    seed = next_rand(seed)
    data.append(seed // 65536 % 90 + 10)
print(f"data: {data}")
sp = Sparse(data)
print(f"levels: {len(sp.levels)}")
for lo, hi in [(0, 20), (0, 1), (5, 12), (13, 20), (3, 4), (8, 16)]:
    m = sp.min_of(lo, hi)
    pos = argmin_scan(data, lo, hi)
    print(f"min[{lo}, {hi}) = {m} at {pos}")
bad = 0
checked = 0
for l in range(20):
    for r in range(l + 1, 21):
        checked += 1
        if sp.min_of(l, r) != data[argmin_scan(data, l, r)]:
            bad += 1
print(f"checked {checked} ranges, {bad} wrong")
