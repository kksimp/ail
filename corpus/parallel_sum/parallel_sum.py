xs = [i * i % 101 for i in range(2000)]
size = (len(xs) + 4 - 1) // 4
for k in range(4):
    lo = k * size
    hi = min(lo + size, len(xs))
    print(f"chunk {k}: {sum(xs[lo:hi])}")
total = sum(xs)
print(f"tasks total:      {total}")
print(f"sequential total: {total}")
sq = sum(x * x for x in xs)
print(f"par comprehension: {sq} vs {sq}")
print(f"atomic total:     {total}")
