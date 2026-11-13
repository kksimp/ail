def sieve(limit):
    flags = [i >= 2 for i in range(limit + 1)]
    p = 2
    while p * p <= limit:
        if flags[p]:
            j = p * p
            while j <= limit:
                flags[j] = False
                j += p
        p += 1
    return flags


def primes_upto(flags):
    return [i for i in range(len(flags)) if flags[i]]


def factorize(n, primes):
    left = n
    parts = []
    for p in primes:
        if p * p > left:
            break
        e = 0
        while left % p == 0:
            left //= p
            e += 1
        if e == 1:
            parts.append(f"{p}")
        elif e > 1:
            parts.append(f"{p}^{e}")
    if left > 1:
        parts.append(f"{left}")
    return " * ".join(parts)


def goldbach(n, flags):
    for a in range(2, n // 2 + 1):
        if flags[a] and flags[n - a]:
            return f"{n} = {a} + {n - a}"
    return f"{n} has no decomposition"


def main():
    flags = sieve(200)
    primes = primes_upto(flags)
    print(f"primes below 200: {len(primes)}")
    print(f"first twenty: {primes[:20]}")
    print(f"sum of primes below 200: {sum(primes)}")
    twins = []
    for i in range(1, len(primes)):
        if primes[i] - primes[i - 1] == 2:
            twins.append(f"({primes[i - 1]},{primes[i]})")
    print(f"twin pairs: {len(twins)}")
    start = len(twins) - 3
    tail = " ".join(twins[start:])
    print(f"last three: {tail}")
    for n in [84, 97, 120, 169, 198]:
        print(f"{n} = {factorize(n, primes)}")
    for m in [28, 100, 198]:
        print(goldbach(m, flags))


main()
