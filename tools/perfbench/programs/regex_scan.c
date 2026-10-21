/* Regex scanning: the same generator, patterns and counts as regex_scan.ail,
   with POSIX extended regular expressions (the patterns have no alternation,
   so leftmost-longest and leftmost-first agree). */
#include <regex.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *WORDS[] = {"the", "of", "and", "to", "in", "fox",
                              "jumps", "lazy", "dog", "river", "stone", "moss"};
#define NWORDS 12

static long seed = 7;

static void line(char *out) {
    out[0] = 0;
    for (int k = 0; k < 6; k++) {
        seed = (seed * 3877 + 29573) % 139968;
        long t = seed / 16 % 8;
        char piece[64];
        if (t == 0) {
            snprintf(piece, sizeof piece, "user%ld@host%ld.com", seed % 97, seed % 13);
        } else if (t == 1) {
            snprintf(piece, sizeof piece, "id=%ld", seed % 1000);
        } else if (t == 2) {
            snprintf(piece, sizeof piece, "%ld", seed % 100000);
        } else if (t == 3) {
            snprintf(piece, sizeof piece, "val=%ld", seed % 50);
        } else {
            snprintf(piece, sizeof piece, "%s", WORDS[seed % NWORDS]);
        }
        if (k > 0) {
            strcat(out, " ");
        }
        strcat(out, piece);
    }
}

/* The matches of `re` in `text`: their count, and their total length. */
static long scan(const regex_t *re, const char *text, long *chars) {
    long count = 0;
    regmatch_t m;
    const char *p = text;
    while (*p && regexec(re, p, 1, &m, 0) == 0) {
        count += 1;
        *chars += m.rm_eo - m.rm_so;
        p += m.rm_eo > m.rm_so ? m.rm_eo : m.rm_so + 1;
    }
    return count;
}

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 20000;
    regex_t digits, mail, pair;
    regcomp(&digits, "[0-9]+", REG_EXTENDED);
    regcomp(&mail, "[a-z0-9]+@[a-z0-9]+\\.com", REG_EXTENDED);
    regcomp(&pair, "[a-z]+=[0-9]+", REG_EXTENDED);
    long runs = 0, digit_chars = 0, mails = 0, pairs = 0, replaced = 0, ignored = 0;
    char text[512];
    for (long i = 0; i < n; i++) {
        line(text);
        long chars = 0;
        runs += scan(&digits, text, &chars);
        digit_chars += chars;
        mails += scan(&mail, text, &ignored);
        pairs += scan(&pair, text, &ignored);
        /* "#" replaces each digit run: the text, less the runs, plus one per run. */
        replaced += (long)strlen(text) - chars + scan(&digits, text, &ignored);
    }
    printf("digit runs %ld\n", runs);
    printf("digit chars %ld\n", digit_chars);
    printf("addresses %ld\n", mails);
    printf("pairs %ld\n", pairs);
    printf("replaced length %ld\n", replaced);
    return 0;
}
