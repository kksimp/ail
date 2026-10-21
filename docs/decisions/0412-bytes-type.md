# 0412. `bytes` is an immutable value type of `u8`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [type.bytes.1–7](../spec/04-types.md#bytes)


## Context

AIL has only text (`str`, UTF-8) and `List[u8]`. Files, sockets, hashes and
compressed data are bytes, and a `str` can't hold them. Lanes 124 (sockets)
and 128 (hashing, compression) build on a byte type, and so do `std.fs` and
`std.net` for non-text bodies.

## Options considered

### Option A — `bytes` as a separate immutable value type (chosen)
- **Tokens:** `bytes` is one word; `b[i]`, `b[a..b]`, `a + b` cost the same
  as the list forms.
- **AI understanding:** a type that can't hold text and can't be mutated
  leaves nothing to guess: no encoding question, no aliasing.
- **Speed:** a flat `Vec<u8>` (1 byte per element) against a list of 16-byte
  values; slices and joins are `memcpy`.

### Option B — use `List[u8]`
- **Cons:** 16 times the memory in the interpreter, no literal, no `find`
  or `starts_with`, and nothing separates text from data.

### Option C — a mutable `bytes` (a buffer)
- **Cons:** AIL values are independent (ADRs 0060–0063), so a buffer
  would need `mut` rules of its own. The common cases (read, slice, join)
  don't need mutation; `List[u8]` converts when they do.

### Option D — allow `str` to hold invalid UTF-8
- **Cons:** breaks `type.str.1` and every `str` method's guarantees.

## Decision

Option A. `bytes` is immutable; `b[i]` is a `u8` and traps out of range;
`b[a..b]` slices; `for` iterates `u8`; `+` and `concat` join. It is `Eq`,
`Ord` (lexicographic by byte) and `Hash`. Its text form is the literal form,
so `print(b)` shows `b"..."` and the bytes are never silently read as text.
Assigning an element is a compile-time error with a help that names the
rebuild. Property tests can take `bytes` parameters (a generator and a
shrinker, `test.prop.2`).

## Consequences

- The interpreter stores `Value::Bytes(Rc<Vec<u8>>)`; the bytecode tier
  indexes and iterates it and delegates the rest; threads copy it as
  `Plain::Bytes`.
- The AIL checker knows `bytes` as a type name, a trait-constructor key and
  a generatable property type (ADR 0377); the AIL lexer and parser read the
  literal (ADR 0376).
- A user `impl Trait for bytes` is possible, like for `str`.

## Rejected alternatives

- `List[u8]` for binary data (option B): rejected because it costs 16 times the memory in the interpreter, has no literal, no `find` or `starts_with`, and nothing separates text from data.
- A mutable `bytes` buffer (option C): rejected because AIL values are independent (ADRs 0060-0063), so a buffer would need `mut` rules of its own. Read, slice and join need no mutation, and `List[u8]` converts when it does.
- Letting `str` hold invalid UTF-8 (option D): rejected because it breaks `type.str.1` and every `str` method's guarantees.
