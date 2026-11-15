#include "gifts.h"

#include <stdlib.h>
#include <string.h>

#include "grow.h"

void cards_init(struct cards *cs) {
    memset(cs, 0, sizeof *cs);
}

void cards_free(struct cards *cs) {
    free(cs->all);
    cards_init(cs);
}

struct card *cards_find(const struct cards *cs, const char *card) {
    for (size_t i = 0; i < cs->len; i++)
        if (strcmp(cs->all[i].card, card) == 0) return &cs->all[i];
    return NULL;
}

void cards_sell(struct cards *cs, const char *card, long long cents) {
    size_t at = 0;
    while (at < cs->len && strcmp(cs->all[at].card, card) < 0) at++;
    GROW(cs->all, cs->len, cs->cap);
    memmove(cs->all + at + 1, cs->all + at, (cs->len - at) * sizeof *cs->all);
    cs->len++;
    struct card *c = &cs->all[at];
    strcpy(c->card, card);
    c->sold = c->balance = cents;
}
