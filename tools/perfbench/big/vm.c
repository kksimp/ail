/* C twin of vm.ail. */
#include <stdio.h>
#include <stdlib.h>

static long long code[256];
static int ncode = 0;

static void emit(long long op, long long arg) { code[ncode++] = op; code[ncode++] = arg; }
static int here(void) { return ncode / 2; }

static void build(long long limit) {
    emit(0, 2); emit(2, 0); emit(0, 0); emit(2, 1);
    int top = here();
    emit(1, 0); emit(0, limit); emit(6, 0);
    int jz_end = here(); emit(8, 0);
    emit(0, 2); emit(2, 2);
    int inner = here();
    emit(1, 2); emit(1, 2); emit(4, 0); emit(1, 0); emit(0, 1); emit(3, 0); emit(6, 0);
    int jz_prime = here(); emit(8, 0);
    emit(1, 0); emit(1, 2); emit(5, 0);
    int jz_next = here(); emit(8, 0);
    emit(1, 2); emit(0, 1); emit(3, 0); emit(2, 2); emit(7, inner);
    int prime = here();
    emit(1, 1); emit(0, 1); emit(3, 0); emit(2, 1);
    int next = here();
    emit(1, 0); emit(0, 1); emit(3, 0); emit(2, 0); emit(7, top);
    int end = here(); emit(9, 0);
    code[jz_end * 2 + 1] = end;
    code[jz_prime * 2 + 1] = prime;
    code[jz_next * 2 + 1] = next;
}

static long long run(long long *vars) {
    long long st[64] = {0};
    long long sp = 0, pc = 0, steps = 0;
    for (;;) {
        long long op = code[pc * 2], arg = code[pc * 2 + 1];
        pc++; steps++;
        switch (op) {
        case 0: st[sp++] = arg; break;
        case 1: st[sp++] = vars[arg]; break;
        case 2: vars[arg] = st[--sp]; break;
        case 3: sp--; st[sp - 1] = st[sp - 1] + st[sp]; break;
        case 4: sp--; st[sp - 1] = st[sp - 1] * st[sp]; break;
        case 5: sp--; st[sp - 1] = st[sp - 1] % st[sp]; break;
        case 6: sp--; st[sp - 1] = st[sp - 1] < st[sp] ? 1 : 0; break;
        case 7: pc = arg; break;
        case 8: sp--; if (st[sp] == 0) pc = arg; break;
        case 9: return steps;
        default: abort();
        }
    }
}

int main(int argc, char **argv) {
    long long limit = argc > 1 ? atoll(argv[1]) : 200000;
    build(limit);
    long long vars[3] = {0, 0, 0};
    long long steps = run(vars);
    printf("primes %lld\nsteps %lld\n", vars[1], steps);
    return 0;
}
