class Inventory:
    def __init__(self):
        self.items = []
        self.refused = 0

    def find(self, sku):
        for i, it in enumerate(self.items):
            if it["sku"] == sku:
                return i
        return None

    def receive(self, sku, n, unit):
        i = self.find(sku)
        if i is not None:
            self.items[i]["qty"] += n
        else:
            self.items.append({"sku": sku, "qty": n, "price": unit, "reorder": 5})

    def ship(self, sku, n):
        i = self.find(sku)
        if i is not None:
            if self.items[i]["qty"] >= n:
                self.items[i]["qty"] -= n
                return
        self.refused += 1
        print(f"refused: ship {n} of {sku}")

    def total(self):
        return sum(it["qty"] * it["price"] for it in self.items)


def money(cents):
    rest = cents % 100
    return f"{cents // 100}.{rest:0>2}"


def main():
    inv = Inventory()
    inv.receive("bolt", 200, 15)
    inv.receive("nut", 150, 10)
    inv.receive("gear", 12, 1250)
    inv.receive("belt", 8, 899)
    inv.ship("bolt", 120)
    inv.ship("gear", 9)
    inv.ship("belt", 9)
    inv.ship("cog", 1)
    inv.receive("belt", 4, 899)
    inv.ship("nut", 148)
    ranked = sorted(inv.items, key=lambda it: -(it["qty"] * it["price"]))
    print("sku      qty    value")
    for r in ranked:
        sku = r["sku"]
        qty = r["qty"]
        value = money(r["qty"] * r["price"])
        print(f"{sku:<6} {qty:>5} {value:>8}")
    print("reorder:")
    for low in inv.items:
        if low["qty"] <= low["reorder"]:
            print(f"  {low['sku']} ({low['qty']} left)")
    print(f"total value: {money(inv.total())}")
    print(f"refused orders: {inv.refused}")


main()
