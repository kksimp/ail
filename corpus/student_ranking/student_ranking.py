def data():
    return [
        "ana,math,92",
        "ben,math,75",
        "cy,math,88",
        "ana,art,85",
        "dee,math,75",
        "ben,art,80",
        "cy,art,80",
        "eli,math,60",
        "dee,art,80",
        "eli,art,x",
        "ana,music,90",
        "ben,music,70",
        "cy,music,97",
        "fay,music",
    ]


def collect(lines):
    totals = {}
    counts = {}
    for line in lines:
        cells = line.split(",")
        if len(cells) != 3:
            continue
        try:
            score = int(cells[2])
        except ValueError:
            continue
        totals[cells[0]] = totals.get(cells[0], 0) + score
        counts[cells[0]] = counts.get(cells[0], 0) + 1
    out = []
    for name in sorted(totals):
        t = totals[name]
        c = counts[name]
        out.append((name, t, c, t * 10 // c))
    return out


def rank_all(st):
    st = sorted(st, key=lambda s: s[0])
    return sorted(st, key=lambda s: -s[3])


def fmt_tenths(t):
    return f"{t // 10}.{t % 10}"


def main():
    ranked = rank_all(collect(data()))
    print("rank name  exams  avg")
    rank = 0
    prev = -1
    for i in range(len(ranked)):
        s = ranked[i]
        if s[3] != prev:
            rank = i + 1
            prev = s[3]
        avg = fmt_tenths(s[3])
        print(f"{rank:>4} {s[0]:<5} {s[2]:>5} {avg:>5}")
    best = ranked[0]
    print(f"top: {best[0]} with total {best[1]}")


main()
