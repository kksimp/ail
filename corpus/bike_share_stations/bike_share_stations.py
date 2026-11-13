class RideError(Exception):
    pass


def demo():
    return [["Park", 10, 9], ["Depot", 8, 0], ["Library", 6, 6], ["Mall", 10, 1], ["Pier", 8, 4]]


class Network:
    def __init__(self):
        self.stations = demo()
        self.completed = 0

    def index(self, name):
        for i, s in enumerate(self.stations):
            if s[0] == name:
                return i
        raise RideError(f"unknown station {name}")

    def ride(self, src, dst):
        a = self.index(src)
        b = self.index(dst)
        if self.stations[a][2] == 0:
            raise RideError(f"no bikes at {src}")
        if a != b and self.stations[b][2] >= self.stations[b][1]:
            raise RideError(f"{dst} is full")
        self.stations[a][2] -= 1
        self.stations[b][2] += 1
        self.completed += 1
        return self.completed

    def apply(self, plan):
        for src, dst, count in plan:
            self.stations[src][2] -= count
            self.stations[dst][2] += count

    def line(self):
        return " ".join(f"{s[0]}:{s[2]}/{s[1]}" for s in self.stations)

    def copy(self):
        n = Network()
        n.stations = [list(s) for s in self.stations]
        n.completed = self.completed
        return n


def target_moves(stations, pct):
    give, take = [], []
    for s in stations:
        goal = s[1] * pct // 100
        give.append(max(s[2] - goal, 0))
        take.append(max(goal - s[2], 0))
    out = []
    for d in range(len(stations)):
        for o in range(len(stations)):
            n = min(give[o], take[d])
            if n > 0:
                give[o] -= n
                take[d] -= n
                out.append((o, d, n))
    return out


def rescue_moves(stations, floor, lift):
    counts = [s[2] for s in stations]
    out = []
    for d in range(len(counts)):
        if counts[d] < floor:
            donor = 0
            for o in range(len(counts)):
                if counts[o] > counts[donor]:
                    donor = o
            n = min(lift, counts[donor] - floor)
            if n > 0 and donor != d:
                counts[donor] -= n
                counts[d] += n
                out.append((donor, d, n))
    return out


rides = [["Park", "Mall"], ["Depot", "Park"], ["Pier", "Library"], ["Mall", "Zoo"],
         ["Park", "Pier"], ["Mall", "Mall"], ["Mall", "Depot"], ["Mall", "Depot"],
         ["Library", "Park"]]
net = Network()
for r in rides:
    try:
        n = net.ride(r[0], r[1])
    except RideError as e:
        print(f"ride {r[0]}->{r[1]} rejected: {e}")
        continue
    print(f"ride {n}: {r[0]}->{r[1]}")
print(f"after rides: {net.line()}")
policies = [("target 50%", lambda st: target_moves(st, 50)),
            ("rescue below 2", lambda st: rescue_moves(st, 2, 3))]
for label, fn in policies:
    cp = net.copy()
    plan = fn(cp.stations)
    cp.apply(plan)
    print(f"{label}: {len(plan)} moves")
    for src, dst, count in plan:
        print(f"  move {count} from {net.stations[src][0]} to {net.stations[dst][0]}")
    print(f"  result: {cp.line()}")
