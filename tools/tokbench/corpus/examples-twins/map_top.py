# Python twin of examples/map_top.ail: the most frequent keys, ties in first-seen order.
import json
from collections import Counter

codes = ["E12", "E07", "E12", "E30", "E07", "E12", "E41", "E30", "E55"]
counts = Counter()
for c in codes:
    counts[c] += 1


def top(n):
    return [k for k, _ in sorted(counts.items(), key=lambda kv: -kv[1])][: max(n, 0)]


print(json.dumps(top(3)))
print(json.dumps(top(0)))
for code in top(2):
    print(f"{code} x{counts[code]}")
