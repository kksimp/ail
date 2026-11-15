/* One line of the stock log. */
#ifndef EVENT_H
#define EVENT_H

#include <stddef.h>

enum kind { RECEIVE, SHIP, COUNT };

struct event {
    enum kind kind;
    char sku[64];
    long long qty;
    long long price;
};

/* 1 to 5 ASCII digits of s[0..n), as a number in *out. Returns 1 on success. */
int event_number(const char *s, size_t n, long long *out);

/* Parses one line (no newline). Returns 1 and fills *e for a valid event, 0 otherwise. */
int event_parse(const char *line, struct event *e);

#endif
