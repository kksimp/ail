/* Constants, enums, function-pointer types, a union and a bit-field struct (ffi.decl.4, ffi.decl.5). */
#ifndef CONSTS_H
#define CONSTS_H

#define BUF 4096
#define FLAGS (1 << 3 | 0x10)
#define NEG -7
#define PI 3.5
#define GREETING "hi"
#define TWICE(x) ((x) * 2)

enum color { RED, GREEN = 5, BLUE };
typedef enum { SLOW = 1, FAST = 1 << 1 } speed_t2;

union val { int i; float f; };
struct flags { unsigned a : 3; unsigned b : 5; };

typedef int (*visit_fn)(int);

int abs(int x);
void walk(visit_fn f, void *user);
int pick(int (*cb)(int), int x);
int paint(enum color c, speed_t2 s);

#endif
