spans = []


def add(lo, hi, label):
    at = len(spans)
    for i in range(len(spans)):
        if spans[i][0] > lo:
            at = i
            break
    spans.insert(at, (lo, hi, label))


def find(q):
    out = []
    for lo, hi, label in spans:
        if q[0] == "point":
            hit = lo <= q[1] <= hi
        else:
            hit = lo <= q[2] and q[1] <= hi
        if hit:
            out.append(label)
    return out


def covered():
    total = 0
    reach = -1
    for lo, hi, _ in spans:
        if hi > reach:
            start = max(lo, reach + 1)
            total += hi - start + 1
            reach = hi
    return total


add(15, 20, "dev")
add(10, 14, "plan")
add(5, 8, "kickoff")
add(18, 30, "release")
add(40, 45, "retro")
print("sorted: " + ",".join(s[2] for s in spans))
queries = [("point", 19), ("point", 9), ("range", 12, 16), ("range", 31, 39), ("range", 1, 100)]
for q in queries:
    found = find(q)
    desc = f"point {q[1]}" if q[0] == "point" else f"range {q[1]}..{q[2]}"
    print(f"{desc}: {len(found)} -> {','.join(found)}")
print(f"covered length: {covered()}")
