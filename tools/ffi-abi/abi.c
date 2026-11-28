/* The C side of the FFI ABI check (ffi.struct.1-3, ffi.cb.1-4, ADR 1074):
   structs of every class each platform convention has, passed and returned
   by value, and C code that calls AIL callbacks. Built as a shared library
   by run.sh; tools/ffi-abi/abi.ail calls it and abi.out is the expected text. */
#include <stdbool.h>
#include <stdint.h>

#ifdef _WIN32
#define API __declspec(dllexport)
#else
#define API __attribute__((visibility("default")))
#endif

typedef struct { int32_t a; int32_t b; } I2;              /* 8: one integer register */
typedef struct { int64_t a; int64_t b; int64_t c; } I3;   /* 24: memory or by reference */
typedef struct { double x; double y; } D2;                /* 16: two SSE / an HFA */
typedef struct { float x; float y; float z; } F3;         /* 12: packed singles / an HFA */
typedef struct { double a; double b; double c; double d; } D4; /* 32: an HFA of four */
typedef struct { int32_t i; float f; } Mix;               /* 8: integer class */
typedef struct { double d; int64_t n; } DN;               /* 16: SSE then INTEGER */
typedef struct { uint8_t a; uint16_t b; int8_t c; } Tiny; /* 6, aligned 2 */
typedef struct { I2 in; double w; } Nest;                 /* 16, nested */
typedef struct { bool ok; int64_t v; } BV;                /* 16 */

API I2 i2_make(int32_t a, int32_t b) { I2 r = { a, b }; return r; }
API int64_t i2_sum(I2 p) { return (int64_t)p.a + p.b; }
API I3 i3_scale(I3 p, int64_t k) { I3 r = { p.a * k, p.b * k, p.c * k }; return r; }
API int64_t i3_sum(I3 p) { return p.a + p.b + p.c; }
API D2 d2_add(D2 a, D2 b) { D2 r = { a.x + b.x, a.y + b.y }; return r; }
API F3 f3_scale(F3 v, float k) { F3 r = { v.x * k, v.y * k, v.z * k }; return r; }
API D4 d4_make(double s) { D4 r = { s, s * 2, s * 3, s * 4 }; return r; }
API double d4_sum(D4 v) { return v.a + v.b + v.c + v.d; }
API Mix mix_swap(Mix m) { Mix r = { (int32_t)m.f, (float)m.i }; return r; }
API DN dn_make(double d, int64_t n) { DN r = { d, n }; return r; }
API Tiny tiny_echo(Tiny t) { Tiny r = { (uint8_t)(t.a + 1), (uint16_t)(t.b + 1), (int8_t)(t.c - 1) }; return r; }
API Nest nest_make(int32_t a, int32_t b, double w) { Nest r = { { a, b }, w }; return r; }
API double nest_sum(Nest n) { return n.in.a + n.in.b + n.w; }
API BV bv_flip(BV x) { BV r = { !x.ok, -x.v }; return r; }

/* Registers run out: the later structs and scalars go on the stack. */
API double many(int64_t a, D2 b, I3 c, int64_t d, D2 e, int64_t f, int64_t g, F3 h,
                int64_t i, int64_t j, double k) {
    return a + b.x + b.y + c.a + c.b + c.c + d + e.x + e.y + f + g + h.x + h.y + h.z + i + j + k;
}
API double hfa_spill(D4 a, D4 b, float c, int32_t d, D2 e) {
    return d4_sum(a) * 1000 + d4_sum(b) * 100 + c * 10 + d + e.x - e.y;
}

/* Callbacks. */
API int64_t apply_cb(int64_t (*f)(int64_t, double), int64_t x, double y) { return f(x, y) + 1; }
API int32_t call_narrow(int32_t (*f)(int8_t, uint16_t, bool, float)) { return f(-5, 65535, true, 2.5f); }
API int64_t call6(int64_t (*f)(int64_t, int64_t, int64_t, int64_t, int64_t, double)) {
    return f(1, 2, 3, 4, 5, 0.5);
}
API int64_t repeat_cb(int64_t (*f)(int64_t), int64_t n) {
    int64_t s = 0;
    for (int64_t i = 0; i < n; i++) s += f(i);
    return s;
}

/* More than six integer-like arguments (ffi.decl.3): the rest go on the stack. */
API int64_t ints12(int64_t a, int64_t b, int64_t c, int64_t d, int64_t e, int64_t f,
                   int64_t g, int64_t h, int64_t i, int64_t j, int64_t k, int64_t l) {
    return a + 2 * b + 3 * c + 4 * d + 5 * e + 6 * f + 7 * g + 8 * h + 9 * i + 10 * j + 11 * k + 12 * l;
}
/* Narrow integers, floats and a pointer among the stack arguments (Apple arm64 packs them). */
API double mix14(int32_t a, double b, uint8_t c, int64_t d, float e, int16_t f, int64_t g,
                 bool h, int64_t i, int32_t j, double k, int8_t l, uint16_t m, const int64_t *p) {
    return a + b * 2 + c * 3 + d * 4 + e * 5 + f * 6 + g * 7 + (h ? 8 : 0) + i * 9 + j * 10 + k * 11 + l * 12 + m * 13 + p[1] * 14;
}
/* The shape of cblas_sgemm: twelve integer-like and two float parameters. */
API float sgemm_like(int order, int ta, int tb, int m, int n, int k, float alpha,
                     const int32_t *a, int lda, const int32_t *b, int ldb, float beta, int32_t *c, int ldc) {
    int32_t s = 0;
    for (int i = 0; i < m * n && i < 4; i++) s += a[i] * b[i];
    c[0] = s;
    return order * 1 + ta * 2 + tb * 3 + m * 4 + n * 5 + k * 6 + lda * 7 + ldb * 8 + ldc * 9 + alpha * s + beta;
}
