"""Vending machine inventory manager."""


class VendError(Exception):
    pass


def money(cents):
    return "%d.%02d" % (cents // 100, cents % 100)


def change_for(amount):
    left = amount
    parts = []
    for coin in [100, 25, 10, 5, 1]:
        n = left // coin
        if n > 0:
            parts.append("%dx%d" % (n, coin))
            left -= n * coin
    if not parts:
        return "no change"
    return " ".join(parts)


def to_int(s):
    try:
        return int(s)
    except ValueError:
        return None


class Machine:
    def __init__(self, cap):
        self.slots = {}
        self.revenue = 0
        self.cap = cap

    def exec(self, line):
        p = line.split()
        bad = VendError("bad command '%s'" % line)
        if len(p) == 5 and p[0] == "stock":
            price = to_int(p[3])
            if price is None:
                raise bad
            count = to_int(p[4])
            if count is None:
                raise bad
            if count > self.cap:
                raise VendError("slot %s would overflow" % p[1])
            self.slots[p[1]] = {"name": p[2], "price": price, "qty": count, "sold": 0}
            return "stocked %s with %d %s" % (p[1], count, p[2])
        if len(p) != 3:
            raise bad
        if p[1] not in self.slots:
            raise VendError("no slot %s" % p[1])
        s = self.slots[p[1]]
        n = to_int(p[2])
        if n is None:
            raise bad
        if p[0] == "buy":
            if s["qty"] == 0:
                raise VendError("slot %s is sold out" % p[1])
            if n < s["price"]:
                raise VendError("need %d, got %d" % (s["price"], n))
            self.slots[p[1]] = dict(s, qty=s["qty"] - 1, sold=s["sold"] + 1)
            self.revenue += s["price"]
            return "sold %s, change: %s" % (s["name"], change_for(n - s["price"]))
        if p[0] == "restock":
            if s["qty"] + n > self.cap:
                raise VendError("slot %s would overflow" % p[1])
            self.slots[p[1]] = dict(s, qty=s["qty"] + n)
            return "%s now holds %d" % (p[1], s["qty"] + n)
        if p[0] == "price" and n > 0:
            self.slots[p[1]] = dict(s, price=n)
            return "%s now costs %s" % (s["name"], money(n))
        raise bad


def main():
    cmds = [
        "stock A1 cola 150 5", "stock A2 chips 100 3", "stock B1 water 125 2", "stock B2 gum 50 99",
        "buy A1 200", "buy A1 100", "buy A2 500", "buy B1 125", "buy B1 125", "buy B1 125",
        "buy C9 100", "restock B1 4", "restock A1 9", "price A2 110", "buy A2 100", "buy A2 110",
        "refund A2", "price A1 0", "stock A3 mints x 5",
    ]
    m = Machine(10)
    for c in cmds:
        try:
            print(m.exec(c))
        except VendError as err:
            print("! %s: %s" % (c, err))
    for code in sorted(m.slots):
        s = m.slots[code]
        flag = " LOW" if s["qty"] <= 1 else ""
        print("%s %-6s price %s qty %d sold %d%s" % (code, s["name"], money(s["price"]), s["qty"], s["sold"], flag))
    print("revenue %s" % money(m.revenue))


main()
