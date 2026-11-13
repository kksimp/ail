class SuffixArray:
    def __init__(self, source):
        count = len(source)
        order = sorted(range(count), key=lambda i: source[i:count])
        pos_of = [0] * count
        for k in range(count):
            pos_of[order[k]] = k
        common = [0] * count
        h = 0
        for i in range(count):
            if pos_of[i] > 0:
                j = order[pos_of[i] - 1]
                while i + h < count and j + h < count and source[i + h] == source[j + h]:
                    h += 1
                common[pos_of[i]] = h
                if h > 0:
                    h -= 1
            else:
                h = 0
        self.text = source
        self.sa = order
        self.lcp = common

    def suffix(self, k):
        return self.text[self.sa[k]:]

    def distinct(self):
        n = len(self.text)
        total = n * (n + 1) // 2
        for v in self.lcp:
            total -= v
        return total

    def longest_repeat(self):
        best = 0
        at = 0
        for k in range(len(self.lcp)):
            if self.lcp[k] > best:
                best = self.lcp[k]
                at = k
        return self.text[self.sa[at]:self.sa[at] + best]

    def lower_bound(self, pattern):
        lo = 0
        hi = len(self.sa)
        while lo < hi:
            mid = (lo + hi) // 2
            if self.suffix(mid) < pattern:
                lo = mid + 1
            else:
                hi = mid
        return lo

    def find_all(self, pattern):
        start = self.lower_bound(pattern)
        stop = start
        while stop < len(self.sa) and self.suffix(stop).startswith(pattern):
            stop += 1
        return sorted(self.sa[start:stop])


sa = SuffixArray("mississippi")
print(f"sa:  {sa.sa}")
print(f"lcp: {sa.lcp}")
for k in range(4):
    print(f"suffix {k}: {sa.suffix(k)}")
print(f"distinct substrings: {sa.distinct()}")
print(f"longest repeat: {sa.longest_repeat()}")
for p in ["ss", "i", "issi", "pp", "x", "mississippi", "ippi"]:
    hits = sa.find_all(p)
    print(f"{p}: {len(hits)} at {hits}")
ab = SuffixArray("abracadabra")
print(f"abracadabra sa: {ab.sa}")
print(f"abracadabra distinct: {ab.distinct()}, repeat: {ab.longest_repeat()}")
