class PickError(Exception):
    pass


def stock():
    return [["bolt", 0, 2, 50], ["nut", 0, 7, 5], ["gear", 1, 1, 8], ["belt", 1, 6, 3],
            ["motor", 2, 3, 2], ["fan", 3, 0, 9], ["lamp", 3, 8, 4]]


def key_snake(aisle, slot):
    if aisle % 2 == 0:
        return aisle * 100 + slot
    return aisle * 100 + (10 - 1 - slot)


def key_asc(aisle, slot):
    return aisle * 100 + slot


def plan(bins, lines, keyfn):
    picks = []
    asked = {}
    for line in lines:
        parts = line.split(":")
        if len(parts) != 2:
            raise PickError(f"bad pick line '{line}'")
        try:
            want = int(parts[1])
        except ValueError:
            raise PickError(f"bad pick line '{line}'")
        at = None
        for i, b in enumerate(bins):
            if b[0] == parts[0]:
                at = i
                break
        if at is None:
            raise PickError(f"unknown sku {parts[0]}")
        asked[parts[0]] = asked.get(parts[0], 0) + want
        have = bins[at][3]
        if asked[parts[0]] > have:
            raise PickError(f"{parts[0]}: have {have}, want {asked[parts[0]]}")
        b = bins[at]
        picks.append({"sku": b[0], "aisle": b[1], "slot": b[2], "qty": want, "key": keyfn(b[1], b[2])})
    for p in picks:
        for b in bins:
            if b[0] == p["sku"]:
                b[3] -= p["qty"]
                break
    return sorted(picks, key=lambda p: p["key"])


def walk(route):
    total = 0
    a = 0
    s = 0
    for p in route:
        total += abs(p["aisle"] - a) * 10 + abs(p["slot"] - s)
        a = p["aisle"]
        s = p["slot"]
    return total + a * 10 + s


def describe(route):
    return " ".join(f"{p['sku']}x{p['qty']}@{p['aisle']}.{p['slot']}" for p in route)


def run(name, keyfn):
    orders = [
        ["lamp:1", "bolt:10", "fan:2", "gear:1"],
        ["motor:1", "nut:2", "belt:1"],
        ["motor:2", "bolt:1"],
        ["fan:1", "widget:1"],
        ["gear:7", "lamp:2", "nut:3"],
        ["belt:x"],
    ]
    bins = stock()
    total = 0
    print(f"strategy {name}")
    for i, order in enumerate(orders):
        try:
            route = plan(bins, order, keyfn)
        except PickError as e:
            print(f"  order {i + 1} rejected: {e}")
            continue
        d = walk(route)
        total += d
        print(f"  order {i + 1}: {describe(route)} (distance {d})")
    print(f"  total distance {total}")
    return total


a = run("snake", key_snake)
b = run("ascending", key_asc)
print(f"snake saves {b - a}")
