class LruCache:
    def __init__(self, capacity):
        self.cap = capacity
        self.keys = []
        self.vals = []
        self.hits = 0
        self.misses = 0
        self.evictions = 0

    def get(self, k):
        if k in self.keys:
            i = self.keys.index(k)
            v = self.vals[i]
            self.keys.pop(i)
            self.vals.pop(i)
            self.keys.append(k)
            self.vals.append(v)
            self.hits += 1
            return v
        self.misses += 1
        return None

    def put(self, k, v):
        present = False
        if k in self.keys:
            i = self.keys.index(k)
            self.keys.pop(i)
            self.vals.pop(i)
            present = True
        if not present and len(self.keys) >= self.cap:
            self.keys.pop(0)
            self.vals.pop(0)
            self.evictions += 1
        self.keys.append(k)
        self.vals.append(v)


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


def main():
    cache = LruCache(3)
    seed = 42
    total = 0
    for i in range(40):
        seed = next_rand(seed)
        key = seed // 65536 % 6
        if i % 3 == 0:
            cache.put(key, i * 10)
        else:
            v = cache.get(key)
            if v is not None:
                total += v
    print(f"hits: {cache.hits}")
    print(f"misses: {cache.misses}")
    print(f"evictions: {cache.evictions}")
    print(f"sum of hit values: {total}")
    print(f"order oldest to newest: {cache.keys}")


main()
