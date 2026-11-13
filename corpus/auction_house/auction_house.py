class AuctionError(Exception):
    pass


lots = [
    {"name": "vase", "start": 100, "reserve": 300, "high": 0, "leader": "", "bids": 0, "closed": False},
    {"name": "clock", "start": 50, "reserve": 500, "high": 0, "leader": "", "bids": 0, "closed": False},
]


def next_min(i, high):
    if i == 0:
        return high + 25
    return high + max(high * 10 // 100, 1)


def find(name):
    for i, lot in enumerate(lots):
        if lot["name"] == name:
            return i
    raise AuctionError(f"no lot {name}")


def run(line):
    parts = line.split(" ")
    if parts[0] == "close" and len(parts) == 2:
        at = find(parts[1])
        lot = lots[at]
        if lot["closed"]:
            raise AuctionError(f"lot {parts[1]} is closed")
        lot["closed"] = True
        if lot["high"] >= lot["reserve"] and lot["bids"] > 0:
            return f"{parts[1]} sold to {lot['leader']}"
        return f"{parts[1]} passed"
    if parts[0] != "bid" or len(parts) != 4:
        raise AuctionError(f"cannot read '{line}'")
    try:
        amt = int(parts[3])
    except ValueError:
        raise AuctionError(f"cannot read '{line}'")
    at = find(parts[1])
    lot = lots[at]
    if lot["closed"]:
        raise AuctionError(f"lot {parts[1]} is closed")
    if lot["leader"] == parts[2]:
        raise AuctionError(f"{parts[2]} already leads")
    need = lot["start"] if lot["bids"] == 0 else next_min(at, lot["high"])
    if amt < need:
        raise AuctionError(f"{parts[1]}: need at least {need}, got {amt}")
    lot["high"] = amt
    lot["leader"] = parts[2]
    lot["bids"] += 1
    return f"{parts[2]} leads {parts[1]} at {amt}"


commands = [
    "bid vase ann 100", "bid vase bob 110", "bid vase bob 125", "bid vase bob 200",
    "bid vase ann 200", "bid vase ann 350", "bid clock cy 50", "bid clock dee 54",
    "bid clock dee 55", "bid rug ann 10", "bid vase ann", "close vase",
    "bid vase eve 999", "close clock", "close rug", "sell vase",
]
for c in commands:
    try:
        msg = run(c)
    except AuctionError as e:
        print(f"rejected '{c}': {e}")
        continue
    print(msg)
for lot in lots:
    premium = lot["high"] * 10 // 100
    print(f"{lot['name']}: {lot['bids']} bids, high {lot['high']} by {lot['leader']}, premium {premium}")
