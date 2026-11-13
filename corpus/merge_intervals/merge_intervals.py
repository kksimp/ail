def parse(text):
    parts = text.split("-")
    a = int(parts[0])
    b = int(parts[1])
    return [min(a, b), max(a, b)]


def merge(items):
    ordered = sorted(items, key=lambda p: p[0])
    out = []
    for cur in ordered:
        if not out:
            out.append(list(cur))
        else:
            n = len(out) - 1
            if cur[0] <= out[n][1] + 1:
                out[n][1] = max(out[n][1], cur[1])
            else:
                out.append(list(cur))
    return out


def covered(items):
    total = 0
    for p in items:
        total += p[1] - p[0] + 1
    return total


def gaps(items):
    out = []
    for i in range(1, len(items)):
        out.append([items[i - 1][1] + 1, items[i][0] - 1])
    return out


def show(items):
    return " ".join(f"[{p[0]}, {p[1]}]" for p in items)


def main():
    raw = ["1-3", "2-6", "8-10", "15-18", "17-20", "11-12", "30-30", "25-22"]
    items = [parse(t) for t in raw]
    print(f"input:  {show(items)}")
    merged = merge(items)
    print(f"merged: {show(merged)}")
    print(f"ranges: {len(merged)}")
    print(f"covered: {covered(merged)}")
    print(f"gaps:   {show(gaps(merged))}")
    touching = merge([parse("1-2"), parse("3-4")])
    print(f"touching: {show(touching)}")


main()
