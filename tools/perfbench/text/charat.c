#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(void) {
    ll n = 20000000, total = 0;
    char *s = malloc(n + 1);
    for (ll i = 0; i < n; i++) s[i] = "abcdefghij"[i % 10];
    s[n] = 0;
    for (int r = 0; r < 25; r++)
        for (ll i = 0; i < n; i++) if (s[i] == 'e') total++;
    printf("e %lld\n", total);
    return 0;
}
