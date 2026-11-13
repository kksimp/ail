def data():
    return [
        "u1,2026-01", "u2,2026-01", "u3,2026-01", "u1,2026-02", "u2,2026-02",
        "u4,2026-02", "u5,2026-02", "u1,2026-03", "u4,2026-03", "u6,2026-03",
        "u3,2026-04", "u4,2026-04", "u6,2026-04", "u7,2026-04", "u1,2026-04",
        "u5,2026-05", "u7,2026-05", "u2,2025-12", "bogus", "u9,2026-13",
        "u1,2026-02",
    ]


def parse(line):
    cells = line.split(",")
    if len(cells) != 2:
        return None
    ym = cells[1].split("-")
    if len(ym) != 2:
        return None
    try:
        year = int(ym[0])
        mon = int(ym[1])
    except ValueError:
        return None
    if mon < 1 or mon > 12:
        return None
    return (cells[0], year * 12 + mon - 1)


def label(month):
    mon = month % 12 + 1
    return f"{month // 12}-{mon:0>2}"


def main():
    events = []
    for line in data():
        ev = parse(line)
        if ev is None:
            print(f"skipped: {line}")
            continue
        events.append(ev)
    born = {}
    seen = set()
    for e in events:
        cur = born.get(e[0], 999999)
        if e[1] < cur:
            born[e[0]] = e[1]
        seen.add(f"{e[0]}|{e[1]}")
    size = {}
    kept = {}
    for u in born:
        start = born[u]
        size[start] = size.get(start, 0) + 1
        for off in range(4):
            if f"{u}|{start + off}" in seen:
                k = f"{start}|{off}"
                kept[k] = kept.get(k, 0) + 1
    print("cohort   users  +0   +1   +2   +3")
    for start in sorted(size):
        row = f"{label(start)}  {size[start]:>5}"
        for off in range(4):
            n = kept.get(f"{start}|{off}", 0)
            pct = n * 100 // size[start]
            row = f"{row} {pct:>3}%"
        print(row)


main()
