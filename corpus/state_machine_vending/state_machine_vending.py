def change_text(amount):
    left = amount
    parts = []
    for unit in [100, 25, 10, 5]:
        k = left // unit
        if k > 0:
            parts.append(f"{k}x{unit}")
            left -= k * unit
    if not parts:
        return "none"
    return " ".join(parts)


class Machine:
    def __init__(self):
        self.mode = "idle"
        self.credit = 0
        self.strikes = 0
        self.stock = [2, 1, 0, 3]
        self.prices = [75, 100, 125, 150]
        self.sales = 0
        self.takings = 0

    def coin(self, cents):
        if self.mode == "locked":
            return "ignored, machine locked"
        if cents not in (5, 10, 25, 100):
            self.strikes += 1
            if self.strikes >= 3:
                self.mode = "locked"
                return f"coin {cents} rejected, machine locked"
            return f"coin {cents} rejected (strike {self.strikes})"
        self.credit += cents
        self.mode = "credit"
        return f"credit {self.credit}"

    def choose(self, slot):
        if self.mode == "locked":
            return "ignored, machine locked"
        if slot < 0 or slot >= len(self.stock):
            return f"no such slot {slot}"
        if self.stock[slot] == 0:
            return f"slot {slot} sold out"
        price = self.prices[slot]
        if self.credit < price:
            return f"slot {slot} costs {price}, need {price - self.credit} more"
        due = self.credit - price
        self.stock[slot] -= 1
        self.sales += 1
        self.takings += price
        self.credit = 0
        self.mode = "idle"
        return f"vend slot {slot}, change {change_text(due)}"

    def refund(self):
        if self.mode == "locked":
            return "ignored, machine locked"
        if self.credit == 0:
            return "nothing to refund"
        back = self.credit
        self.credit = 0
        self.mode = "idle"
        return f"refund {change_text(back)}"

    def service(self, units):
        for i in range(len(self.stock)):
            self.stock[i] += units
        self.strikes = 0
        self.mode = "credit" if self.credit > 0 else "idle"
        return f"serviced, +{units} per slot"

    def handle(self, ev):
        kind, arg = ev
        if kind == "coin":
            return self.coin(arg)
        if kind == "select":
            return self.choose(arg)
        if kind == "refund":
            return self.refund()
        return self.service(arg)


def describe(ev):
    kind, arg = ev
    if kind == "refund":
        return "refund"
    return f"{kind} {arg}"


events = [
    ("select", 0),
    ("coin", 25),
    ("coin", 25),
    ("coin", 25),
    ("select", 0),
    ("coin", 100),
    ("select", 1),
    ("coin", 100),
    ("select", 1),
    ("select", 2),
    ("select", 7),
    ("refund", 0),
    ("coin", 3),
    ("coin", 7),
    ("coin", 50),
    ("coin", 25),
    ("service", 2),
    ("coin", 100),
    ("coin", 25),
    ("select", 2),
    ("coin", 100),
    ("coin", 100),
    ("select", 3),
    ("refund", 0),
]
m = Machine()
for ev in events:
    msg = m.handle(ev)
    print(f"{describe(ev):<10} -> {msg} [{m.mode}]")
print(f"sales: {m.sales}, takings: {m.takings}, credit: {m.credit}")
print("stock: " + " ".join(str(x) for x in m.stock))
