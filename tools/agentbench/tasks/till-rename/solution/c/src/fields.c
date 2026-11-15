#include "fields.h"

#include <string.h>

static int upper(char c) { return c >= 'A' && c <= 'Z'; }
static int lower(char c) { return c >= 'a' && c <= 'z'; }
static int digit(char c) { return c >= '0' && c <= '9'; }
static int sku_char(char c) { return upper(c) || digit(c) || c == '-'; }
static int code_char(char c) { return upper(c) || digit(c); }
static int id_char(char c) { return lower(c) || digit(c); }

/* `lo` to `hi` characters, the first accepted by `first` and every one by `rest`. */
static int shaped(const char *s, size_t lo, size_t hi, int (*first)(char), int (*rest)(char)) {
    size_t n = strlen(s);
    if (n < lo || n > hi || !first(s[0])) return 0;
    for (size_t i = 0; i < n; i++)
        if (!rest(s[i])) return 0;
    return 1;
}

int digits(const char *s, int width, long long *out) {
    size_t n = strlen(s);
    if (n == 0 || n > (size_t)width) return 0;
    long long v = 0;
    for (size_t i = 0; i < n; i++) {
        if (!digit(s[i])) return 0;
        v = v * 10 + (s[i] - '0');
    }
    *out = v;
    return 1;
}

int is_sku(const char *s) { return shaped(s, 1, 12, upper, sku_char); }
int is_cat(const char *s) { return shaped(s, 1, 12, lower, lower); }
int is_id(const char *s) { return shaped(s, 1, 12, lower, id_char); }
int is_code(const char *s) { return shaped(s, 3, 10, upper, code_char); }

static int prefixed(const char *s, char p) {
    long long v;
    return s[0] == p && digits(s + 1, 6, &v);
}

int is_order(const char *s) { return prefixed(s, 'o'); }
int is_po(const char *s) { return prefixed(s, 'p'); }
int is_card(const char *s) { return prefixed(s, 'g'); }

int is_tier(const char *s) {
    return strcmp(s, "basic") == 0 || strcmp(s, "silver") == 0 || strcmp(s, "gold") == 0;
}
