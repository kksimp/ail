NAMES = ["d", "h", "m", "s"]
SIZES = [86400, 3600, 60, 1]


def format_long(total):
    if total == 0:
        return "0s"
    rest = total
    parts = []
    for i in range(len(NAMES)):
        n = rest // SIZES[i]
        rest = rest % SIZES[i]
        if n > 0:
            parts.append("%d%s" % (n, NAMES[i]))
    return " ".join(parts)


def format_short(total):
    return " ".join(format_long(total).split(" ")[:2])


def format_clock(total):
    h = total // 3600
    m = (total % 3600) // 60
    s = total % 60
    return "%02d:%02d:%02d" % (h, m, s)


def unit_seconds(c):
    return {"d": 86400, "h": 3600, "m": 60, "s": 1}.get(c, 0)


def parse_duration(text):
    total = 0
    num = ""
    for c in text:
        if c.isspace():
            continue
        if c.isdigit():
            num += c
            continue
        size = unit_seconds(c)
        if size == 0 or num == "":
            return None
        total += int(num) * size
        num = ""
    if num != "":
        return None
    return total


def main():
    for t in [0, 7, 59, 60, 61, 3599, 3600, 3661, 86399, 86400, 90061, 1000000]:
        print("%8d  %-14s %-7s %s" % (t, format_long(t), format_short(t), format_clock(t)))
    for text in ["1h30m", "2d 4h", "90s", "1h 5", "5x", "m", "1d1h1m1s", ""]:
        v = parse_duration(text)
        if v is None:
            print("%-10s invalid" % text)
        else:
            print("%-10s %d" % (text, v))


main()
