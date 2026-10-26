# C reference card (C11, POSIX; compiled with `cc -Wall -Wextra -fsanitize=address,undefined`)

Placeholder for the parity-matched C reference. It must reach the AIL language card's token count (`agentbench.py docs TASK` checks) before any trial counts. Grow it with libc signatures and pitfalls only, never solutions.

## Headers and calls most programs need
- `<stdio.h>`: `fgets(buf, n, stream)`, `getline(&line, &cap, stream)` (POSIX, malloc'd), `fprintf(stderr, ...)`, `printf`, `fputs`, `puts`, `snprintf(buf, n, fmt, ...)`. `%lld` takes `long long`; `%zu` takes `size_t`.
- `<stdlib.h>`: `malloc`, `calloc`, `realloc`, `free`, `strtoll(s, &end, 10)` (check `end`, `errno == ERANGE`), `qsort(base, n, size, cmp)` (not stable), `exit`.
- `<string.h>`: `strlen`, `strcmp` (byte order), `memcmp`, `strchr`, `strdup`, `memcpy`, `memmove`, `memset`; `strtok` mutates and is not reentrant.
- `<stdint.h>`: `int64_t`, `INT64_MAX`; signed overflow is undefined behaviour.
- `<errno.h>`, `<ctype.h>` (`isdigit` takes an `unsigned char` value).

## Pitfalls
- Every `malloc` needs a `free`; ASan reports leaks and out-of-bounds at run time.
- Strings need a trailing NUL; `getline` keeps the newline.
- `qsort` comparators must not subtract ints (overflow); compare with `<` and `>`.
- Exit with `return N` from `main` or `exit(N)`.
