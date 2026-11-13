def to_roman(n):
    values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1]
    symbols = ["M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"]
    rest = n
    out = ""
    for i in range(len(values)):
        while rest >= values[i]:
            out += symbols[i]
            rest -= values[i]
    return out


def roman_value(c):
    return {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000}.get(c, 0)


def from_roman(s):
    total = 0
    for i in range(len(s)):
        v = roman_value(s[i])
        after = 0
        if i + 1 < len(s):
            after = roman_value(s[i + 1])
        if after > v:
            total -= v
        else:
            total += v
    return total


def main():
    for n in [1, 4, 9, 14, 40, 90, 400, 1994, 2024, 3888, 3999]:
        r = to_roman(n)
        print(f"{n:>4} {r:<14}{from_roman(r)}")
    ok = 0
    for n in range(1, 4000):
        if from_roman(to_roman(n)) == n:
            ok += 1
    print(f"round trips ok: {ok}")
    print(f"MCMXCIV = {from_roman('MCMXCIV')}")


main()
