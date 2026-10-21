counts = {}
seed = 5
for _ in range(200000):
    seed = (seed * 3877 + 29573) % 139968
    k = f"key{seed % 20000}"
    counts[k] = counts.get(k, 0) + 1
hits = 0
for i in range(20000):
    k = f"key{i}"
    if k in counts:
        hits += counts[k]
top = sorted(counts, key=lambda k: -counts[k])[:5]
for k in top:
    print(f"{k} {counts[k]}")
print(f"distinct {len(counts)} hits {hits}")
