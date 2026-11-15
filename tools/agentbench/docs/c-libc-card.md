# C reference card

> C11 with POSIX.1-2008 on Linux and macOS. Non-normative: the C standard and the
> man pages are authoritative. Size-matched to the AIL language card for the agent
> benchmark ([ADR 0916](../../../docs/decisions/0916-agent-end-to-end-benchmark.md)):
> signatures and pitfalls, no solutions.

## Building and running programs

```sh
cc -std=c11 -O1 -g -Wall -Wextra -o prog src/main.c      # build; warnings are bugs
cc -O1 -g -Wall -Wextra -fsanitize=address,undefined -o prog src/main.c   # catches heap/stack overflow, use-after-free, leaks, signed overflow, bad shifts
cc -c a.c && cc -c b.c && cc -o prog a.o b.o -lm         # separate compilation; `-lm` for <math.h>
./prog ARGS < input.txt                                  # arguments in argv, input on stdin
echo $?                                                  # exit status of the last command
gdb ./prog                                               # break main / run / bt / print x / next / step
valgrind ./prog                                          # where installed: leaks and invalid reads
```

A `Makefile` rule is `target: deps` followed by a tab-indented command. Sanitizer
errors print a report and exit nonzero; read the first frame that names your file.

A program needs `int main(void)` or `int main(int argc, char **argv)`. `argv[0]` is
the program name, `argv[1..argc-1]` the arguments, `argv[argc]` is `NULL`. Returning
`n` from `main` is `exit(n)`; 0 is success.

## Layout

- Blocks use `{ }`, statements end in `;`. Indentation is style, not syntax; use 4 spaces.
- `//` and `/* */` comments. Every name must be declared before use: put prototypes
  or `static` functions above their callers.
- `#include <stdio.h>` for system headers, `#include "local.h"` for yours. A header
  needs an include guard (`#ifndef X_H` / `#define X_H` / `#endif`).
- `static` on a file-level function or variable limits it to the file.

## Functions

```c
int add(int a, int b) {              /* `Type name`; return type first */
    return a + b;
}

static void greet(const char *name) {   /* `void`: returns nothing */
    printf("hello, %s\n", name);
}

static int half(int n);              /* a prototype: declare now, define later */
```

Arguments are passed by value (copies). To let a function change a caller's variable
or return several results, pass a pointer: `void bump(int *p) { *p += 1; }` called as
`bump(&n)`. A function must `return` a value on every path unless it is `void`.
Arrays decay to a pointer to the first element when passed; the length is not
passed with them, so pass it too: `int sum(const int *xs, size_t n)`.

## Variables and types

```c
int x = 1;            /* initialize at declaration; locals are otherwise garbage */
const int k = 5;      /* read-only */
size_t i = 0;         /* sizes, lengths and indexes; unsigned */
int a[4] = {1, 2, 3, 4};
int zeros[100] = {0};
```

- Integers: `char`, `short`, `int`, `long`, `long long` and their `unsigned` forms.
  Exact widths come from `<stdint.h>`: `int8_t`…`int64_t`, `uint8_t`…`uint64_t`,
  `INT64_MIN`, `INT64_MAX`, `UINT64_MAX`; `<limits.h>` has `INT_MAX`, `LLONG_MAX`.
  `long` is 64-bit on Linux and macOS but not on Windows; use `long long` or
  `int64_t` when 64 bits are required.
- `float`, `double` (64-bit IEEE). `bool`, `true`, `false` come from `<stdbool.h>`.
- Mixed arithmetic converts to the wider type; if signed and unsigned of equal rank
  meet, the signed one becomes unsigned, so `-1 < 1u` is false. `size_t` minus a
  larger `size_t` wraps to a huge value.
- **Signed overflow is undefined behaviour** (`INT64_MAX + 1`, `abs(INT_MIN)`,
  `-INT64_MIN`). Unsigned arithmetic wraps. To negate or take the absolute value of
  a signed number safely, convert first: `unsigned long long u = x < 0 ? 0ULL - (unsigned long long)x : (unsigned long long)x;`.
- Integer division truncates toward zero, and `%` takes the sign of the dividend:
  `-7 / 2 == -3`, `-7 % 2 == -1`. Division or `%` by zero is undefined.
- Shifts: the count must be `0 <= n < width`; shifting a negative signed value is undefined.
- Struct and array contents are not zeroed by default; use `= {0}` or `calloc`/`memset`.

## Operators

`+ - * / %`, `== != < <= > >=`, `&& || !` (short-circuit; the result is 0 or 1),
`& | ^ ~ << >>`, `++ --`, `+= -= *= /= %= &= |= ^= <<= >>=`, `?:`, `sizeof`, casts
`(type)x`. Any nonzero value is true. Assignment `=` is an expression: `if (x = 1)`
assigns; write `==` to compare. Comparing strings with `==` compares addresses; use
`strcmp`. Precedence trap: `a & b == c` parses as `a & (b == c)`; parenthesize.

## Structs, enums and unions

