#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(int argc, char **argv) {
    ll n = argc > 1 ? atoll(argv[1]) : 3000000, total = 0;
    const char *line = "alpha beta gamma delta 12345";
    char buf[64];
    for (ll i = 0; i < n; i++) {
        strcpy(buf, line);
        char *save, *parts[8]; int k = 0;
        for (char *t = strtok_r(buf, " ", &save); t; t = strtok_r(NULL, " ", &save)) parts[k++] = t;
        total += k + atoll(parts[4]);
    }
    printf("total %lld\n", total);
    return 0;
}
