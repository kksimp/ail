def gcd(a, b):
    p = abs(a)
    q = abs(b)
    while q != 0:
        p, q = q, p % q
    return p


def lcm(a, b):
    if a == 0 or b == 0:
        return 0
    return abs(a // gcd(a, b) * b)


def ext_gcd(a, b):
    if b == 0:
        return (a, 1, 0)
    g, x, y = ext_gcd(b, a % b)
    return (g, y, x - (a // b) * y)


def lcm_all(xs):
    acc = 1
    for v in xs:
        acc = lcm(acc, v)
    return acc


def print_table(size):
    head = "   |"
    for c in range(1, size + 1):
        head = f"{head}{c:>3}"
    print(head)
    print("---+------------------------")
    for r in range(1, size + 1):
        line = f"{r:>2} |"
        for k in range(1, size + 1):
            g = gcd(r, k)
            line = f"{line}{g:>3}"
        print(line)


def main():
    print_table(8)
    firsts = [4, 21, 8, 17, 100]
    seconds = [6, 6, 12, 5, 75]
    for a, b in zip(firsts, seconds):
        print(f"gcd({a}, {b}) = {gcd(a, b)}, lcm({a}, {b}) = {lcm(a, b)}")
    for pair in [[240, 46], [99, 78], [17, 5]]:
        g, x, y = ext_gcd(pair[0], pair[1])
        print(f"{pair[0]}*({x}) + {pair[1]}*({y}) = {g}")
    nums = [k for k in range(1, 21)]
    print(f"lcm of 1..20 = {lcm_all(nums)}")


main()
