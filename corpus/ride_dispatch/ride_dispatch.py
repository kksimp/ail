"""Ride dispatch simulation on a 20x20 grid."""


class DispatchError(Exception):
    pass


class Flat:
    def __init__(self, base, per):
        self.base = base
        self.per = per

    def label(self):
        return "flat"

    def fare(self, dist, when):
        return self.base + self.per * dist


class Surge:
    def __init__(self, base, per, pct, first, last):
        self.base, self.per, self.pct, self.first, self.last = base, per, pct, first, last

    def label(self):
        return "surge %d%%" % self.pct

    def fare(self, dist, when):
        plain = self.base + self.per * dist
        if self.first <= when <= self.last:
            return plain * self.pct // 100
        return plain


def fleet():
    return [dict(id=1, x=2, y=2, free_at=0, trips=0, earned=0),
            dict(id=2, x=10, y=10, free_at=0, trips=0, earned=0),
            dict(id=3, x=18, y=3, free_at=0, trips=0, earned=0)]


class Dispatcher:
    def __init__(self, drivers, pricing, zone):
        self.drivers = drivers
        self.pricing = pricing
        self.zone = zone

    def assign(self, line):
        p = line.split()
        if len(p) != 6:
            raise DispatchError("bad request '%s'" % line)
        v = []
        for k in [0, 2, 3, 4, 5]:
            try:
                v.append(int(p[k]))
            except ValueError:
                raise DispatchError("bad request '%s'" % line)
        for k in range(1, 5):
            if v[k] < 0 or v[k] >= self.zone:
                raise DispatchError("%s is outside the service zone" % p[1])
        trip = abs(v[3] - v[1]) + abs(v[4] - v[2])
        if trip == 0:
            raise DispatchError("bad request '%s'" % line)
        best = -1
        best_d = 0
        for i, d in enumerate(self.drivers):
            gap = abs(d["x"] - v[1]) + abs(d["y"] - v[2])
            if d["free_at"] <= v[0] and (best < 0 or gap < best_d):
                best = i
                best_d = gap
        if best < 0:
            raise DispatchError("no free driver for %s at t=%d" % (p[1], v[0]))
        d = self.drivers[best]
        cost = self.pricing.fare(trip, v[0])
        self.drivers[best] = dict(id=d["id"], x=v[3], y=v[4], free_at=v[0] + best_d + trip,
                                  trips=d["trips"] + 1, earned=d["earned"] + cost)
        return "t=%d %s -> driver %d, pickup %d, trip %d, free at %d, fare %d" % (
            v[0], p[1], d["id"], best_d, trip, v[0] + best_d + trip, cost)


def run(pricing, requests):
    print("== %s" % pricing.label())
    disp = Dispatcher(fleet(), pricing, 20)
    for r in requests:
        try:
            print(disp.assign(r))
        except DispatchError as err:
            print("! %s" % err)
    for d in disp.drivers:
        print("driver %d: %d trips, earned %d, at %d,%d" % (d["id"], d["trips"], d["earned"], d["x"], d["y"]))


def main():
    requests = ["0 ann 3 3 8 9", "2 bob 9 9 1 1", "3 cy 17 2 17 15", "4 dee 5 5 6 6", "15 eli 8 9 2 2",
                "16 fay 40 1 2 2", "17 gus 1 1 x 2", "21 hal 1 1 5 5", "22 ivy 2 2 2 2"]
    run(Flat(300, 120), requests)
    run(Surge(300, 120, 150, 14, 20), requests)


main()
