class Entry:
    def __init__(self, key, val, expires):
        self.key = key
        self.val = val
        self.expires = expires


class TtlCache:
    def __init__(self, capacity):
        self.cap = capacity
        self.now = 0
        self.items = []
        self.hits = 0
        self.misses = 0
        self.expired = 0
        self.evicted = 0

    def find(self, k):
        for i in range(len(self.items)):
            if self.items[i].key == k:
                return i
        return -1

    def purge(self):
        kept = []
        for e in self.items:
            if e.expires > self.now:
                kept.append(e)
            else:
                self.expired += 1
        self.items = kept

    def put(self, k, v, ttl):
        self.purge()
        i = self.find(k)
        if i >= 0:
            del self.items[i]
        elif len(self.items) >= self.cap:
            del self.items[0]
            self.evicted += 1
        self.items.append(Entry(k, v, self.now + ttl))

    def get(self, k):
        i = self.find(k)
        if i >= 0:
            e = self.items[i]
            del self.items[i]
            if e.expires > self.now:
                self.items.append(e)
                self.hits += 1
                return e.val
            self.expired += 1
        self.misses += 1
        return None

    def tick(self, dt):
        self.now += dt


def run(script):
    cache = TtlCache(3)
    lines = []
    for cmd in script.split(";"):
        parts = cmd.split(" ")
        op = parts[0]
        a = int(parts[1]) if len(parts) > 1 else 0
        if op == "p":
            cache.put(a, int(parts[2]), int(parts[3]))
        elif op == "t":
            cache.tick(a)
        else:
            v = cache.get(a)
            if v is not None:
                lines.append(f"t={cache.now} get {a} -> {v}")
            else:
                lines.append(f"t={cache.now} get {a} -> miss")
    keys = [e.key for e in cache.items]
    lines.append(f"live keys oldest first: {keys}")
    lines.append(f"hits {cache.hits}, misses {cache.misses}, expired {cache.expired}, evicted {cache.evicted}")
    return lines


script = "p 1 10 5;p 2 20 3;p 3 30 9;g 1;t 4;g 2;g 1;p 4 40 2;p 5 50 6;g 3;t 3;g 4;g 5;p 6 60 5;g 3"
for line in run(script):
    print(line)
