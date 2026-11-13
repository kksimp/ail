class Bucket:
    def __init__(self, burst, per_sec):
        self.cap, self.rate, self.level, self.last = burst, per_sec, burst * 1000, 0

    def allow(self, now):
        elapsed = now - self.last
        self.last = now
        self.level = min(self.level + elapsed * self.rate, self.cap * 1000)
        if self.level >= 1000:
            self.level -= 1000
            return True
        return False


class Window:
    def __init__(self, limit, size):
        self.limit, self.size, self.start, self.count = limit, size, 0, 0

    def allow(self, now):
        slot = now - now % self.size
        if slot != self.start:
            self.start = slot
            self.count = 0
        if self.count < self.limit:
            self.count += 1
            return True
        return False


def verdict(ok):
    return "ok" if ok else "deny"


times = [0, 50, 120, 300, 450, 900, 1000, 1010, 1020, 1500, 1900, 2000, 2400, 2500, 2600, 3000, 3500]
bucket = Bucket(3, 2)
window = Window(3, 1000)
b_ok = w_ok = 0
print(f"{'t_ms':>6}  {'bucket':<7}{'window':<7}")
for t in times:
    b = bucket.allow(t)
    w = window.allow(t)
    b_ok += b
    w_ok += w
    print(f"{t:>6}  {verdict(b):<7}{verdict(w):<7}")
print(f"bucket allowed {b_ok} of {len(times)}, window allowed {w_ok} of {len(times)}")
