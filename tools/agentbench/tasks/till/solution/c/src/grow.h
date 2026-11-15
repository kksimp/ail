/* Growing arrays: every collection in the program is a pointer, a length and a capacity. */
#ifndef GROW_H
#define GROW_H

#include <stdlib.h>

/* Makes room for at least `need` elements of `size` bytes in *p, whose capacity is *cap. */
static inline void grow(void **p, size_t *cap, size_t need, size_t size) {
    if (need <= *cap) return;
    size_t n = *cap ? *cap * 2 : 8;
    while (n < need) n *= 2;
    void *t = realloc(*p, n * size);
    if (!t) abort();
    *p = t;
    *cap = n;
}

#define GROW(arr, len, cap) grow((void **)&(arr), &(cap), (len) + 1, sizeof *(arr))

#endif
