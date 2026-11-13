def gcd(a, b):
    p, q = a, b
    while q != 0:
        p, q = q, p % q
    return p


def ext_gcd(a, b):
    old_r, cur_r = a, b
    old_s, cur_s = 1, 0
    while cur_r != 0:
        q = old_r // cur_r
        old_r, cur_r = cur_r, old_r - q * cur_r
        old_s, cur_s = cur_s, old_s - q * cur_s
    return old_r, old_s, (old_r - a * old_s) // b


def norm(x, m):
    return ((x % m) + m) % m


def mod_inv(a, m):
    g, x, _ = ext_gcd(norm(a, m), m)
    if g != 1:
        return 0
    return norm(x, m)


def mod_pow(base, exp, m):
    result = 1
    b = base % m
    e = exp
    while e > 0:
        if e % 2 == 1:
            result = result * b % m
        b = b * b % m
        e //= 2
    return result


def factors(n):
    out = []
    x = n
    p = 2
    while p * p <= x:
        while x % p == 0:
            out.append(p)
            x //= p
        p += 1
    if x > 1:
        out.append(x)
    return out


def factor_text(n):
    fs = factors(n)
    parts = []
    i = 0
    while i < len(fs):
        j = i
        while j < len(fs) and fs[j] == fs[i]:
            j += 1
        if j - i == 1:
            parts.append(f"{fs[i]}")
        else:
            parts.append(f"{fs[i]}^{j - i}")
        i = j
    return " * ".join(parts)


def totient(n):
    res = n
    last = 0
    for p in factors(n):
        if p != last:
            res = res // p * (p - 1)
            last = p
    return res


def is_prime_slow(n):
    return n >= 2 and len(factors(n)) == 1


def witness(n, a):
    d = n - 1
    s = 0
    while d % 2 == 0:
        d //= 2
        s += 1
    x = mod_pow(a, d, n)
    if x == 1 or x == n - 1:
        return False
    for _ in range(1, s):
        x = x * x % n
        if x == n - 1:
            return False
    return True


def is_prime_mr(n):
    if n < 2:
        return False
    for p in [2, 3, 5, 7]:
        if n == p:
            return True
        if n % p == 0:
            return False
    for a in [2, 3, 5, 7]:
        if witness(n, a):
            return False
    return True


def b(v):
    return "true" if v else "false"


print(f"gcd(84, 36) = {gcd(84, 36)}, gcd(17, 5) = {gcd(17, 5)}")
g, ex, ey = ext_gcd(240, 46)
print(f"ext_gcd(240, 46): g={g} x={ex} y={ey}, check {240 * ex + 46 * ey}")
print(f"17^-1 mod 3120 = {mod_inv(17, 3120)}, 6^-1 mod 9 = {mod_inv(6, 9)}")
for n in [360, 97, 1001, 65536, 999999, 123456789]:
    print(f"{n} = {factor_text(n)}, phi = {totient(n)}")
agree = 0
primes = 0
for n in range(2000):
    if is_prime_mr(n) == is_prime_slow(n):
        agree += 1
    if is_prime_mr(n):
        primes += 1
print(f"miller-rabin agrees with trial division on {agree} of 2000, primes found: {primes}")
print(f"561 prime: {b(is_prime_mr(561))}, 1000000007 prime: {b(is_prime_mr(1000000007))}, 2147483647 prime: {b(is_prime_mr(2147483647))}")
