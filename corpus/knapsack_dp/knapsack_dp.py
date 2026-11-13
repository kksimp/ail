def solve(items, cap):
    width = cap + 1
    table = [0] * ((len(items) + 1) * width)
    for row in range(1, len(items) + 1):
        name, weight, value = items[row - 1]
        for w in range(cap + 1):
            best = table[(row - 1) * width + w]
            if weight <= w:
                cand = table[(row - 1) * width + w - weight] + value
                if cand > best:
                    best = cand
            table[row * width + w] = best
    chosen = []
    left = cap
    idx = len(items)
    while idx > 0:
        if table[idx * width + left] != table[(idx - 1) * width + left]:
            chosen.append(idx - 1)
            left -= items[idx - 1][1]
        idx -= 1
    chosen.reverse()
    return chosen


def total_value(items, picks):
    return sum(items[i][2] for i in picks)


def total_weight(items, picks):
    return sum(items[i][1] for i in picks)


def main():
    items = [
        ("tent", 7, 12),
        ("stove", 3, 7),
        ("water", 4, 9),
        ("food", 5, 10),
        ("camera", 2, 6),
        ("books", 6, 5),
        ("jacket", 3, 4),
    ]
    for cap in [0, 5, 10, 15, 30]:
        picks = solve(items, cap)
        names = [items[i][0] for i in picks]
        label = "nothing" if not names else ", ".join(names)
        print(f"capacity {cap}: value {total_value(items, picks)}, weight {total_weight(items, picks)}: {label}")


main()
