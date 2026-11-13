def divisor_sums(limit):
    sums = [0 for _ in range(limit + 1)]
    for d in range(1, limit // 2 + 1):
        m = d * 2
        while m <= limit:
            sums[m] += d
            m += d
    return sums


def kind(n, total):
    if total == n:
        return "perfect"
    if total > n:
        return "abundant"
    return "deficient"


def perfect_list(sums):
    return [n for n in range(2, len(sums)) if sums[n] == n]


def amicable_pairs(sums):
    out = []
    limit = len(sums) - 1
    for a in range(2, limit + 1):
        b = sums[a]
        if b > a and b <= limit and sums[b] == a:
            out.append(f"{a}/{b}")
    return out


def main():
    limit = 10000
    sums = divisor_sums(limit)
    perfect = 0
    abundant = 0
    deficient = 0
    for n in range(1, limit + 1):
        k = kind(n, sums[n])
        if k == "perfect":
            perfect += 1
        elif k == "abundant":
            abundant += 1
        else:
            deficient += 1
    print(f"numbers 1..{limit}")
    print(f"  perfect {perfect}, abundant {abundant}, deficient {deficient}")
    print(f"perfect numbers: {' '.join(str(x) for x in perfect_list(sums))}")
    print(f"amicable pairs: {' '.join(amicable_pairs(sums))}")
    small = [n for n in range(1, 31) if kind(n, sums[n]) == "abundant"]
    print(f"abundant up to 30: {' '.join(str(x) for x in small)}")
    odd = [n for n in range(1, 1001) if n % 2 == 1 and kind(n, sums[n]) == "abundant"]
    print(f"first odd abundant: {odd[0]}")
    print(f"sigma(28) = {sums[28] + 28}")


main()
