def readings():
    return [100, 104, 99, 120, 130, 125, 90, 85, 95, 140, 150, 148, 110, 105, 102]


def sma(xs, w):
    out = []
    total = 0
    for i in range(len(xs)):
        total += xs[i]
        if i >= w:
            total -= xs[i - w]
        if i + 1 >= w:
            out.append(total // w)
    return out


def ema(xs):
    out = []
    cur = xs[0]
    for x in xs:
        cur = (cur * 3 + x) // 4
        out.append(cur)
    return out


def crossings(xs, trend):
    out = []
    for i in range(1, len(xs)):
        was_above = xs[i - 1] > trend[i - 1]
        now_above = xs[i] > trend[i]
        if was_above and not now_above:
            out.append(f"day {i} down")
        elif now_above and not was_above:
            out.append(f"day {i} up")
    return out


def main():
    xs = readings()
    w = 3
    avg = sma(xs, w)
    trend = ema(xs)
    print(f"day value sma{w} ema")
    for i in range(len(xs)):
        cell = "-" if i + 1 < w else f"{avg[i + 1 - w]}"
        e = trend[i]
        v = xs[i]
        print(f"{i:>3} {v:>5} {cell:>4} {e:>4}")
    for c in crossings(xs, trend):
        print(c)
    print(f"last sma {avg[-1]}, last ema {trend[-1]}")


main()
