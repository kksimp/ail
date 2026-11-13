def nim_sum(piles):
    x = 0
    for p in piles:
        x = x ^ p
    return x


def optimal(piles):
    x = nim_sum(piles)
    if x != 0:
        for i in range(len(piles)):
            if (piles[i] ^ x) < piles[i]:
                return (i, piles[i] - (piles[i] ^ x))
    for i in range(len(piles)):
        if piles[i] > 0:
            return (i, 1)
    return (0, 0)


def naive(piles):
    best = 0
    for i in range(len(piles)):
        if piles[i] > piles[best]:
            best = i
    return (best, 1)


def wins(a, b, c, memo):
    if a + b + c == 0:
        return False
    key = f"{a},{b},{c}"
    if key in memo:
        return memo[key]
    result = False
    for t in range(1, a + 1):
        if not wins(a - t, b, c, memo):
            result = True
    for t in range(1, b + 1):
        if not wins(a, b - t, c, memo):
            result = True
    for t in range(1, c + 1):
        if not wins(a, b, c - t, memo):
            result = True
    memo[key] = result
    return result


def game(start, first_optimal):
    piles = list(start)
    turn = 0
    print(f"start {piles}, nim-sum {nim_sum(piles)}")
    while sum(piles) > 0:
        use_optimal = (turn % 2 == 0) == first_optimal
        pile, take = optimal(piles) if use_optimal else naive(piles)
        piles[pile] -= take
        who = "optimal" if use_optimal else "naive  "
        print(f"  {who} takes {take} from pile {pile} -> {piles}")
        turn += 1
    winner = "optimal" if (turn % 2 == 1) == first_optimal else "naive"
    print(f"  last object taken by the {winner} player")


def main():
    game([3, 4, 5], True)
    game([3, 4, 5], False)
    game([1, 2, 3], True)
    memo = {}
    agree = 0
    losing = 0
    for a in range(5):
        for b in range(5):
            for c in range(5):
                brute = wins(a, b, c, memo)
                rule = nim_sum([a, b, c]) != 0
                if brute == rule:
                    agree += 1
                if not brute:
                    losing += 1
    print(f"positions checked: 125, rule agrees on {agree}, lost for the mover: {losing}")


def tests():
    assert nim_sum([3, 4, 5]) == 2 and nim_sum([1, 2, 3]) == 0
    i, t = optimal([3, 4, 5])
    after = [3, 4, 5]
    after[i] -= t
    assert nim_sum(after) == 0
    memo = {}
    assert wins(1, 2, 4, memo) and not wins(1, 2, 3, memo) and not wins(0, 0, 0, memo)


tests()
main()
