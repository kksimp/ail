class TradeError(Exception):
    pass


held = {}
realized = 0
fees = 0


def trade(line, flat):
    global realized, fees
    parts = line.split(" ")
    if len(parts) != 4:
        raise TradeError(f"cannot read '{line}'")
    try:
        qty = int(parts[2])
        price = int(parts[3])
    except ValueError:
        raise TradeError(f"cannot read '{line}'")
    if qty <= 0 or price <= 0:
        raise TradeError(f"non-positive amount in '{line}'")
    sym = parts[1]
    shares, basis = held.get(sym, (0, 0))
    fee = 499 if flat else 2 * qty
    if parts[0] == "buy":
        held[sym] = (shares + qty, basis + qty * price)
    elif parts[0] == "sell":
        if qty > shares:
            raise TradeError(f"{sym}: have {shares}, want to sell {qty}")
        removed = basis * qty // shares
        realized += qty * price - removed
        held[sym] = (shares - qty, basis - removed)
    else:
        raise TradeError(f"cannot read '{line}'")
    fees += fee
    return fee


def money(c):
    sign = "-" if c < 0 else ""
    a = abs(c)
    return f"{sign}{a // 100}.{a % 100:0>2}"


trades = ["buy ACME 100 1500", "buy ACME 50 1800", "buy BOLT 10 20000",
          "sell ACME 60 2000", "sell BOLT 11 21000", "buy ZED 0 100",
          "hold ACME 1 1", "sell BOLT 4 19000", "buy CORN 7"]
marks = {"ACME": 1900, "BOLT": 20500, "CORN": 100}
for i, t in enumerate(trades):
    try:
        paid = trade(t, i % 2 == 0)
    except TradeError as e:
        print(f"rejected: {e}")
        continue
    print(f"ok {t}, fee {money(paid)}")
unreal = 0
for s in sorted(held):
    sh, ba = held[s]
    mark = sh * marks[s]
    unreal += mark - ba
    print(f"{s}: {sh} shares, basis {money(ba)}, value {money(mark)}")
print(f"realized {money(realized)}")
print(f"unrealized {money(unreal)}")
print(f"fees {money(fees)}")
