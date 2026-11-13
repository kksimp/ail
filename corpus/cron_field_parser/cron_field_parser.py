class CronError(Exception):
    pass


def split_once(s, sep):
    i = s.find(sep)
    if i < 0:
        return (s, "")
    return (s[:i], s[i + len(sep):])


def to_int(t):
    try:
        return int(t)
    except ValueError:
        return None


def parse_value(t, lo, hi, names):
    v = lo
    idx = names.index(t.upper()) if t.upper() in names else -1
    if idx >= 0:
        v = lo + idx
    else:
        n = to_int(t)
        if n is None:
            raise CronError("bad value " + t)
        v = n
    if v < lo or v > hi:
        raise CronError("%d is outside %d-%d" % (v, lo, hi))
    return v


def parse_field(text, lo, hi, names):
    seen = set()
    for part in text.split(","):
        base, step_text = split_once(part, "/")
        step = 1
        if step_text != "":
            s = to_int(step_text)
            if s is None:
                raise CronError("bad step " + step_text)
            if s <= 0:
                raise CronError("bad step " + step_text)
            step = s
        lowest = lo
        highest = hi
        if base != "*":
            a, b = split_once(base, "-")
            lowest = parse_value(a, lo, hi, names)
            if b == "":
                highest = hi if step_text != "" else lowest
            else:
                highest = parse_value(b, lo, hi, names)
                if highest < lowest:
                    raise CronError("descending range " + base)
        v = lowest
        while v <= highest:
            seen.add(v)
            v += step
    return sorted(seen)


def parse_cron(expr):
    parts = expr.split()
    if len(parts) != 5:
        raise CronError("expected 5 fields, got %d" % len(parts))
    month_names = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"]
    day_names = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]
    f_min = parse_field(parts[0], 0, 59, [])
    f_hour = parse_field(parts[1], 0, 23, [])
    f_day = parse_field(parts[2], 1, 31, [])
    f_month = parse_field(parts[3], 1, 12, month_names)
    f_week = parse_field(parts[4], 0, 6, day_names)
    return (f_min, f_hour, f_day, f_month, f_week)


def fires(s, month, dom, dow, hour, minute):
    return month in s[3] and dom in s[2] and dow in s[4] and hour in s[1] and minute in s[0]


def brief(xs):
    if len(xs) <= 8:
        return ",".join(str(x) for x in xs)
    return ",".join(str(x) for x in xs[:6]) + ",... (%d values)" % len(xs)


def show(expr):
    print(expr)
    try:
        s = parse_cron(expr)
    except CronError as e:
        print(f"  error: {e}")
        return
    print(f"  minute   {brief(s[0])}")
    print(f"  hour     {brief(s[1])}")
    print(f"  day      {brief(s[2])}")
    print(f"  month    {brief(s[3])}")
    print(f"  weekday  {brief(s[4])}")
    print(f"  runs per day: {len(s[0]) * len(s[1])}")


show("*/15 9-17 * * MON-FRI")
show("0 0 1,15 JAN-JUN *")
show("5,10-12,50/5 */6 * * 0")
show("30 2 * * 7")
show("61 * * * *")
show("*/0 * * * *")
show("10-5 * * * *")
show("x * * * *")
show("* * * *")
s = parse_cron("*/15 9-17 * * MON-FRI")
print("Wed 09:30 fires: " + str(fires(s, 6, 12, 3, 9, 30)).lower())
print("Wed 09:31 fires: " + str(fires(s, 6, 12, 3, 9, 31)).lower())
print("Sun 09:30 fires: " + str(fires(s, 6, 12, 0, 9, 30)).lower())
