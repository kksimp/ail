def factorize(n):
    out = []
    rest = n
    p = 2
    while p * p <= rest:
        if rest % p == 0:
            times = 0
            while rest % p == 0:
                rest = rest // p
                times += 1
            out.append((p, times))
        p += 1 if p == 2 else 2
    if rest > 1:
        out.append((rest, 1))
    return out


def power_text(f):
    if f[1] == 1:
        return f"{f[0]}"
    return f"{f[0]}^{f[1]}"


def product_text(fs):
    if not fs:
        return "1"
    parts = [power_text(f) for f in fs]
    return " * ".join(parts)


def divisor_count(fs):
    total = 1
    for f in fs:
        total *= f[1] + 1
    return total


def divisor_sum(fs):
    total = 1
    for f in fs:
        term = 1
        power = 1
        for _ in range(f[1]):
            power *= f[0]
            term += power
        total *= term
    return total


def totient(fs):
    total = 1
    for f in fs:
        block = f[0] - 1
        for _ in range(1, f[1]):
            block *= f[0]
        total *= block
    return total


def is_prime(n):
    fs = factorize(n)
    return len(fs) == 1 and fs[0][1] == 1


def report(n):
    fs = factorize(n)
    print(f"{n} = {product_text(fs)}")
    print(f"  divisors {divisor_count(fs)}, sigma {divisor_sum(fs)}, phi {totient(fs)}")


def main():
    for n in [1, 2, 12, 97, 360, 1001, 4096, 65536, 999999, 600851475143]:
        report(n)
    line = []
    for m in range(2, 31):
        fs = factorize(m)
        line.append(f"{m}:{fs[0][0]}")
    print("smallest prime factor, 2..30")
    print(" ".join(line))
    primes = [q for q in range(2, 61) if is_prime(q)]
    print(f"primes up to 60: {' '.join(str(x) for x in primes)}")


main()
