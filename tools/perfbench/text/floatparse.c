#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef long long ll;
int main(int argc, char **argv) {
    ll n = argc > 1 ? atoll(argv[1]) : 50000000;
    const char *texts[8] = {"3.14159", "0.001", "12345.678", "1e10", "2.718281828459045", "-7.5", "100", "6.02214076e23"};
    double total = 0.0;
    for (ll i = 0; i < n; i++) total += strtod(texts[i % 8], NULL);
    printf("total %.17g\n", total);
    return 0;
}
