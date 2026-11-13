FLOORS = 10


class Elevator:
    def __init__(self):
        self.level = 0
        self.heading = 0
        self.waiting = [-1] * FLOORS
        self.served = 0
        self.wait_total = 0
        self.moved = 0

    def ahead(self, d):
        f = self.level + d
        while f >= 0 and f < FLOORS:
            if self.waiting[f] >= 0:
                return True
            f += d
        return False

    def choose(self):
        up = self.ahead(1)
        down = self.ahead(-1)
        if self.heading == 1 and up:
            return 1
        if self.heading == -1 and down:
            return -1
        if up:
            return 1
        if down:
            return -1
        return 0

    def tick(self, t):
        if self.waiting[self.level] >= 0:
            w = t - self.waiting[self.level]
            self.wait_total += w
            self.served += 1
            self.waiting[self.level] = -1
            print(f"t={t} stop at floor {self.level} (waited {w})")
            return
        self.heading = self.choose()
        self.level += self.heading
        self.moved += abs(self.heading)


def main():
    requests = [
        (0, 5), (1, 2), (3, 8), (4, 3), (12, 0), (14, 9), (15, 4), (22, 6),
    ]
    car = Elevator()
    for t in range(40):
        for at, floor in requests:
            if at == t and car.waiting[floor] < 0:
                car.waiting[floor] = t
        car.tick(t)
    print(f"served: {car.served}")
    print(f"floors travelled: {car.moved}")
    print(f"total wait: {car.wait_total}")
    print(f"average wait x10: {car.wait_total * 10 // car.served}")
    print(f"ended at floor {car.level}")


main()
