"""Library overdue fines."""


class FineError(Exception):
    pass


POLICIES = [
    ("book", 21, 25, 1000),
    ("dvd", 7, 100, 2000),
    ("magazine", 14, 10, 200),
]


def money(cents):
    return "%d.%02d" % (cents // 100, cents % 100)


def assess(line):
    parts = line.split()
    if len(parts) != 4:
        raise FineError("bad record '%s'" % line)
    try:
        out_day = int(parts[2])
        in_day = int(parts[3])
    except ValueError:
        raise FineError("bad record '%s'" % line)
    if in_day < out_day:
        raise FineError("%s returned before it was borrowed" % parts[1])
    code = parts[1][0]
    slot = "BDM".find(code)
    if slot < 0:
        raise FineError("unknown item kind %s" % code)
    label, limit, rate, cap = POLICIES[slot]
    held = in_day - out_day
    late = held - limit
    cents = 0 if late <= 0 else min(late * rate, cap)
    print("%s %s (%s) held %d, fine %s" % (parts[0], parts[1], label, held, money(cents)))
    return cents


def main():
    records = [
        "alice B101 1 20", "alice D7 3 15", "bob B102 1 40", "bob M9 2 30",
        "carol D8 10 80", "carol X1 5 9", "dave B103 30 25", "erin B104 5",
        "erin M3 1 15", "alice M4 2 24",
    ]
    totals = {}
    for rec in records:
        try:
            cents = assess(rec)
        except FineError as err:
            print("skipped: %s" % err)
            continue
        who = rec.split()[0]
        totals[who] = totals.get(who, 0) + cents
    for who in sorted(totals):
        print("%s owes %s" % (who, money(totals[who])))


main()
