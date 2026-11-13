class BookingError(Exception):
    pass


class Hotel:
    def __init__(self, rooms):
        self.rooms = rooms
        self.bookings = []
        self.next_id = 1

    def book(self, room, start, stop):
        if room not in self.rooms:
            raise BookingError(f"no room {room}")
        if start < 1 or stop <= start:
            raise BookingError(f"bad stay {start}-{stop}")
        for b in self.bookings:
            if b[1] == room and start < b[3] and b[2] < stop:
                raise BookingError(f"room {room} is held by booking {b[0]}")
        bid = self.next_id
        self.next_id += 1
        self.bookings.append((bid, room, start, stop))
        return bid

    def cancel(self, bid):
        at = -1
        for i in range(len(self.bookings)):
            if self.bookings[i][0] == bid:
                at = i
        if at < 0:
            raise BookingError(f"no booking {bid}")
        del self.bookings[at]

    def nights(self, room):
        total = 0
        for b in self.bookings:
            if b[1] == room:
                total += b[3] - b[2]
        return total

    def row(self, room, days):
        cells = []
        for d in range(1, days + 1):
            busy = False
            for b in self.bookings:
                if b[1] == room and b[2] <= d and d < b[3]:
                    busy = True
            cells.append("#" if busy else ".")
        return "".join(cells)

    def run(self, line):
        parts = line.split(" ")
        if parts[0] == "book" and len(parts) == 4:
            try:
                a = int(parts[2])
                z = int(parts[3])
            except ValueError:
                raise BookingError(f"bad command '{line}'")
            bid = self.book(parts[1], a, z)
            return f"booked #{bid}"
        if parts[0] == "cancel" and len(parts) == 2:
            try:
                n = int(parts[1])
            except ValueError:
                raise BookingError(f"bad command '{line}'")
            self.cancel(n)
            return f"cancelled #{n}"
        raise BookingError(f"bad command '{line}'")


def main():
    hotel = Hotel(["101", "102", "201"])
    script = [
        "book 101 1 4",
        "book 101 4 6",
        "book 101 3 5",
        "book 102 2 3",
        "book 102 0 3",
        "book 102 5 5",
        "book 301 1 2",
        "book 201 2 9",
        "cancel 1",
        "book 101 2 4",
        "cancel 42",
        "stay 101",
    ]
    for line in script:
        try:
            msg = hotel.run(line)
        except BookingError as err:
            print(f"{line}: REJECTED {err}")
            continue
        print(f"{line}: {msg}")
    for room in hotel.rooms:
        print(f"{room} {hotel.row(room, 10)} {hotel.nights(room)} nights")


main()
