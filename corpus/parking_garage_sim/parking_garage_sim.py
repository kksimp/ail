"""Parking garage simulation."""


class GarageError(Exception):
    pass


def kind_of(plate):
    return "Moto" if plate.startswith("M") else "Car"


def fee(minutes, kind):
    billable = minutes - 30
    if billable <= 0:
        return 0
    hours = (billable + 59) // 60
    full = min(hours * 200, 1500)
    return full if kind == "Car" else full // 2


def money(cents):
    return "%d.%02d" % (cents // 100, cents % 100)


class Garage:
    def __init__(self, capacity):
        self.capacity = capacity
        self.inside = {}
        self.revenue = 0
        self.served = 0

    def apply(self, line):
        parts = line.split()
        bad = GarageError("bad event '%s'" % line)
        if len(parts) != 3:
            raise bad
        try:
            at = int(parts[2])
        except ValueError:
            raise bad
        plate = parts[1]
        if parts[0] == "in":
            if plate in self.inside:
                raise GarageError("%s is already inside" % plate)
            if len(self.inside) >= self.capacity:
                raise GarageError("garage full")
            self.inside[plate] = at
            return "%s enters at %d" % (plate, at)
        if parts[0] != "out":
            raise bad
        if plate not in self.inside:
            raise GarageError("%s is not inside" % plate)
        since = self.inside.pop(plate)
        charge = fee(at - since, kind_of(plate))
        self.revenue += charge
        self.served += 1
        return "%s leaves after %d min, pays %s" % (plate, at - since, money(charge))


def main():
    events = [
        "in CAR1 0", "in CAR2 5", "in M77 10", "in CAR1 12", "in CAR3 15",
        "out CAR2 20", "in CAR3 21", "out CAR1 100", "out M77 250", "out XYZ 260",
        "park CAR4 270", "in CAR4 abc", "out CAR3 1000",
    ]
    g = Garage(3)
    for ev in events:
        try:
            print(g.apply(ev))
        except GarageError as err:
            print("rejected '%s': %s" % (ev, err))
    print("served %d, still inside %d, revenue %s" % (g.served, len(g.inside), money(g.revenue)))


main()