```c
typedef struct {
    int x;
    int y;
} Point;

typedef enum { RED, GREEN, BLUE } Color;       /* RED == 0, GREEN == 1, ... */

static int point_sum(const Point *p) { return p->x + p->y; }   /* `->` through a pointer */

Point p = {.x = 1, .y = 2};      /* designated initializer; unnamed members are zero */
Point q = p;                     /* struct assignment copies every member */
Point *heap = malloc(sizeof *heap);
heap->x = 3;
```

`==` does not work on structs; compare members. Structs may hold arrays, pointers
and other structs. A `union` shares storage among its members. A flexible array
member (`char data[];`, last member) lets one allocation hold a header and its data.
Tagged unions are written by hand: a struct holding an enum tag plus a union.

```c
typedef struct { enum { CIRCLE, RECT } kind; union { int r; struct { int w, h; } rect; } as; } Shape;
switch (s.kind) { case CIRCLE: ...; break; case RECT: ...; break; }
```

## Pointers, arrays and memory

```c
int *p = &x;         /* address of x */
*p = 5;              /* write through the pointer */
int *q = p + 2;      /* pointer arithmetic counts elements, not bytes */
a[i] == *(a + i);    /* indexing is pointer arithmetic; no bounds check */
```

- `NULL` is the null pointer; dereferencing it is undefined (usually a crash).
- Stack variables vanish when the function returns: never return `&local` or a
  local array. Return a `malloc`'d buffer, take a caller's buffer, or use `static` storage.
- `sizeof a` of an array gives its byte size in its own scope; of a parameter it
  gives a pointer's size. Element count: `sizeof a / sizeof a[0]`.
- `malloc(n)` returns uninitialized memory or `NULL`; `calloc(count, size)` zeroes it and
  checks the multiplication for overflow; `realloc(p, n)` may move the block and
  returns `NULL` on failure while the old block stays valid, so assign to a temporary:
  `char *t = realloc(p, n); if (!t) { free(p); return -1; } p = t;`.
  `free(p)` once per allocation; `free(NULL)` is a no-op; using memory after `free`
  or freeing twice is undefined. Match every `malloc`, `calloc`, `realloc`, `strdup`
  and `getline` buffer with a `free`.
- A growable array is a pointer, a length and a capacity; double the capacity when
  full: `if (len == cap) { cap = cap ? cap * 2 : 8; items = realloc(items, cap * sizeof *items); }`.
- `const char *s = "text";` points at read-only memory; modifying it is undefined.
  `char buf[16] = "text";` is a writable copy.
- Two pointers are comparable or subtractable only inside the same array.
- Function pointers: `int (*cmp)(const void *, const void *)`; call as `cmp(a, b)`.

## Strings

A C string is a `char` array ended by a NUL byte (`'\0'`). It has no length field and
no bounds checks.

```c
#include <string.h>
size_t n = strlen(s);                  /* bytes before the NUL, O(n) */
strcmp(a, b)                            /* <0, 0, >0 by unsigned byte value; 0 means equal */
strncmp(a, b, n)                        /* at most n bytes */
strchr(s, c) / strrchr(s, c)            /* first / last c, or NULL */
strstr(hay, needle)                     /* first occurrence, or NULL */
strdup(s)                               /* malloc'd copy; caller frees (POSIX) */
strndup(s, n)                           /* copy of at most n bytes, always NUL-terminated (POSIX) */
memcpy(dst, src, n)                     /* regions must not overlap */
memmove(dst, src, n)                    /* overlap allowed */
memset(p, byte, n)  memcmp(a, b, n)  memchr(p, byte, n)
strcpy / strcat                          /* no bounds check: avoid */
snprintf(buf, size, fmt, ...)           /* bounded; returns the length it wanted, so >= size means truncated */
strtok(s, delims)                        /* modifies s, keeps hidden state: avoid; split with strchr or strstr */
strspn(s, accept) / strcspn(s, reject)  /* length of a prefix made of / free of those bytes */
```

- Allocate `strlen(s) + 1` bytes for a copy.
- To split on a separator without `strtok`, find it with `strchr`, overwrite it with
  `'\0'`, and continue after it; the pieces are pointers into the original buffer, so
  it must outlive them.
- `char` may be signed: pass `(unsigned char)c` to `<ctype.h>` functions.
- Strings are bytes. UTF-8 characters can be several bytes; `strlen` counts bytes.
- Compare for sorting with `strcmp` (byte order). `strcoll` is locale-dependent.

## Characters

`<ctype.h>`: `isdigit`, `isalpha`, `isalnum`, `isspace`, `isupper`, `islower`,
`isprint`, `toupper`, `tolower`. Each takes an `unsigned char` value (or `EOF`) and
is locale-sensitive beyond ASCII. For strictly ASCII digits write `c >= '0' && c <= '9'`.
Character literals are `int` in C: `'a'` is 97; `'\n'`, `'\t'`, `'\\'`, `'\0'`, `'\x41'`.

## Input and output

