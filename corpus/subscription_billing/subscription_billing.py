class BillError(Exception):
    pass


PRICES = {"basic": 900, "pro": 2900, "team": 9900}
OFF = [lambda m: 0, lambda m: 10 if m >= 2 else 0, lambda m: 25]
accounts = {}


def price(plan):
    if plan not in PRICES:
        raise BillError(f"no plan {plan}")
    return PRICES[plan]


def run(line):
    parts = line.split(" ")
    if len(parts) < 4:
        raise BillError(f"cannot read '{line}'")
    try:
        day = int(parts[1])
    except ValueError:
        raise BillError(f"cannot read '{line}'")
    act = parts[2]
    who = parts[3]
    if act == "subscribe" and len(parts) == 6:
        cost = price(parts[4])
        cur = accounts.get(who)
        if cur and cur["active"]:
            raise BillError(f"{who} is already subscribed")
        kinds = ["plain", "loyal", "student"]
        if parts[5] not in kinds:
            raise BillError(f"cannot read '{line}'")
        accounts[who] = {"plan": parts[4], "months": 0, "active": True, "disc": kinds.index(parts[5])}
        return cost * (30 - day) // 30
    acct = accounts.get(who)
    if acct is None or not acct["active"]:
        raise BillError(f"no customer {who}")
    if act == "cancel" and len(parts) == 4:
        accounts[who] = dict(acct, active=False)
        return 0
    if act == "upgrade" and len(parts) == 5:
        old = price(acct["plan"])
        fresh = price(parts[4])
        if fresh <= old:
            raise BillError(f"{parts[4]} is not an upgrade for {who}")
        accounts[who] = dict(acct, plan=parts[4])
        return (fresh - old) * (30 - day) // 30
    raise BillError(f"cannot read '{line}'")


def invoice(who):
    acct = accounts[who]
    base = price(acct["plan"])
    off = OFF[acct["disc"]](acct["months"])
    accounts[who] = dict(acct, months=acct["months"] + 1)
    return base - base * off // 100


events = [
    "1 1 subscribe ann basic plain", "1 10 subscribe bob pro student",
    "1 12 subscribe cy gold plain", "1 15 subscribe ann pro plain",
    "2 3 upgrade ann pro", "2 5 upgrade bob basic", "2 9 upgrade dan team",
    "2 20 subscribe dee team loyal", "3 7 cancel bob", "3 8 cancel bob",
    "3 9 upgrade dee basic", "4 2 subscribe bob basic plain", "4 4 bogus",
    "4 5 subscribe cy basic plain",
]
revenue = 0
for month in range(1, 5):
    print(f"month {month}")
    for line in events:
        if line.split(" ")[0] != str(month):
            continue
        try:
            extra = run(line)
        except BillError as e:
            print(f"  rejected '{line}': {e}")
            continue
        revenue += extra
        print(f"  ok {line} (prorated {extra})")
    for who in sorted(accounts):
        if accounts[who]["active"]:
            amt = invoice(who)
            revenue += amt
            print(f"  invoice {who}: {amt}")
print(f"revenue {revenue}")
