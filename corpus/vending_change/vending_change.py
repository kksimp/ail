class ChangeError(Exception):
    pass


def solve(denoms, avail, idx, left):
    if idx == len(denoms):
        if left == 0:
            return []
        return None
    top = left // denoms[idx]
    if avail[idx] < top:
        top = avail[idx]
    found = False
    best = []
    best_n = 0
    for j in range(top + 1):
        k = top - j
        rest = solve(denoms, avail, idx + 1, left - k * denoms[idx])
        if rest is None:
            continue
        total = k + sum(rest)
        if not found or total < best_n:
            best = [k] + rest
            best_n = total
            found = True
    if found:
        return best
    return None


class Till:
    def __init__(self, denoms, counts):
        self.denoms = denoms
        self.counts = counts

    def change(self, price, paid):
        if paid < price:
            raise ChangeError(f"paid {paid} for a price of {price}")
        due = paid - price
        plan = solve(self.denoms, self.counts, 0, due)
        if plan is None:
            raise ChangeError(f"cannot make exact change for {due}")
        for i in range(len(plan)):
            self.counts[i] = self.counts[i] - plan[i]
        return plan


def describe(denoms, plan):
    parts = []
    for i in range(len(plan)):
        if plan[i] > 0:
            parts.append(f"{plan[i]}x{denoms[i]}")
    if not parts:
        return "no coins"
    return " ".join(parts)


def main():
    till = Till([100, 25, 20, 5], [2, 3, 3, 8])
    sales = [(60, 100), (75, 100), (35, 200), (100, 100), (120, 100), (45, 100), (45, 100), (10, 100), (85, 100)]
    for sale in sales:
        try:
            plan = till.change(sale[0], sale[1])
        except ChangeError as err:
            print(f"sale {sale[0]} paid {sale[1]}: {err}")
            continue
        print(f"sale {sale[0]} paid {sale[1]}: {describe(till.denoms, plan)}")
    print("box left: " + " ".join(str(x) for x in till.counts))


main()
