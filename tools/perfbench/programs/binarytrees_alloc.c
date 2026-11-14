/* Binary trees, same allocation as binarytrees.ail: every tree gets fresh
   left/right arrays that grow by doubling as nodes are pushed, and are freed
   when the tree is done. (binarytrees.c reuses one arena, best effort.) */
#include <stdio.h>
#include <stdlib.h>

static int *left;
static int *right;
static int used;
static int cap;

static int make(int depth) {
    int node = used;
    if (used == cap) {
        cap = cap ? cap * 2 : 4;
        left = realloc(left, cap * sizeof(int));
        right = realloc(right, cap * sizeof(int));
    }
    used += 1;
    left[node] = -1;
    right[node] = -1;
    if (depth > 0) {
        int l = make(depth - 1);
        int r = make(depth - 1);
        left[node] = l;
        right[node] = r;
    }
    return node;
}

static long long check(int node) {
    if (left[node] < 0) {
        return 1;
    }
    return 1 + check(left[node]) + check(right[node]);
}

int main(int argc, char **argv) {
    int max_depth = argc > 1 ? atoi(argv[1]) : 12;
    int min_depth = 4;
    for (int d = min_depth; d <= max_depth; d += 2) {
        long long iters = 1LL << (max_depth - d + min_depth);
        long long total = 0;
        for (long long t = 0; t < iters; t++) {
            used = 0;
            cap = 0;
            left = NULL;
            right = NULL;
            int root = make(d);
            total += check(root);
            free(left);
            free(right);
        }
        printf("%lld trees of depth %d check: %lld\n", iters, d, total);
    }
    return 0;
}
