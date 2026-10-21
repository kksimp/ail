/* Output: 300,000 formatted lines to stdout. */
#include <stdio.h>

int main(void) {
    long sum = 0;
    for (long i = 0; i < 300000; i++) {
        sum += i % 13;
        printf("row %ld: value %ld sum %ld\n", i, i * 7 % 1000, sum);
    }
    return 0;
}
