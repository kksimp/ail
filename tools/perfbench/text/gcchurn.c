#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(int argc, char **argv) {
    ll n = argc > 1 ? atoll(argv[1]) : 30000000, total = 0;
    char **keep = malloc(500000 * sizeof *keep);
    for (ll i = 0; i < 500000; i++) { keep[i] = malloc(24); sprintf(keep[i], "row %lld", i); }
    for (ll i = 0; i < n; i++) { char *s = malloc(24); total += sprintf(s, "%lld", i); free(s); }
    printf("chars %lld keep %s\n", total, keep[499999]);
    return 0;
}
