class SensorError(Exception):
    pass


def tenths(n):
    return f"{n // 10}.{n % 10}"


def read_sensor(hour, actual):
    raw = actual + 700 if hour == 9 else actual
    if raw < 0 or raw > 450:
        raise SensorError(f"sensor reading {raw} out of range")
    return raw


def drift(outside, temp):
    d = outside - temp
    if d >= 0:
        return d // 10
    return -((-d) // 10)


class Controller:
    def __init__(self, band):
        self.mode = "idle"
        self.band = band
        self.switches = 0
        self.heat_hours = 0
        self.cool_hours = 0

    def update(self, temp, goal):
        nm = self.mode
        if temp <= goal - self.band:
            nm = "heating"
        elif temp >= goal + self.band:
            nm = "cooling"
        elif self.mode == "heating" and temp >= goal:
            nm = "idle"
        elif self.mode == "cooling" and temp <= goal:
            nm = "idle"
        changed = nm != self.mode
        if changed:
            self.switches += 1
            self.mode = nm
        if self.mode == "heating":
            self.heat_hours += 1
        elif self.mode == "cooling":
            self.cool_hours += 1
        return changed


def weekday(h):
    return 210 if 6 <= h < 22 else 180


def weekend(h):
    return 195


def simulate(name, sched):
    print(f"schedule {name}")
    ctl = Controller(5)
    temp = 170
    for h in range(24):
        goal = sched(h)
        outside = 100 + 10 * (h if h < 12 else 24 - h)
        try:
            seen = read_sensor(h, temp)
        except SensorError as e:
            print(f"  h={h:0>2} {e}; holding {ctl.mode}")
            temp += drift(outside, temp)
            continue
        if ctl.update(seen, goal):
            print(f"  h={h:0>2} temp {tenths(seen)} goal {tenths(goal)} -> {ctl.mode}")
        temp += drift(outside, temp)
        if ctl.mode == "heating":
            temp += 25
        elif ctl.mode == "cooling":
            temp -= 25
    print(f"  switches {ctl.switches}, heat {ctl.heat_hours}h, cool {ctl.cool_hours}h, final {tenths(temp)}")
    return ctl


a = simulate("weekday", weekday)
b = simulate("weekend", weekend)
print(f"total switches {a.switches + b.switches}")
