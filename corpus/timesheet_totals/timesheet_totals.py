def data():
    return [
        "ann,2026-03-02,08:30,17:00",
        "bob,2026-03-02,09:00,13:15",
        "ann,2026-03-03,08:00,18:30",
        "bob,2026-03-03,10:00,09:00",
        "cy,2026-03-03,07:45,16:15",
        "bob,2026-03-04,09:00,17:30",
        "ann,2026-03-04,12:00,xx:00",
        "cy,2026-03-04,08:00,12:00",
        "ann,2026-03-05,08:00,16:00",
    ]


def clock(text):
    parts = text.split(":")
    if len(parts) != 2:
        return None
    try:
        h = int(parts[0])
        m = int(parts[1])
    except ValueError:
        return None
    if h < 0 or h > 23 or m < 0 or m > 59:
        return None
    return h * 60 + m


def parse(line):
    c = line.split(",")
    if len(c) != 4:
        return None
    start = clock(c[2])
    if start is None:
        return None
    stop = clock(c[3])
    if stop is None:
        return None
    if stop <= start:
        return None
    span = stop - start
    if span > 360:
        span -= 30
    return (c[0], c[1], span)


def hm(minutes):
    return f"{minutes // 60}:{minutes % 60:0>2}"


def main():
    shifts = []
    for line in data():
        sh = parse(line)
        if sh is None:
            print(f"rejected: {line}")
            continue
        shifts.append(sh)
    total = {}
    overtime = {}
    days = {}
    for s in shifts:
        total[s[0]] = total.get(s[0], 0) + s[2]
        days[s[0]] = days.get(s[0], 0) + 1
        if s[2] > 480:
            overtime[s[0]] = overtime.get(s[0], 0) + s[2] - 480
    print("name  days   total  regular  overtime")
    everyone = 0
    for n in sorted(total):
        t = total[n]
        ot = overtime.get(n, 0)
        everyone += t
        print(f"{n:<5} {days[n]:>4} {hm(t):>7} {hm(t - ot):>8} {hm(ot):>9}")
    print(f"all   {hm(everyone)}")


main()
