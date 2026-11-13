class CartError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def find_product(catalog, sku):
    for p in catalog:
        if p["sku"] == sku:
            return p
    return None


def cart_add(rows, catalog, sku, qty):
    p = find_product(catalog, sku)
    if p is None:
        raise CartError(f"unknown sku {sku}")
    have = 0
    for r in rows:
        if r["sku"] == sku:
            have += r["qty"]
    if have + qty > p["stock"]:
        raise CartError(f"only {p['stock']} of {sku} in stock, wanted {have + qty}")
    merged = False
    for r in rows:
        if r["sku"] == sku:
            r["qty"] += qty
            merged = True
    if not merged:
        rows.append({"sku": sku, "cat": p["cat"], "price": p["price"], "qty": qty})


def line_cost(item, rules):
    cost = item["price"] * item["qty"]
    for rule in rules:
        if rule[0] == "pct":
            if item["cat"] == rule[1]:
                cost -= cost * rule[2] // 100
        elif rule[0] == "bulk":
            if item["sku"] == rule[1] and item["qty"] >= rule[2]:
                cost = rule[3] * item["qty"]
    return cost


def cart_total(rows, rules):
    total = 0
    for item in rows:
        total += line_cost(item, rules)
    for rule in rules:
        if rule[0] == "flat":
            if total >= rule[1]:
                total -= rule[2]
    return total


def shipping(total):
    if total >= 5000:
        return 0
    return 499


def main():
    catalog = [
        {"sku": "tee", "cat": "apparel", "price": 1500, "stock": 10},
        {"sku": "cap", "cat": "apparel", "price": 1200, "stock": 3},
        {"sku": "mug", "cat": "kitchen", "price": 800, "stock": 20},
        {"sku": "pan", "cat": "kitchen", "price": 3500, "stock": 2},
    ]
    rules = [("pct", "apparel", 10), ("bulk", "mug", 4, 650), ("flat", 8000, 500)]
    orders = [("tee", 2), ("mug", 5), ("cap", 4), ("pan", 1), ("hat", 1), ("cap", 3), ("pan", 2)]
    rows = []
    for o in orders:
        try:
            cart_add(rows, catalog, o[0], o[1])
        except CartError as err:
            print(f"rejected {o[0]} x{o[1]}: {err}")
            continue
    for item in rows:
        print(f"{item['sku']} x{item['qty']} -> {money(line_cost(item, rules))}")
    total = cart_total(rows, rules)
    ship = shipping(total)
    print(f"total {money(total)} shipping {money(ship)} due {money(total + ship)}")


main()