```c
#include <stdio.h>
printf("%d %s\n", n, s);                 /* stdout */
fprintf(stderr, "error: %s\n", msg);     /* stderr: unbuffered */
puts(s)                                   /* writes s and a newline */
fputs(s, stream)                          /* writes s, no newline */
putchar(c)  fputc(c, stream)
int c = getchar();  fgetc(stream)         /* returns int: a byte, or EOF; keep it in an int */
fgets(buf, size, stream)                  /* reads at most size-1 bytes, stops after '\n'; NULL at EOF; keeps the '\n' */
ssize_t n = getline(&line, &cap, stream); /* POSIX: grows *line with malloc, keeps '\n', returns -1 at EOF; free(line) at the end */
size_t got = fread(buf, 1, size, stream); /* returns bytes read; fewer than size means EOF or error */
fwrite(buf, 1, n, stream)
FILE *f = fopen(path, "r");               /* "r" "w" "a" "rb" "wb"; NULL on failure, errno set */
fclose(f);  fflush(stream);  feof(f);  ferror(f);
perror("open");                           /* prints a message from errno to stderr */
```

- Format specifiers: `%d` int, `%u` unsigned, `%ld` long, `%lld` long long, `%llu`
  unsigned long long, `%zu` size_t, `%zd` ssize_t, `%x` hex, `%c` char, `%s` string,
  `%f %g %e` double, `%p` pointer, `%%` percent. Width and precision: `%5d`, `%-8s`
  (left-aligned), `%05d` (zero-padded), `%.2f`, `%*d` (width from an argument), `%.*s`.
  `PRId64` and `PRIu64` from `<inttypes.h>` format `int64_t` and `uint64_t`.
  A specifier that does not match its argument is undefined; `-Wall` warns.
- stdout is line-buffered on a terminal and block-buffered in a pipe; output is
  flushed by `exit`, by returning from `main`, or by `fflush`. After `abort` or a
  crash, buffered output may be lost. Errors on stderr appear immediately.
- Read all of stdin: loop `fread` into a buffer that doubles when full, and add a NUL
  at the end if you treat it as text. A trailing newline is part of the data.
- Binary data: open with `"rb"`/`"wb"` and treat the buffer as `unsigned char`.
- `scanf` is rarely the right tool for validation: it leaves bad input unconsumed and
  `%s` has no bound. Read a line, then parse it.

## Numbers from text

```c
#include <stdlib.h>
#include <errno.h>
char *end;
errno = 0;
long long v = strtoll(s, &end, 10);       /* base 10; accepts leading space and + or - */
if (end == s || *end != '\0' || errno == ERANGE) { /* not a clean number, or out of range */ }
unsigned long long u = strtoull(s, &end, 10);   /* accepts a leading '-' and wraps: check yourself */
double d = strtod(s, &end);
```

`atoi` and `atof` cannot report errors; do not use them to validate. `strtoll`
returns `LLONG_MIN` or `LLONG_MAX` and sets `errno` to `ERANGE` on overflow. It
skips leading whitespace and accepts a `+` sign, so check characters first when a
strict format is required.

## Control flow

```c
if (a < b) { ... } else if (a == b) { ... } else { ... }

while (n > 0) { n--; }
do { ... } while (cond);
for (size_t i = 0; i < n; i++) {         /* the loop variable can be declared in the header */
    if (xs[i] == 0) continue;            /* next iteration */
    if (xs[i] > 9) break;                /* leave the loop */
}

switch (c) {
case '+':
case '-':                                /* fall through is the default: end each case with break */
    sign = 1;
    break;
case '0' ... '9':                        /* GNU/clang extension; not standard C */
    digits++;
    break;
default:
    return -1;
}
```

`switch` works on integers and characters only (not strings): compare strings with
`strcmp` in an `if` chain. `goto cleanup;` jumping to a label near the function's end
is the common way to release resources on every error path:

```c
int load(const char *path) {
    int rc = -1;
    FILE *f = fopen(path, "r");
    if (!f) return -1;
    char *buf = malloc(4096);
    if (!buf) goto out;
    /* ... use buf ... */
    rc = 0;
out:
    free(buf);
    fclose(f);
    return rc;
}
```

## Errors and absence

C has no exceptions and no optional types. Conventions:

- Return a status: `0` for success and a negative or nonzero value for failure, with
  results returned through pointer parameters: `int parse(const char *s, long long *out)`.
- Return a sentinel: a pointer function returns `NULL` on failure; `EOF`, `-1`.
- Library failures set `errno` (`<errno.h>`): read it right after the failing call;
  `strerror(errno)` gives text. Successful calls may change `errno`, so reset it to 0
  before calls you check afterwards (`strtoll`).
- Check every `malloc`, `fopen`, `fread` and `write`. An unchecked failure becomes a
  crash far from its cause.
- Report errors on `stderr` and exit with a nonzero status: `fprintf(stderr, "error: ...\n"); exit(2);`.
  `exit` runs `atexit` handlers and flushes streams; `_exit` does not; `abort` raises SIGABRT.
- `assert(cond)` from `<assert.h>` aborts with the file and line when `cond` is false;
  compiled out when `NDEBUG` is defined.

## Sorting, searching and collections

