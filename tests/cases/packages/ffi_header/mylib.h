/* The declarations of a few C library functions, as a header writes them (ffi.decl.1). */
#ifndef MYLIB_H
#define MYLIB_H

#ifdef __cplusplus
extern "C" {
#endif

int abs(int x);
int toupper(int c);
double hypot(double a, double b);
int atoi(const char *s);

/* Left out: struct by value and a variadic function (ffi.decl.3). */
struct pair { int a; int b; };
struct pair swap(struct pair p);
int printf(const char *format, ...);

#ifdef __cplusplus
}
#endif

#endif
