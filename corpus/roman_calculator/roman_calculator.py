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
        if v == 0:
            return None
        after = 0
        if i + 1 < len(s):
            after = roman_value(s[i + 1])
        if after > v:
            total -= v
        else:
            total += v
    if 1 <= total <= 3999 and to_roman(total) == s:
        return total
    return None


class CalcError(Exception):
    pass


def calc(src):
    toks = src.split()
    if len(toks) % 2 == 0:
        raise CalcError("bad syntax")
    nums = []
    ops = []
    for i in range(len(toks)):
        if i % 2 == 0:
            n = from_roman(toks[i])
            if n is None:
                raise CalcError(f"bad numeral {toks[i]}")
            nums.append(n)
        elif toks[i] in ["+", "-", "*", "/"]:
            ops.append(toks[i])
        else:
            raise CalcError("bad syntax")
    vals = [nums[0]]
    adds = []
    for i in range(len(ops)):
        rhs = nums[i + 1]
        k = len(vals) - 1
        if ops[i] == "*":
            vals[k] = vals[k] * rhs
        elif ops[i] == "/":
            vals[k] = vals[k] // rhs
        else:
            adds.append(ops[i])
            vals.append(rhs)
    total = vals[0]
    for i in range(len(adds)):
        if adds[i] == "+":
            total += vals[i + 1]
        else:
            total -= vals[i + 1]
    if total < 1 or total > 3999:
        raise CalcError(f"result {total} is outside 1..3999")
    return to_roman(total)


def main():
    for src in [
        "XIV + II * III",
        "MM / X - XL",
        "C - L - X",
        "XII * XII",
        "IIII + I",
        "X +",
        "V - X",
        "M * IV",
        "XL / III + VII",
        "X % V",
    ]:
        try:
            r = calc(src)
        except CalcError as err:
            print(f"{src} => error: {err}")
            continue
        print(f"{src} => {r}")


main()