```c
#include <stdlib.h>
qsort(base, count, size, cmp);            /* in place; NOT stable; cmp returns <0, 0, >0 */
void *hit = bsearch(&key, base, count, size, cmp);   /* the array must be sorted by cmp */

static int cmp_int(const void *pa, const void *pb) {
    int a = *(const int *)pa, b = *(const int *)pb;
    return (a > b) - (a < b);             /* never `a - b`: it overflows */
}
static int cmp_str(const void *pa, const void *pb) {
    return strcmp(*(char *const *)pa, *(char *const *)pb);   /* sorting an array of char * */
}
```

- `qsort` is not stable. To keep equal elements in input order, store the original
  index in each element and compare it last.
- C has no list, map or set. Use a growable array (above), a sorted array with
  `bsearch`, a linked list of nodes (`struct Node { struct Node *next; ... }`), or a
  hand-written hash table (an array of buckets, a hash function over the key, chains
  of entries; FNV-1a is a fine byte hash). Find a key in a small array with a linear scan.
- `<search.h>` offers POSIX `hcreate`/`hsearch`, a single global table of fixed size
  with `char *` keys; prefer your own table.
- A graph is an adjacency matrix (`unsigned char *m` indexed `i * n + j`) or per-node
  arrays of neighbour indexes. Convert names to indexes once, then work on integers.
- Memory for many small strings: keep pointers into one input buffer instead of copying.

## Math

`#include <math.h>` and link with `-lm`: `sqrt`, `pow`, `fabs`, `floor`, `ceil`,
`round`, `fmod`, `sin`, `cos`, `exp`, `log`, `isnan`. `abs`/`labs`/`llabs` are in
`<stdlib.h>` (and overflow on the minimum value). Integer helpers `min`/`max` do
not exist; write a macro with parenthesized arguments or a small `static inline`
function. Floating-point division by zero gives infinity, not a trap. `float` and
`double` values are compared with a tolerance, not `==`.

## Time, environment and processes

```c
#include <time.h>
time_t now = time(NULL);                              /* seconds since 1970-01-01 UTC */
struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);   /* ts.tv_sec, ts.tv_nsec: for measuring time */
strftime(buf, size, "%Y-%m-%d %H:%M:%S", gmtime(&now));
const char *home = getenv("HOME");                    /* NULL if unset */
system("ls -l");                                      /* runs through /bin/sh; avoid with untrusted text */
```

Process control and files, POSIX: `<unistd.h>` (`read`, `write`, `close`, `fork`,
`execvp`, `pipe`, `dup2`, `getpid`, `sleep`), `<sys/wait.h>` (`waitpid`), `<fcntl.h>`
(`open` with `O_RDONLY`, `O_WRONLY|O_CREAT|O_TRUNC`), `<sys/stat.h>` (`stat`,
`mkdir`), `<dirent.h>` (`opendir`, `readdir`, `closedir`), `<signal.h>` (`signal`,
`sigaction`). `read` and `write` may transfer fewer bytes than asked: loop. Check
`fork` for `-1`, `0` (child) and a positive pid (parent); a child calls `_exit` if
`exec` fails.

```c
int fds[2]; pipe(fds);                    /* fds[0] reads, fds[1] writes */
pid_t pid = fork();
if (pid == 0) { dup2(fds[1], 1); close(fds[0]); execvp("ls", (char *[]){"ls", "-l", NULL}); _exit(127); }
int status; waitpid(pid, &status, 0);
if (WIFEXITED(status)) code = WEXITSTATUS(status);
```

## Threads and atomics

`<pthread.h>` (link with `-pthread`): `pthread_create(&t, NULL, fn, arg)` where
`fn` is `void *fn(void *)`; `pthread_join(t, &result)`; `pthread_mutex_t m =
PTHREAD_MUTEX_INITIALIZER; pthread_mutex_lock(&m); ... pthread_mutex_unlock(&m);`.
A data race on any shared variable is undefined behaviour: guard it with a mutex,
or use `<stdatomic.h>` (`atomic_int n; atomic_fetch_add(&n, 1);`). Arguments to a
thread must stay alive until it finishes: do not pass `&local` of a function that returns first.
`pthread_cond_wait` needs the mutex held and a loop around the predicate.

## Preprocessor and headers

`#define N 100`, function-like macros need parentheses around every parameter and the whole
body: `#define SQUARE(x) ((x) * (x))` (arguments evaluate twice). `#ifdef`/`#ifndef`/
`#if`/`#else`/`#endif` select code. `static inline` functions are usually better than macros.
`_Static_assert(cond, "message")` checks at compile time. `<stddef.h>` has `size_t`,
`ptrdiff_t`, `NULL`, `offsetof`. `<stdbool.h>`, `<stdint.h>`, `<inttypes.h>`,
`<limits.h>`, `<float.h>` as above. Feature macros: define `_POSIX_C_SOURCE 200809L`
or `_GNU_SOURCE` before the first include to get `getline` and `strdup` under `-std=c11`.

## Tests

C has no built-in test runner. The usual pattern is a program or a `make test`
target that calls functions and checks results:

