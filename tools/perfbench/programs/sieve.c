/* Sieve of Eratosthenes: the same count, sum and largest as sieve.ail. */
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 1000000;
    char *is_prime = malloc(n);
    for (long k = 0; k < n; k++) {
        is_prime[k] = 1;
    }
    is_prime[0] = 0;
    is_prime[1] = 0;
    for (long i = 2; i * i < n; i++) {
        if (is_prime[i]) {
            for (long j = i * i; j < n; j += i) {
                is_prime[j] = 0;
            }
        }
    }
    long count = 0;
    long total = 0;
    long largest = 0;
    for (long k = 0; k < n; k++) {
        if (is_prime[k]) {
            count += 1;
            total += k;
            largest = k;
        }
    }
    printf("primes below %ld: %ld\n", n, count);
    printf("sum: %ld\n", total);
    printf("largest: %ld\n", largest);
    free(is_prime);
    return 0;
}
