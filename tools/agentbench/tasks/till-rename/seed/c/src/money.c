#include "money.h"

#include <stdio.h>

long long percent(long long a, long long p) {
    return (a * p + 50) / 100;
}

char *money_show(long long c, char buf[32]) {
    long long a = c < 0 ? -c : c;
    snprintf(buf, 32, "%s%lld.%02lld", c < 0 ? "-" : "", a / 100, a % 100);
    return buf;
}