```c
#include <assert.h>
static int clamp(int x, int lo, int hi) { return x < lo ? lo : x > hi ? hi : x; }
static void test_clamp(void) {
    assert(clamp(5, 0, 3) == 3);
    assert(clamp(-1, 0, 3) == 0);
}
int main(void) { test_clamp(); puts("ok"); return 0; }
```

Black-box tests run the built program with arguments and input and compare its
output and exit status with the expected values, for example
`printf 'a,b\n' | ./prog cmd > out.txt; echo $? ; diff out.txt want.txt`.
Build with the sanitizers on while testing: they turn silent memory errors into
reports with a stack trace and the line of the bug.

## Command-line arguments and exit status

```c
int main(int argc, char **argv)        /* argv[0] is the program name; argv[argc] is NULL */
if (argc < 2) { fprintf(stderr, "usage: %s CMD\n", argv[0]); return 2; }
if (strcmp(argv[1], "list") == 0) { ... }
exit(EXIT_SUCCESS);  exit(EXIT_FAILURE);  /* 0 and 1, from <stdlib.h> */
```

- Returning `n` from `main` is the same as `exit(n)`; only the low 8 bits reach the
  shell (`$?`), so use 0 to 125.
- `exit` flushes and closes stdio streams and runs `atexit` handlers; `_exit` and
  `abort` do not flush.
- Arguments are bytes with no parsing done: `"5"` is a string; convert it with
  `strtoll` and check `end` as shown above.
- `getopt(argc, argv, "vo:")` (POSIX, `<unistd.h>`) parses short options: it returns
  each option letter, sets `optarg` for an option that takes a value, returns `'?'`
  for an unknown one and `-1` at the end; `optind` is then the first operand.
  macOS `getopt` stops at the first non-option argument; glibc permutes unless
  `POSIXLY_CORRECT` is set or the option string starts with `+`.
- A usage error conventionally prints to stderr and exits 2; keep stdout for the
  program's output so it can be piped.

## Ownership and cleanup

C has no destructors: every `malloc`, `calloc`, `realloc`, `strdup`, `getline` and
`fopen` needs one matching `free` or `fclose` on every path, including error paths.

- Decide one owner per allocation and write it in a comment on the struct field
  (`/* owned */` or `/* borrowed from buf */`).
- A pointer into a buffer (`strchr` result, a field split in place) is borrowed: it
  dies when the buffer is freed or `realloc` moves it. After `realloc`, recompute
  every pointer into the old block from saved offsets.
- `free(NULL)` is a no-op, so initialise pointers to `NULL` and free them all at one
  exit label:

```c
int run(const char *path) {
    int rc = 1;
    FILE *f = NULL; char *line = NULL; size_t cap = 0;
    if (!(f = fopen(path, "r"))) { perror(path); goto out; }
    while (getline(&line, &cap, f) != -1) { /* ... */ }
    if (ferror(f)) goto out;
    rc = 0;
out:
    free(line);
    if (f) fclose(f);
    return rc;
}
```

- `p = realloc(p, n)` leaks the old block when it fails; assign to a temporary first.
- `calloc(n, size)` zeroes memory and checks `n * size` for overflow; `malloc(n * size)`
  does neither.
- At process exit the OS reclaims memory, so a short-lived CLI may skip freeing at
  the very end; LeakSanitizer (part of ASan on Linux) still reports it unless
  `ASAN_OPTIONS=detect_leaks=0`.

## Reading compiler and sanitizer reports

Warnings from `-Wall -Wextra` name the file, line, column and a flag:

```
src/main.c:42:13: warning: comparison of integers of different signs [-Wsign-compare]
src/main.c:57:5: warning: 'n' may be used uninitialized [-Wmaybe-uninitialized]
src/main.c:12:1: warning: control reaches end of non-void function [-Wreturn-type]
src/main.c:30:20: warning: format '%d' expects argument of type 'int', but argument 2 has type 'long long' [-Wformat=]
src/main.c:8:6: warning: implicit declaration of function 'strdup' [-Wimplicit-function-declaration]
```

- Treat every warning as a bug until shown otherwise; `-Werror` makes them errors.
- An implicit declaration means a missing `#include` (or, for POSIX functions with
  `-std=c11`, a missing `#define _POSIX_C_SOURCE 200809L` before the first include).
- "undefined reference to `sqrt`" at link time means a missing `-lm` (Linux).

AddressSanitizer stops the program at the first memory error and prints a report to
stderr; the first stack frame in your own file is usually the bug:

```
==1234==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x6020000000f1
READ of size 1 at 0x6020000000f1 thread T0
    #0 0x4f2a1c in parse_row src/main.c:88
    #1 0x4f3b20 in main src/main.c:140
0x6020000000f1 is located 0 bytes to the right of 1-byte region allocated by:
    #1 0x4f2900 in read_all src/main.c:61
```

- `heap-buffer-overflow` / `stack-buffer-overflow`: an index or length one past the
  end ("0 bytes to the right" is the classic missing NUL or `<=` loop bound).
