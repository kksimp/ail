class OrderError(Exception):
    pass


MENU = [["steak", 2450, "hot", 3], ["salad", 850, "cold", 10], ["cola", 300, "drink", 4]]
totals = {}


def prep(kind, qty):
    if kind == "hot":
        return f"grill {8 + 3 * qty}m"
    if kind == "cold":
        return f"garde {2 + qty}m"
    return "bar 1m"


def order(line):
    parts = line.split(" ")
    if len(parts) != 3:
        raise OrderError(f"cannot read '{line}'")
    try:
        qty = int(parts[2])
    except ValueError:
        raise OrderError(f"cannot read '{line}'")
    at = -1
    for i, d in enumerate(MENU):
        if d[0] == parts[1]:
            at = i
    if at < 0:
        raise OrderError(f"no dish named {parts[1]}")
    if qty < 1:
        raise OrderError(f"bad quantity {qty} for {parts[1]}")
    if MENU[at][3] < qty:
        raise OrderError(f"{parts[1]} sold out ({MENU[at][3]} left)")
    MENU[at][3] -= qty
    cost = MENU[at][1] * qty
    totals[parts[0]] = totals.get(parts[0], 0) + cost
    return cost


def money(c):
    return f"{c // 100}.{c % 100:0>2}"


lines = ["t1 steak 2", "t2 salad 1", "t1 cola 2", "t3 pizza 1", "t2 steak 2",
         "t3 cola 0", "t3 cola 2", "t2 cola 3", "oops", "t1 salad x"]
for line in lines:
    try:
        cost = order(line)
    except OrderError as e:
        print(f"rejected '{line}': {e}")
        continue
    print(f"ok {line} = {money(cost)}")
for d in MENU:
    print(f"{d[0]}: {d[3]} left, {prep(d[2], 2)}")
for t in sorted(totals):
    sub = totals[t]
    tax = sub * 8 // 100
    print(f"{t}: {money(sub)} + tax {money(tax)} = {money(sub + tax)}")
