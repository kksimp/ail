/* The shapes of journal and command-line fields (spec "Fields"). */
#ifndef FIELDS_H
#define FIELDS_H

/* Every field fits in this many bytes, with its terminating zero. */
#define FIELD 16

/* 1 to `width` ASCII digits (leading zeros allowed): stores the number in *out and returns 1; else 0. */
int digits(const char *s, int width, long long *out);

int is_sku(const char *s);   /* 1 to 12 of A-Z 0-9 -, starting with a letter */
int is_cat(const char *s);   /* 1 to 12 letters a-z */
int is_id(const char *s);    /* 1 to 12 of a-z 0-9, starting with a letter */
int is_code(const char *s);  /* 3 to 10 of A-Z 0-9, starting with a letter */
int is_order(const char *s); /* o and 1 to 6 digits */
int is_po(const char *s);    /* p and 1 to 6 digits */
int is_card(const char *s);  /* g and 1 to 6 digits */
int is_tier(const char *s);  /* basic, silver or gold */

#endif
