"""Flight connection finder."""


class RouteError(Exception):
    pass


def parse_time(s):
    if len(s) != 4:
        raise RouteError("bad flight data '%s'" % s)
    try:
        hh = int(s[0:2])
        mm = int(s[2:4])
    except ValueError:
        raise RouteError("bad flight data '%s'" % s)
    if hh > 23 or mm > 59:
        raise RouteError("bad flight data '%s'" % s)
    return hh * 60 + mm


def parse_flight(line):
    p = line.split()
    if len(p) != 6:
        raise RouteError("bad flight data '%s'" % line)
    try:
        cost = int(p[5])
    except ValueError:
        raise RouteError("bad flight data '%s'" % line)
    dep = parse_time(p[3])
    arr = parse_time(p[4])
    return {"id": p[0], "src": p[1], "dst": p[2], "dep": dep, "arr": arr, "cost": cost}


def known(flights, code):
    return any(f["src"] == code or f["dst"] == code for f in flights)


def clock(t):
    return "%02d:%02d" % (t // 60, t % 60)


def search(flights, src, dst, max_legs):
    for code in (src, dst):
        if not known(flights, code):
            raise RouteError("unknown airport %s" % code)
    found = []
    frontier = [{"here": src, "arrive": 0, "ids": [], "price": 0, "visited": [src], "legs": 0}]
    rounds = 0
    while rounds < max_legs and frontier:
        grown = []
        for t in frontier:
            for f in flights:
                board = t["legs"] == 0 or f["dep"] >= t["arrive"] + 45
                if f["src"] == t["here"] and board and f["dst"] not in t["visited"]:
                    n = {
                        "here": f["dst"], "arrive": f["arr"], "ids": t["ids"] + [f["id"]],
                        "price": t["price"] + f["cost"], "visited": t["visited"] + [f["dst"]],
                        "legs": t["legs"] + 1,
                    }
                    if n["here"] == dst:
                        found.append(n)
                    else:
                        grown.append(n)
        frontier = grown
        rounds += 1
    if not found:
        raise RouteError("no route %s -> %s" % (src, dst))
    return found


def describe(t):
    return "%s cost %d arrive %s (%d legs)" % ("+".join(t["ids"]), t["price"], clock(t["arrive"]), t["legs"])


def main():
    lines = [
        "UA1 SFO DEN 0800 1100 150", "UA2 DEN JFK 1200 1600 200", "UA3 SFO ORD 0700 1300 300",
        "UA4 ORD JFK 1400 1700 120", "UA5 SFO JFK 0900 1700 520", "UA6 DEN ORD 1130 1400 90",
        "UA7 DEN MIA 1245 1630 180", "UA8 MIA JFK 1800 2100 100", "UA9 SEA SFO 0600 0800 80",
        "UA10 SEA DEN 0900 1200 140", "UA11 ORD MIA 1445 1900 150", "UA99 SFO LAX 25xx 1000 50",
        "UA98 SFO LAX 0900", "UA97 SFO LAX 0900 1000 abc",
    ]
    flights = []
    for line in lines:
        try:
            flights.append(parse_flight(line))
        except RouteError as err:
            print("skip: %s" % err)
    print("%d flights loaded" % len(flights))
    queries = [("SFO", "JFK"), ("SEA", "MIA"), ("SEA", "JFK"), ("JFK", "SFO"), ("SFO", "XXX")]
    for q in queries:
        try:
            trips = search(flights, q[0], q[1], 3)
        except RouteError as err:
            print("%s -> %s: %s" % (q[0], q[1], err))
            continue
        cheapest = sorted(trips, key=lambda t: t["price"] * 10000 + t["arrive"])
        earliest = sorted(trips, key=lambda t: t["arrive"] * 10000 + t["price"])
        print("%s -> %s: %d itineraries" % (q[0], q[1], len(trips)))
        print("  cheapest: %s" % describe(cheapest[0]))
        print("  earliest: %s" % describe(earliest[0]))


main()
