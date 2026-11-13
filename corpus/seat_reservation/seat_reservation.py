ROW_NAMES = "ABCDEFGH"


class SeatError(Exception):
    pass


class Hall:
    def __init__(self, rows, cols):
        self.rows = rows
        self.cols = cols
        self.seats = [0] * (rows * cols)

    def name(self, row, col):
        return f"{ROW_NAMES[row]}{col + 1}"

    def pick(self, party, label):
        if len(label) < 2:
            raise SeatError(f"no such seat {label}")
        r = ROW_NAMES.find(label[0])
        if r < 0:
            raise SeatError(f"no such seat {label}")
        try:
            c = int(label[1:])
        except ValueError:
            raise SeatError(f"no such seat {label}")
        if r >= self.rows or c < 1 or c > self.cols:
            raise SeatError(f"no such seat {label}")
        at = r * self.cols + c - 1
        if self.seats[at] != 0:
            raise SeatError(f"seat {label} is taken by party {self.seats[at]}")
        self.seats[at] = party

    def block(self, party, size):
        if size < 1 or size > self.cols:
            raise SeatError(f"no block of {size} seats together")
        found = False
        best = 0
        best_row = 0
        best_col = 0
        for r in range(self.rows):
            for c in range(self.cols - size + 1):
                free = True
                for k in range(size):
                    if self.seats[r * self.cols + c + k] != 0:
                        free = False
                if free:
                    score = abs(2 * r - (self.rows - 1)) * 100 + abs(2 * c + size - self.cols)
                    if not found or score < best:
                        found = True
                        best = score
                        best_row = r
                        best_col = c
        if not found:
            raise SeatError(f"no block of {size} seats together")
        for k in range(size):
            self.seats[best_row * self.cols + best_col + k] = party
        return f"{self.name(best_row, best_col)}-{self.name(best_row, best_col + size - 1)}"

    def run(self, line):
        parts = line.split(" ")
        if len(parts) != 3:
            raise SeatError(f"bad command '{line}'")
        try:
            party = int(parts[1])
        except ValueError:
            raise SeatError(f"bad command '{line}'")
        if parts[0] == "pick":
            self.pick(party, parts[2])
            return f"party {party} has {parts[2]}"
        if parts[0] == "block":
            try:
                n = int(parts[2])
            except ValueError:
                raise SeatError(f"bad command '{line}'")
            got = self.block(party, n)
            return f"party {party} has {got}"
        raise SeatError(f"bad command '{line}'")

    def render(self):
        out = []
        for r in range(self.rows):
            cells = []
            for c in range(self.cols):
                v = self.seats[r * self.cols + c]
                cells.append("." if v == 0 else str(v))
            out.append(f"{ROW_NAMES[r]} {''.join(cells)}")
        return out


def main():
    hall = Hall(5, 8)
    script = [
        "block 1 3",
        "block 2 4",
        "pick 3 C4",
        "pick 4 C4",
        "pick 4 Z9",
        "pick 4 C9",
        "block 5 9",
        "block 6 5",
        "block 7 6",
        "block 8 2",
        "sit 1 A1",
    ]
    for line in script:
        try:
            msg = hall.run(line)
        except SeatError as err:
            print(f"{line}: {err}")
            continue
        print(f"{line}: {msg}")
    for row in hall.render():
        print(row)
    free = 0
    for v in hall.seats:
        if v == 0:
            free += 1
    print(f"{free} of {len(hall.seats)} seats free")


main()