- `heap-use-after-free`: the report shows where the block was freed and where it was
  allocated; a borrowed pointer outlived its buffer, often after `realloc`.
- `attempting double-free`, `SEGV on unknown address 0x000000000000` (a NULL
  dereference), `detected memory leaks` (the allocation stack of each leak).
- UndefinedBehaviorSanitizer prints `runtime error:` lines such as
  `signed integer overflow: 9223372036854775807 + 1 cannot be represented in type
  'long long'`, `shift exponent 64 is too large`, `load of misaligned address`, and
  `index 10 out of bounds for type 'int [10]'`. By default it prints and continues;
  the exit status is unchanged unless `-fno-sanitize-recover` is given.
- A sanitizer report goes to stderr and the program exits non-zero (ASan uses 1), so
  a test that checks the exit code fails even when stdout matched.

## Undefined behaviour checklist

Undefined behaviour may appear to work, then change with the optimisation level or
the input. The common sources in small programs:

- Reading or writing outside an array or allocation, including one byte past a
  string's NUL.
- Using memory after `free`, freeing twice, or freeing a pointer not from the allocator.
- Reading an uninitialised local or a `malloc`'d byte before writing it.
- Signed overflow, division by zero, `INT_MIN / -1`, oversized shifts.
- Dereferencing `NULL`, or a pointer to a local whose function has returned.
- Modifying a string literal; `memcpy` with overlapping regions.
- Modifying a variable twice without a sequence point: `i = i++`, `a[i] = i++`.
- Calling a `<ctype.h>` function with a negative `char` other than `EOF`.
- A `printf` specifier that does not match its argument's type.

## Recipes: generic containers and multi-key ordering

A typed growable array with macros (one definition per element type):

```c
#define VEC(T) struct { T *items; size_t len, cap; }
#define VEC_PUSH(v, x) do { \
    if ((v).len == (v).cap) { \
        size_t nc = (v).cap ? (v).cap * 2 : 8; \
        void *t = realloc((v).items, nc * sizeof *(v).items); \
        if (!t) abort(); \
        (v).items = t; (v).cap = nc; } \
    (v).items[(v).len++] = (x); } while (0)

typedef VEC(int) IntVec;
IntVec xs = {0};
VEC_PUSH(xs, 42);
free(xs.items);
```

A `qsort` comparator over several keys returns at the first key that differs; compare
with `<` and `>` rather than subtracting, which can overflow:

```c
typedef struct { const char *name; long long size; int id; } Item;
static int cmp_item(const void *pa, const void *pb) {
    const Item *a = pa, *b = pb;
    if (a->size != b->size) return a->size > b->size ? -1 : 1;   /* size, descending */
    int c = strcmp(a->name, b->name);                              /* then name, ascending */
    if (c) return c;
    return (a->id > b->id) - (a->id < b->id);                      /* then id: a total order */
}
qsort(items, n, sizeof *items, cmp_item);
```

`qsort` is not stable: when equal elements must keep their input order, add the
input index as the last key. Sort an array of pointers (`Item **`) instead of the
structs when they are large; the comparator then receives `Item *const *`.

## Common mistakes

- Off-by-one on strings: forgetting the NUL byte when allocating or copying.
- Using a value after `free`, returning a pointer to a local, or freeing something
  not from `malloc`.
- Reading an uninitialized variable or a `malloc`'d buffer before writing it.
- Storing `getchar()`/`fgetc` in a `char`: EOF is an `int` and gets confused with a byte.
- Comparing signed with unsigned values; using `int` for sizes and indexes that may exceed 2 billion.
- Forgetting `break` in a `switch`; `=` instead of `==`.
- Passing `%d` for a `long long` or `size_t`; `%s` for a non-string.
- `sizeof` of a pointer parameter instead of the array it was passed from.
- Assuming `qsort` is stable or that `strtok` is safe to nest.
- Ignoring return values of `fread`, `fwrite`, `fclose`, `malloc` and `realloc`.
- Signed overflow in sums, products and absolute values of untrusted input; check
  before the operation with `__builtin_add_overflow(a, b, &r)` (GCC and clang) or
  compare against `INT64_MAX - b` first.

## Recipes: building blocks C lacks

A growable byte buffer (string builder):

```c
typedef struct { char *data; size_t len, cap; } Buf;

static int buf_add(Buf *b, const char *s, size_t n) {
    if (b->len + n + 1 > b->cap) {
        size_t cap = b->cap ? b->cap : 64;
        while (cap < b->len + n + 1) cap *= 2;
        char *t = realloc(b->data, cap);
        if (!t) return -1;
        b->data = t;
        b->cap = cap;
    }
    memcpy(b->data + b->len, s, n);
    b->len += n;
    b->data[b->len] = '\0';
    return 0;
}
```

A string-keyed hash table with chaining (insert or find):

