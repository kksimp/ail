ROWS = 6
COLS = 4


class SeatError(Exception):
    pass


def fare(row, window):
    if row <= 2:
        return 175 if window else 150
    return 90 if window else 80


class Cabin:
    def __init__(self):
        self.seats = [""] * (ROWS * COLS)
        self.booked = 0

    def book(self, who, code):
        n = len(code)
        if n < 2:
            raise SeatError(f"bad seat code {code}")
        try:
            row = int(code[:n - 1])
        except ValueError:
            raise SeatError(f"bad seat code {code}")
        col = {"A": 0, "B": 1, "C": 2, "D": 3}.get(code[n - 1], -1)
        if col < 0:
            raise SeatError(f"bad seat code {code}")
        if row < 1 or row > ROWS:
            raise SeatError(f"{code} is not in this cabin")
        idx = (row - 1) * COLS + col
        if self.seats[idx] != "":
            raise SeatError(f"{code} already taken by {self.seats[idx]}")
        self.seats[idx] = who
        self.booked += 1
        window = col == 0 or col == COLS - 1
        return fare(row, window)

    def render(self):
        lines = []
        for r in range(ROWS):
            cells = []
            for c in range(COLS):
                who = self.seats[r * COLS + c]
                cells.append("." if who == "" else who[0])
            lines.append(f"{r + 1:>2} {cells[0]} {cells[1]} | {cells[2]} {cells[3]}")
        return lines


bookings = [("Ann", "1A"), ("Bob", "1A"), ("Cy", "2D"), ("Dee", "7B"), ("Eve", "3E"),
            ("Fay", "x"), ("Gus", "4B"), ("Hal", "6C"), ("Ivy", "6D")]
cabin = Cabin()
revenue = 0
for who, code in bookings:
    try:
        f = cabin.book(who, code)
    except SeatError as e:
        print(f"{who}: {e}")
        continue
    print(f"{who}: {code} for {f}")
    revenue += f
for line in cabin.render():
    print(line)
print(f"booked {cabin.booked}, revenue {revenue}")
