SLOTS = 64
HASHES = 3


def slot(x, i):
    return (x * (2 * i + 3) + i * i * 11 + 5) % SLOTS


bits = [0] * SLOTS
added = 0
keys = [k * 9 + 4 for k in range(16)]
for k in keys:
    for i in range(HASHES):
        bits[slot(k, i)] = 1
    added += 1


def check(x):
    for i in range(HASHES):
        if bits[slot(x, i)] == 0:
            return False
    return True


print(f"added {added} keys, {sum(bits)} of {SLOTS} bits set")
missed = 0
for k in keys:
    if not check(k):
        missed += 1
print(f"false negatives: {missed}")
positives = []
maybe_count = 0
for x in range(200):
    if x not in keys:
        if check(x):
            maybe_count += 1
            if len(positives) < 8:
                positives.append(x)
print(f"false positives: {maybe_count} of {200 - len(keys)}")
print(f"first false positives: {positives}")
