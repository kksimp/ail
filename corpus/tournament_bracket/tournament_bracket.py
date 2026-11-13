"""Single-elimination tournament bracket."""


class BracketError(Exception):
    pass


def parse_teams(lines):
    raw = []
    names = set()
    for line in lines:
        p = line.split()
        if len(p) != 2:
            raise BracketError("bad team line '%s'" % line)
        try:
            r = int(p[1])
        except ValueError:
            raise BracketError("bad team line '%s'" % line)
        if p[0] in names:
            raise BracketError("team %s listed twice" % p[0])
        names.add(p[0])
        raw.append((p[0], r))
    if len(raw) < 2:
        raise BracketError("need at least 2 teams, got %d" % len(raw))
    ranked = sorted(raw, key=lambda t: -t[1])
    return [(name, i + 1, rating) for i, (name, rating) in enumerate(ranked)]


def seed_order(size):
    order = [1]
    while len(order) < size:
        total = len(order) * 2
        grown = []
        for s in order:
            grown.append(s)
            grown.append(total + 1 - s)
        order = grown
    return order


def chalk(fav, other):
    return True


def underdog(bonus):
    return lambda fav, other: other[2] + bonus <= fav[2]


def play(teams, pick):
    size = 1
    while size < len(teams):
        size *= 2
    slots = [s - 1 if s <= len(teams) else -1 for s in seed_order(size)]
    log = []
    rnd = 1
    while len(slots) > 1:
        winners = []
        for k in range(len(slots) // 2):
            a = slots[2 * k]
            b = slots[2 * k + 1]
            if b < 0 or a < 0:
                w = b if a < 0 else a
                log.append("R%d: %s has a bye" % (rnd, teams[w][0]))
                winners.append(w)
                continue
            lo = min(a, b)
            hi = max(a, b)
            champ = lo if pick(teams[lo], teams[hi]) else hi
            log.append("R%d: %s beats %s" % (rnd, teams[champ][0], teams[lo + hi - champ][0]))
            winners.append(champ)
        slots = winners
        rnd += 1
    log.append("champion: %s" % teams[slots[0]][0])
    return log


def main():
    lines = ["Lions 88", "Tigers 84", "Bears 91", "Wolves 77", "Hawks 84", "Eagles 70", "Sharks 65"]
    try:
        teams = parse_teams(lines)
    except BracketError as err:
        print("error: %s" % err)
        return
    for name, seed, rating in teams:
        print("seed %d: %s (%d)" % (seed, name, rating))
    rules = [("chalk bracket", chalk), ("underdog +0", underdog(0)), ("underdog +8", underdog(8))]
    for label, pick in rules:
        print("-- %s" % label)
        for row in play(teams, pick):
            print(row)
    for bad in [["Solo 5"], ["A 1", "A 2"], ["A 1", "B x"]]:
        try:
            parse_teams(bad)
        except BracketError as err:
            print("error: %s" % err)


main()
