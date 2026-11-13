class ParkError(Exception):
    pass


RATE = {"car": 300, "bike": 100, "truck": 800}
CAP = {"car": 2000, "bike": 600, "truck": 4500}


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def parse_kind(text):
    if text in RATE:
        return text
    raise ParkError(f"unknown vehicle '{text}'")


def parse_time(text):
    parts = text.split(":")
    if len(parts) != 2:
        raise ParkError(f"bad time '{text}'")
    try:
        h = int(parts[0])
        m = int(parts[1])
    except ValueError:
        raise ParkError(f"bad time '{text}'")
    if h < 0 or h > 23 or m < 0 or m > 59:
        raise ParkError(f"bad time '{text}'")
    return h * 60 + m


def fee_for(kind, minutes):
    billable = minutes - 15 if minutes > 15 else 0
    hours = (billable + 59) // 60
    raw = hours * RATE[kind]
    if raw > CAP[kind]:
        return CAP[kind]
    return raw


def charge(ticket):
    parts = ticket.split(",")
    if len(parts) != 4:
        raise ParkError(f"malformed ticket '{ticket}'")
    kind = parse_kind(parts[1])
    start = parse_time(parts[2])
    stop = parse_time(parts[3])
    if stop < start:
        raise ParkError(f"{parts[0]} left before arriving")
    return fee_for(kind, stop - start)


def main():
    tickets = [
        "AB123,car,08:00,08:10",
        "CD456,car,08:00,09:16",
        "EF789,bike,10:30,13:30",
        "GH012,truck,06:00,18:45",
        "IJ345,car,07:00,22:59",
        "KL678,boat,09:00,10:00",
        "MN901,car,25:00,10:00",
        "OP234,car,12:00,11:00",
        "QR567,car,12:00",
    ]
    total = 0
    paid = 0
    for t in tickets:
        try:
            fee = charge(t)
        except ParkError as err:
            print(f"{t}: {err}")
            continue
        paid += 1
        total += fee
        print(f"{t}: {money(fee)}")
    print(f"{paid} tickets, takings {money(total)}")


main()