```c
typedef struct Entry { char *key; long long value; struct Entry *next; } Entry;
typedef struct { Entry **slots; size_t nslots, count; } Table;   /* nslots a power of two */

static size_t hash(const char *s) {            /* FNV-1a */
    size_t h = 14695981039346656037ULL;
    for (; *s; s++) { h ^= (unsigned char)*s; h *= 1099511628211ULL; }
    return h;
}

static Entry *table_get(Table *t, const char *key, int create) {
    size_t i = hash(key) & (t->nslots - 1);
    for (Entry *e = t->slots[i]; e; e = e->next)
        if (strcmp(e->key, key) == 0) return e;
    if (!create) return NULL;
    Entry *e = calloc(1, sizeof *e);
    e->key = strdup(key);
    e->next = t->slots[i];
    t->slots[i] = e;
    t->count++;
    return e;
}
```

To list a table in sorted order, copy its entries into an array and `qsort` it.
Free every key and entry when done (walk each chain).

A stack is an array plus a length (`push`: `xs[n++] = v`, `pop`: `xs[--n]`); a queue is an
array plus head and tail indexes. A visited set for small integer ids is an
`unsigned char seen[N]` array, zeroed with `memset`.

Splitting a buffer into lines in place:

```c
for (char *line = buf; line && *line; ) {
    char *nl = strchr(line, '\n');
    if (nl) *nl = '\0';
    /* use line */
    line = nl ? nl + 1 : NULL;
}
```

Splitting a line on a one-byte separator keeps empty fields, which `strtok` would skip:
find each separator with `strchr`, write `'\0'` over it, and record the field start.
Count fields as you go and stop at the limit you expect, so a malformed line cannot
overrun a fixed array of field pointers.

Reading a whole file or stdin into memory:

```c
size_t cap = 4096, len = 0, got;
char *buf = malloc(cap);
while ((got = fread(buf + len, 1, cap - len - 1, stdin)) > 0) {
    len += got;
    if (len + 1 >= cap) buf = realloc(buf, cap *= 2);
}
buf[len] = '\0';
```

Binary search over a sorted array of strings, returning an index or -1:

```c
static int find_name(char **names, size_t n, const char *key) {
    size_t lo = 0, hi = n;
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        int c = strcmp(names[mid], key);
        if (c == 0) return (int)mid;
        if (c < 0) lo = mid + 1; else hi = mid;
    }
    return -1;
}
```

Removing duplicates from a sorted array is a single pass that keeps an output index
`w`: copy `xs[i]` to `xs[w++]` when `i == 0 || xs[i] differs from xs[i-1]`.

Printing a 64-bit integer in decimal with a fixed number of fractional digits is done
with integer division and remainder on an unsigned copy of the value; do not go
through `double`, which holds only 53 bits exactly.

Exit status values are 0 to 255; by convention 0 is success, 1 a general failure and
2 misuse or bad input. A shell sees `exit(256)` as 0.

## Multi-file programs

The build compiles every `src/*.c` file in one command, so a program can be split
freely. Each module is a `.c` file with a header that declares what it shares.

```c
/* src/stock.h */
#ifndef STOCK_H
#define STOCK_H
#include <stddef.h>
typedef struct { char sku[32]; long long qty; } Item;
int stock_load(const char *text, Item *out, size_t max);   /* declaration only */
extern int stock_verbose;                                  /* defined in exactly one .c */
#endif

/* src/stock.c */
#include "stock.h"
int stock_verbose = 0;
static int parse_qty(const char *s, long long *out);       /* private to this file */
int stock_load(const char *text, Item *out, size_t max) { /* ... */ return 0; }
```

- A function used by another file needs a declaration in a header that both include;
  a function used only in its file is `static`.
- A global variable is declared `extern` in the header and defined once in one `.c`;
  defining it in the header gives a "multiple definition" link error.
- Struct types shared between files are defined in the header; an opaque type is
  `typedef struct Table Table;` in the header and the full definition in one `.c`.
- `-Wall -Wextra` warns about a call to an undeclared function ("implicit declaration"),
  which in C11 is an error in practice: add the header include.
- Link errors name the missing symbol: "undefined reference to `f`" means no file
  defines it (or it is `static` elsewhere); "multiple definition" means two files do.
- Renaming a function or a field: change the header, then build; every stale use is a
  compile error. `grep -rn 'name' src/` finds uses in comments and strings too.

## Reading all of standard input

```c
char *read_all(FILE *f, size_t *len_out) {
    size_t cap = 4096, len = 0;
    char *buf = malloc(cap);
    if (!buf) return NULL;
    size_t n;
    while ((n = fread(buf + len, 1, cap - len - 1, f)) > 0) {
        len += n;
        if (cap - len - 1 == 0) {
            char *t = realloc(buf, cap * 2);
            if (!t) { free(buf); return NULL; }
            buf = t; cap *= 2;
        }
    }
    buf[len] = '\0';
    if (len_out) *len_out = len;
    return buf;
}
```

- `getline(&line, &cap, stdin)` (POSIX) reads one line of any length, keeping the
  `\n`; it returns `-1` at end of input. Free `line` once after the loop.
- `fgets(buf, sizeof buf, stdin)` truncates long lines silently; check for the `\n`.
- Strip a trailing `\r` too when input may come from Windows (`"a\r\n"`).

## Money and fixed-point decimals

