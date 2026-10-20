# Python twin of sieve.ail (CPython 3, one thread).
import sys


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000000
    is_prime = [True] * n
    is_prime[0] = False
    is_prime[1] = False
    i = 2
    while i * i < n:
        if is_prime[i]:
            j = i * i
            while j < n:
                is_prime[j] = False
                j += i
        i += 1
    count = 0
    total = 0
    largest = 0
    for k in range(n):
        if is_prime[k]:
            count += 1
            total += k
            largest = k
    print(f"primes below {n}: {count}")
    print(f"sum: {total}")
    print(f"largest: {largest}")


main()
