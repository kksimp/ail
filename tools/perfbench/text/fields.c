#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(void) {
    ll total = 0, words = 0;
    char buf[4096];
    while (fgets(buf, sizeof buf, stdin)) {
        size_t k = strlen(buf); if (k && buf[k - 1] == 10) buf[k - 1] = 0;
        char *save, *parts[64]; int n = 0;
        for (char *t = strtok_r(buf, " ", &save); t && n < 64; t = strtok_r(NULL, " ", &save)) parts[n++] = t;
        total += atoll(parts[1]); words += n;
    }
    printf("words %lld total %lld\n", words, total);
    return 0;
}
