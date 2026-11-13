DIGITS = "0123456789ABCDEF"


def to_base(n, base):
    if n == 0:
        return "0"
    out = []
    left = n
    while left > 0:
        out.append(DIGITS[left % base])
        left = left // base
    out.reverse()
    return "".join(out)


def from_base(text, base):
    acc = 0
    for ch in text:
        d = DIGITS.find(ch)
        if d < 0 or d >= base:
            return -1
        acc = acc * base + d
    return acc


def popcount(n):
    left = n
    count = 0
    while left > 0:
        bit = left & 1
        count += bit
        left = left >> 1
    return count


def gray(n):
    return n ^ (n >> 1)


def twos8(n):
    low = n & 255
    return to_base(low, 2).rjust(8, "0")


def main():
    print("   n | binary           | octal | hex  | ones")
    for n in [0, 5, 10, 255, 1000, 65535]:
        b = to_base(n, 2)
        o = to_base(n, 8)
        h = to_base(n, 16)
        print(f"{n:>4} | {b:<16} | {o:>5} | {h:>4} | {popcount(n)}")
    texts = ["101101", "777", "FF", "Z9", "12"]
    bases = [2, 8, 16, 16, 2]
    for text, base in zip(texts, bases):
        print(f"{text} in base {base} = {from_base(text, base)}")
    for v in [1, -1, -2, -128, 127, 100, -100]:
        print(f"{v:>4} -> {twos8(v)}")
    codes = []
    for g in range(8):
        codes.append(to_base(gray(g), 2).rjust(3, "0"))
    print("gray code: " + " ".join(codes))


main()
