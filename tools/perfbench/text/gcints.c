#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(int argc, char **argv) {
    ll n = argc > 1 ? atoll(argv[1]) : 20000000, total = 0;
    ll *big = malloc(50000000 * sizeof *big);
    for (ll i = 0; i < 50000000; i++) big[i] = i;
    for (ll i = 0; i < n; i++) { char *s = malloc(24); total += sprintf(s, "%lld", i); free(s); }
    printf("chars %lld keep %lld\n", total, big[49999999]);
    return 0;
}
