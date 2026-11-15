#include "cli.h"

#include <string.h>

const char *const USAGE = "usage: till orders|invoice ORDER|stock|low N|accounts|history ID|sales|margin|purchases|gifts|promos";

static const struct {
    const char *name;
    enum command cmd;
} SIMPLE[] = {
    {"orders", C_ORDERS}, {"stock", C_STOCK}, {"accounts", C_ACCOUNTS}, {"sales", C_SALES},
    {"margin", C_MARGIN}, {"purchases", C_PURCHASES}, {"gifts", C_GIFTS}, {"promos", C_PROMOS},
};

struct request cli_parse(int argc, char **argv) {
    struct request r;
    memset(&r, 0, sizeof r);
    r.cmd = C_USAGE;
    if (argc == 1) {
        for (size_t i = 0; i < sizeof SIMPLE / sizeof SIMPLE[0]; i++)
            if (strcmp(argv[0], SIMPLE[i].name) == 0) r.cmd = SIMPLE[i].cmd;
        return r;
    }
    if (argc != 2) return r;
    const char *arg = argv[1];
    if (strcmp(argv[0], "invoice") == 0 && is_order(arg)) {
        r.cmd = C_INVOICE;
        strcpy(r.arg, arg);
    } else if (strcmp(argv[0], "history") == 0 && is_id(arg)) {
        r.cmd = C_HISTORY;
        strcpy(r.arg, arg);
    } else if (strcmp(argv[0], "low") == 0 && digits(arg, 5, &r.n)) {
        r.cmd = C_LOW;
    }
    return r;
}
