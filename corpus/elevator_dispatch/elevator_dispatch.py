FLOORS = 12


class DispatchError(Exception):
    pass


class Car:
    def __init__(self, cid, floor):
        self.id = cid
        self.floor = floor
        self.stops = []
        self.travelled = 0
        self.served = 0

    def step(self):
        if not self.stops:
            return False
        goal = self.stops[0]
        if self.floor < goal:
            self.floor += 1
            self.travelled += 1
        elif self.floor > goal:
            self.floor -= 1
            self.travelled += 1
        if self.floor == goal:
            self.stops.pop(0)
            self.served += 1
            return True
        return False


def nearest(car, call):
    return abs(car.floor - call)


def least_loaded(car, call):
    return len(car.stops) * 4 + abs(car.floor - call)


def dispatch(cars, call, score):
    if call < 0 or call >= FLOORS:
        raise DispatchError(f"floor {call} is not in the building")
    best = 0
    best_score = score(cars[0], call)
    for i in range(1, len(cars)):
        s = score(cars[i], call)
        if s < best_score:
            best = i
            best_score = s
    if call not in cars[best].stops:
        cars[best].stops.append(call)
    return cars[best].id


def simulate(name, score, calls):
    print(f"policy {name}")
    cars = [Car(0, 0), Car(1, 6), Car(2, 11)]
    for t in range(24):
        for call in calls:
            if call[0] == t:
                try:
                    who = dispatch(cars, call[1], score)
                except DispatchError as e:
                    print(f"  t={t} rejected: {e}")
                    continue
                print(f"  t={t} floor {call[1]} -> car {who}")
        for car in cars:
            if car.step():
                print(f"  t={t} car {car.id} arrives at {car.floor}")
    total = sum(c.travelled for c in cars)
    served = sum(c.served for c in cars)
    print(f"  served {served}, travelled {total}")
    return total


calls = [[0, 5], [1, 9], [2, 14], [3, 1], [4, 7], [6, 11], [8, 0], [9, 4], [12, 6], [13, 3]]
a = simulate("nearest", nearest, calls)
b = simulate("least-loaded", least_loaded, calls)
print(f"difference {a - b}")
