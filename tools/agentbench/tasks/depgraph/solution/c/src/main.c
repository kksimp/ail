#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int name_char(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' || c == '.' || c == '-';
}

static int valid_name(const char *s, size_t n) {
    if (n == 0) return 0;
    for (size_t i = 0; i < n; i++)
        if (!name_char(s[i])) return 0;
    return 1;
}

static int cmp_str(const void *a, const void *b) { return strcmp(*(char *const *)a, *(char *const *)b); }

static char **names;
static size_t nn;

static int find(const char *s) {
    size_t lo = 0, hi = nn;
    while (lo < hi) {
        size_t mid = (lo + hi) / 2;
        int c = strcmp(names[mid], s);
        if (c == 0) return (int)mid;
        if (c < 0) lo = mid + 1; else hi = mid;
    }
    return -1;
}

/* Marks everything reachable from start through one or more edges of adj (adj[i*n+j]: i -> j). */
static void reach(const unsigned char *adj, int start, unsigned char *seen) {
    int *stack = malloc(sizeof(int) * (nn * nn + 1));
    size_t sp = 0;
    memset(seen, 0, nn);
    for (size_t j = 0; j < nn; j++)
        if (adj[(size_t)start * nn + j]) stack[sp++] = (int)j;
    while (sp) {
        int x = stack[--sp];
        if (seen[x]) continue;
        seen[x] = 1;
        for (size_t j = 0; j < nn; j++)
            if (adj[(size_t)x * nn + j] && !seen[j]) stack[sp++] = (int)j;
    }
    free(stack);
}

int main(int argc, char **argv) {
    int cmd;
    if (argc == 2 && !strcmp(argv[1], "order")) cmd = 0;
    else if (argc == 2 && !strcmp(argv[1], "cycles")) cmd = 1;
    else if (argc == 3 && !strcmp(argv[1], "deps") && valid_name(argv[2], strlen(argv[2]))) cmd = 2;
    else if (argc == 3 && !strcmp(argv[1], "rdeps") && valid_name(argv[2], strlen(argv[2]))) cmd = 3;
    else {
        fputs("usage: depgraph order|cycles|deps NAME|rdeps NAME\n", stderr);
        return 1;
    }
    size_t cap = 4096, len = 0, got;
    char *buf = malloc(cap);
    while ((got = fread(buf + len, 1, cap - len - 1, stdin)) > 0) {
        len += got;
        if (len + 1 >= cap) buf = realloc(buf, cap *= 2);
    }
    buf[len] = 0;
    size_t maxn = len + 2;
    char **all = malloc(sizeof(char *) * 2 * maxn), **ea = malloc(sizeof(char *) * maxn), **eb = malloc(sizeof(char *) * maxn);
    size_t na = 0, ne = 0;
    int bad = 0, lineno = 0;
    for (char *line = buf; line <= buf + len;) {
        char *end = strchr(line, '\n');
        if (end) *end = 0;
        lineno++;
        size_t ll = strlen(line);
        char *arrow = strstr(line, " -> ");
        if (ll == 0 || line[0] == '#') {
        } else if (arrow && valid_name(line, (size_t)(arrow - line)) && valid_name(arrow + 4, strlen(arrow + 4))) {
            *arrow = 0;
            ea[ne] = line;
            eb[ne++] = arrow + 4;
            all[na++] = line;
            all[na++] = arrow + 4;
        } else if (!arrow && valid_name(line, ll)) {
            all[na++] = line;
        } else {
            fprintf(stderr, "error: line %d: bad line\n", lineno);
            bad = 1;
        }
        if (!end) break;
        line = end + 1;
    }
    qsort(all, na, sizeof(char *), cmp_str);
    names = all;
    for (size_t i = 0; i < na; i++)
        if (i == 0 || strcmp(all[i], all[i - 1])) names[nn++] = all[i];
    unsigned char *needs = calloc(nn * nn + 1, 1), *needed = calloc(nn * nn + 1, 1), *seen = calloc(nn + 1, 1);
    for (size_t e = 0; e < ne; e++) {
        int a = find(ea[e]), b = find(eb[e]);
        needs[(size_t)a * nn + b] = 1;
        needed[(size_t)b * nn + a] = 1;
    }
    int rc = bad ? 2 : 0;
    if (cmd >= 2) {
        int s = find(argv[2]);
        if (s < 0) {
            fprintf(stderr, "error: unknown %s\n", argv[2]);
            rc = 4;
        } else {
            reach(cmd == 2 ? needs : needed, s, seen);
            for (size_t i = 0; i < nn; i++)
                if (seen[i]) puts(names[i]);
        }
    } else if (cmd == 1) {
        for (size_t i = 0; i < nn; i++) {
            reach(needs, (int)i, seen);
            if (seen[i]) puts(names[i]);
        }
    } else {
        size_t *left = calloc(nn + 1, sizeof(size_t));
        unsigned char *done = calloc(nn + 1, 1);
        int *out = malloc(sizeof(int) * (nn + 1));
        size_t nout = 0;
        for (size_t i = 0; i < nn; i++)
            for (size_t j = 0; j < nn; j++) left[i] += needs[i * nn + j];
        for (;;) {
            size_t pick = nn;
            for (size_t i = 0; i < nn; i++)
                if (!done[i] && left[i] == 0) { pick = i; break; }
            if (pick == nn) break;
            done[pick] = 1;
            out[nout++] = (int)pick;
            for (size_t j = 0; j < nn; j++)
                if (needs[j * nn + pick]) left[j]--;
        }
        if (nout < nn) {
            fputs("error: cycle\n", stderr);
            rc = 3;
        } else {
            for (size_t i = 0; i < nout; i++) puts(names[out[i]]);
        }
        free(left); free(done); free(out);
    }
    free(needs); free(needed); free(seen); free(all); free(ea); free(eb); free(buf);
    return rc;
}
