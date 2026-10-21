READ = 1 << 2
WRITE = 1 << 1
EXEC = 1 << 0


def mode_text(mode):
    r = "r" if mode & READ else "-"
    w = "w" if mode & WRITE else "-"
    x = "x" if mode & EXEC else "-"
    return f"{r}{w}{x}"


def count_bits(n):
    left = n
    count = 0
    while left != 0:
        count = count + (left & 1)
        left = left >> 1
    return count


def mix(h):
    x = h
    x = x ^ (x >> 7)
    x = x ^ (x << 3)
    return x & 0xFFFF


def main():
    print(f"mode 6 is {mode_text(6)}, mode 5 is {mode_text(5)}")
    print(f"set bits in 0b1011: {count_bits(11)}")
    print(f"not 0 is {~0}, shr of -8 by 1 is {(-8) >> 1}")
    print(f"hash of 1234: {mix(1234)}")
    s = 0
    for i in [1, 4, 9, 40]:
        s = s | (1 << i)
    print(f"bitset has 4: {(s & (1 << 4)) != 0}, has 5: {(s & (1 << 5)) != 0}".lower())
    print(f"bitset count: {count_bits(s)}")
    print(f"bitset as int: {s}")


if __name__ == "__main__":
    main()
