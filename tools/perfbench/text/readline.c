#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(void) {
    ll total = 0, count = 0;
    char buf[4096];
    while (fgets(buf, sizeof buf, stdin)) { size_t k = strlen(buf); if (k && buf[k - 1] == 10) k--; total += k; count++; }
    printf("lines %lld chars %lld\n", count, total);
    return 0;
}
