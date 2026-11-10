# 1010. C header import breadth: `#define` constants, enums and function-pointer types

- **Status:** Accepted (extends [ADR 0926](0926-c-header-import-subset.md))
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.decl.4, ffi.decl.5](../spec/14-ffi.md)

## Context

ADR 0926 imports only function prototypes. A real C library's API is also its
constants (`O_RDONLY`, `SQLITE_OK`), enums and callback types, so most headers
import with half their surface missing. The slice asked for `#define`
constants, unions, bit-fields, enums and function-pointer types.

## Options considered

### Option A — constants, enums and opaque function pointers now; struct-shaped items later (chosen)
- `#define NAME body` (object-like) with an integer, float or string body becomes `pub NAME := value`; integer bodies are evaluated by a small evaluator (literals, earlier constants, parentheses, unary `- + ~`, `* / % + - << >> & ^ |`).
- `enum` and `typedef enum` enumerators become integer constants; a `typedef` enum name is `i32`.
- A function-pointer parameter or `typedef` is `*u8`.
- Unions behind a pointer were already `*u8`; by value they stay skipped.
- **Pros:** needs no language change; every item is a value or an opaque pointer, so the generated module is still only constants and `extern c fn`.
- **Cons:** an integer constant is an `int`, so a call to an `i32` parameter writes `.to_i32()`; no callback can be made from AIL yet.

### Option B — also map structs, unions and bit-fields to AIL types now
- **Pros:** complete.
- **Cons:** needs C layout for AIL structs (`@layout(c)`, ADR 0952, not on main), and bit-field accessors need a layout rule of their own. Not doable without that language feature.

### Option C — typed constants (`pub NAME := 5.to_i32()`)
- **Pros:** no `.to_i32()` at the call.
- **Cons:** no precedent for a conversion in a constant, and a header constant is also used as an `int` in arithmetic; rejected until `ct.const` lists it.

## Decision

Option A. The rules are `ffi.decl.4` (constants) and `ffi.decl.5` (function pointers and unions). A constant that cannot be read exactly is left out, never guessed. Values beyond 2^61 are left out so no evaluation can overflow.

## Follow-ups

Unions and bit-field structs by value, and C callbacks from AIL, wait for `@layout(c)` (ADR 0952) and a callback ADR.

## Rejected alternatives

- A full C preprocessor for `#define` (still rejected, ADR 0926 option B).
- Typed constants (option C) and struct mapping now (option B), above.
