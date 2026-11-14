/* Word frequencies: the same generator and vocabulary as wordfreq.ail, with a
   real hash map from word to count (open addressing, linear probing, FNV-1a,
   doubling at 3/4 load), like the Map[str, int] the AIL program uses. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *WORDS[] = {"the", "of", "and", "to", "in", "fox",
                              "jumps", "lazy", "dog", "river", "stone", "moss"};
#define NWORDS 12

typedef struct {
    const char *key;
    long long count;
} Slot;

typedef struct {
    Slot *slots;
    size_t cap;
    size_t len;
} Map;

static size_t hash_str(const char *s) {
    size_t h = 1469598103934665603ULL;
    for (; *s; s++) {
        h = (h ^ (unsigned char)*s) * 1099511628211ULL;
    }
    return h;
}

static Slot *find(Slot *slots, size_t cap, const char *key) {
    size_t i = hash_str(key) & (cap - 1);
    while (slots[i].key != NULL && strcmp(slots[i].key, key) != 0) {
        i = (i + 1) & (cap - 1);
    }
    return &slots[i];
}

static void grow(Map *m) {
    size_t ncap = m->cap * 2;
    Slot *ns = calloc(ncap, sizeof(Slot));
    for (size_t i = 0; i < m->cap; i++) {
        if (m->slots[i].key != NULL) {
            *find(ns, ncap, m->slots[i].key) = m->slots[i];
        }
    }
    free(m->slots);
    m->slots = ns;
    m->cap = ncap;
}

static long long map_get(Map *m, const char *key) {
    Slot *s = find(m->slots, m->cap, key);
    return s->key != NULL ? s->count : 0;
}

static void map_set(Map *m, const char *key, long long count) {
    Slot *s = find(m->slots, m->cap, key);
    if (s->key == NULL) {
        if ((m->len + 1) * 4 > m->cap * 3) {
            grow(m);
            s = find(m->slots, m->cap, key);
        }
        s->key = key;
        m->len += 1;
    }
    s->count = count;
}

int main(int argc, char **argv) {
    long long n = argc > 1 ? atoll(argv[1]) : 200000;
    Map counts = {calloc(8, sizeof(Slot)), 8, 0};
    long long seed = 7;
    for (long long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        const char *w = WORDS[seed % NWORDS];
        long long c = map_get(&counts, w);
        map_set(&counts, w, c + 1);
    }
    for (int k = 0; k < NWORDS; k++) {
        printf("%s %lld\n", WORDS[k], map_get(&counts, WORDS[k]));
    }
    printf("distinct %zu\n", counts.len);
    return 0;
}
