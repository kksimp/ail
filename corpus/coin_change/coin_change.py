INF = 1000000000


def make_change(coins, amount):
    best = [0]
    pick = [0]
    for a in range(1, amount + 1):
        cheapest = INF
        chosen = 0
        for c in coins:
            if c <= a and best[a - c] + 1 < cheapest:
                cheapest = best[a - c] + 1
                chosen = c
        best.append(cheapest)
        pick.append(chosen)
    if best[amount] >= INF:
        return (-1, [])
    used = []
    left = amount
    while left > 0:
        used.append(pick[left])
        left -= pick[left]
    return (best[amount], sorted(used, reverse=True))


def greedy(coins, amount):
    left = amount
    count = 0
    for c in sorted(coins, reverse=True):
        while left >= c:
            left -= c
            count += 1
    return count if left == 0 else -1


def count_ways(coins, amount):
    ways = [1] + [0] * amount
    for c in coins:
        for t in range(c, amount + 1):
            ways[t] = ways[t] + ways[t - c]
    return ways[amount]


def text_of(n):
    return "none" if n < 0 else str(n)


def report(name, coins, amounts):
    print(f"{name}: coins " + " ".join(str(c) for c in coins))
    for amount in amounts:
        count, used = make_change(coins, amount)
        mix = "-" if count < 0 else "+".join(str(u) for u in used)
        print(f"  {amount:>3}: optimal {text_of(count)} [{mix}], greedy {text_of(greedy(coins, amount))}, ways {count_ways(coins, amount)}")


def main():
    report("us", [1, 5, 10, 25], [0, 30, 41, 99, 100])
    report("odd", [1, 3, 4], [6, 7, 11, 15])
    report("sparse", [5, 10], [7, 15, 35])
    report("euro", [1, 2, 5, 10, 20, 50], [48, 93])


main()