Store amounts as integer minor units (cents) in `long long`; never in `double`.

```c
/* "12.34" or "-0.5" -> cents; returns 0 on success */
int parse_cents(const char *s, long long *out) {
    int neg = (*s == '-'); if (neg || *s == '+') s++;
    long long whole = 0, frac = 0; int digits = 0;
    if (!isdigit((unsigned char)*s)) return -1;
    while (isdigit((unsigned char)*s)) { whole = whole * 10 + (*s++ - '0'); }
    if (*s == '.') { s++; while (isdigit((unsigned char)*s) && digits < 2) { frac = frac * 10 + (*s++ - '0'); digits++; } }
    if (*s != '\0') return -1;
    if (digits == 1) frac *= 10;
    *out = (whole * 100 + frac) * (neg ? -1 : 1);
    return 0;
}
printf("%s%lld.%02lld\n", c < 0 ? "-" : "", llabs(c) / 100, llabs(c) % 100);
```

- `/` on integers truncates toward zero, also for negatives: `-7 / 2` is `-3`, and `%`
  takes the sign of the left operand (`-7 % 2` is `-1`).
- Rounding half away from zero when dividing `a` by positive `b`:
  `(a >= 0 ? a + b / 2 : a - b / 2) / b`. Half to even needs the remainder compared
  with `b - remainder` explicitly.
- Percentages: multiply before dividing (`amount * pct / 100`) and check the product
  for overflow first.
- `printf("%.2f", x)` on a `double` rounds the binary value, so money printed from
  doubles can be off by a cent.

## Dates as text

- ISO dates `YYYY-MM-DD` compare correctly with `strcmp`, so they sort with `qsort`
  and `strcmp` without parsing.
- Validate with `sscanf(s, "%4d-%2d-%2d%n", &y, &m, &d, &n) == 3 && s[n] == '\0'`
  plus range checks; `sscanf` alone accepts `2024-1-5` and trailing junk.
- Days in a month: `{31,28,31,30,31,30,31,31,30,31,30,31}`, plus one in February when
  `(y % 4 == 0 && y % 100 != 0) || y % 400 == 0`.
- `mktime` and `strptime` depend on the local time zone; for whole-day arithmetic,
  count days from a fixed epoch with your own function instead.

## Error messages and exit codes

```c
static void die(int code, const char *fmt, ...) {
    va_list ap; va_start(ap, fmt);
    vfprintf(stderr, fmt, ap); va_end(ap);
    fputc('\n', stderr);
    exit(code);                                  /* flushes stdout first */
}
```

- `#include <stdarg.h>` for `va_list`. Messages go to `stderr`; results to `stdout`.
- `exit(n)` flushes and closes stdio streams; `_exit(n)` does not.
- When a spec says "print an error and continue", count the errors and choose the
  status at the end; `return 1;` from `main` is the same as `exit(1)`.
- Output written with `printf` before a crash can be lost while it sits in the buffer;
  `setvbuf(stdout, NULL, _IOLBF, 0)` makes it line-buffered while debugging.

## Working in an existing codebase

- Read the headers first: they list what each file offers. `grep -n 'name(' src/*.c`
  lists the callers of a function.
- A bug "far from its cause" is often in a shared helper that several callers work
  around; fix the helper and remove the workarounds, or callers double-correct.
- After a change, rebuild with the sanitizers on (the build line already has them) and
  rerun the tests: a heap-buffer-overflow report names the exact line.
- Keep each change compiling: change one interface, fix every error it causes, rebuild.

## Splitting fields

```c
/* split `line` on single spaces into at most `max` fields; returns the count, or -1 on an empty field */
int split_fields(char *line, char **f, int max) {
    int n = 0;
    for (char *p = line; ; ) {
        if (n == max) return -1;
        f[n++] = p;
        char *sp = strchr(p, ' ');
        if (!sp) break;
        *sp = '\0';
        p = sp + 1;
    }
    for (int i = 0; i < n; i++) if (f[i][0] == '\0') return -1;
    return n;
}
```

- `strtok` and `strtok_r` merge runs of separators and skip leading ones, so they never
  report an empty field; use `strchr` or `strsep` when empty fields are errors.
- `strtok` keeps hidden state: a nested or interleaved `strtok` on another string breaks
  the outer loop. `strtok_r(s, " ", &save)` is the reentrant form.
- `sscanf(line, "%31s %lld", sku, &qty)` skips any whitespace, accepts `+5` and leading
  zeros, and ignores trailing text unless `%n` is checked against the length.
- A field count check comes before any conversion, so the error message can name it.

## Sets and de-duplication

- Small integer keys: a `bool seen[N]` or bitset array.
- Strings: sort an array of `char *` with `qsort` and `strcmp`, then skip equal
  neighbours; or use the hash table recipe above with the value unused.
- Sorting an array of pointers needs a comparator that dereferences twice:
  `static int cmp_str(const void *a, const void *b) { return strcmp(*(char *const *)a, *(char *const *)b); }`.
- Stable order by insertion: keep a counter in each entry and use it as the last key
  in the comparator, since `qsort` is not stable.
