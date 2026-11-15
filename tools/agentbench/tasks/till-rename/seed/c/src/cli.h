/* The command line: which report to print (spec "Reports", "Exit codes and stderr"). */
#ifndef CLI_H
#define CLI_H

#include "fields.h"

enum command {
    C_ORDERS, C_INVOICE, C_STOCK, C_LOW, C_CUSTOMERS, C_HISTORY, C_SALES, C_MARGIN, C_PURCHASES, C_GIFTS, C_PROMOS,
    C_USAGE /* missing, unknown or malformed: print the usage line and exit 1 */
};

struct request {
    enum command cmd;
    char arg[FIELD]; /* ORDER for invoice, ID for history */
    long long n;     /* N for low */
};

extern const char *const USAGE;

/* The report the arguments (without the program name) ask for. */
struct request cli_parse(int argc, char **argv);

#endif
