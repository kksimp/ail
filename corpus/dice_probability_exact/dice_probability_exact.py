def gcd(a, b):
    x, y = a, b
    while y != 0:
        x, y = y, x % y
    return x


def frac(num, den):
    if num == 0:
        return "0"
    g = gcd(num, den)
    if den // g == 1:
        return f"{num // g}"
    return f"{num // g}/{den // g}"


def ways(n, sides):
    counts = [1]
    for d in range(n):
        nxt = [0] * (len(counts) + sides)
        for total in range(len(counts)):
            for face in range(1, sides + 1):
                nxt[total + face] += counts[total]
        counts = nxt
    return counts


def at_least(counts, target):
    return sum(counts[t] for t in range(len(counts)) if t >= target)


def expected(counts):
    weighted = sum(t * counts[t] for t in range(len(counts)))
    return frac(weighted, sum(counts))


def advantage(sides, target):
    n = 0
    for a in range(1, sides + 1):
        for b in range(1, sides + 1):
            if max(a, b) >= target:
                n += 1
    return n


def main():
    two = ways(2, 6)
    total_all = sum(two)
    print(f"2d6: {total_all} outcomes")
    for total in range(2, 13):
        bar = "#" * two[total]
        print(f"{total:>3} {two[total]:>2}/{total_all} {bar}")
    print(f"P(sum >= 10) = {frac(at_least(two, 10), total_all)}")
    print(f"P(sum is 7) = {frac(two[7], total_all)}")
    print(f"E[2d6] = {expected(two)}")
    three = ways(3, 6)
    print(f"3d6: {sum(three)} outcomes, E = {expected(three)}")
    print(f"P(3d6 >= 15) = {frac(at_least(three, 15), sum(three))}")
    print(f"P(3d6 >= 11) = {frac(at_least(three, 11), sum(three))}")
    d20 = ways(1, 20)
    print(f"hit DC 15 on a d20: {frac(at_least(d20, 15), 20)}")
    print(f"hit DC 15 with advantage: {frac(advantage(20, 15), 400)}")
    four = ways(4, 6)
    print(f"4d6: {sum(four)} outcomes, most common sum {four.index(max(four))}")


def tests():
    c = ways(2, 6)
    assert c[7] == 6 and c[2] == 1 and sum(c) == 36
    assert frac(6, 36) == "1/6" and frac(36, 36) == "1" and frac(0, 36) == "0"
    assert advantage(20, 15) == 400 - 14 * 14
    assert expected(ways(2, 6)) == "7"


tests()
main()
