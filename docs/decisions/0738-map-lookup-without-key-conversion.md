# 0738. Text and integer map keys are looked up without building a `Key`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library (the interpreter's values)
- **Spec rules:** none (no language behaviour changes)
- **Builds on:** [ADR 0175](0175-interpreter-fast-paths.md) (the fast hasher)

## Context

`Map` and `Set` are one insertion-ordered table (`interp/builtins.rs`). Every
lookup built a `Key` from the value: a clone of the `Rc<str>` (a count
increment and decrement) and the derived `Hash` and `Eq` of a four-variant
enum. On `wordfreq` that conversion, its drop and the comparison were about
7% of the instructions.

## Decision

`Table` keeps two extra hash indexes beside the general one: `Rc<str>` to
slot for text keys and `i64` to slot for integer keys (including the sized
integer types, whose bits were already the key). `get`, `insert` and `remove`
pick the index from the value; every other key type uses the general `Key`
index. Insertion order, gap compaction and `len` are unchanged. A lookup of
a text key hashes the string's bytes directly and compares them with the
stored text.

## Measured (callgrind, Linux x86-64, instructions)

| program | before | after |
|---|---|---|
| wordfreq, 200,000 words | 565.7 M | 530.9 M (-6.2%) |
| map_ops (`tools/perfbench/agent`) | 842.0 M | 768.4 M (-8.7%) |

`./ail selftest` passes.

## Rejected alternatives

- **A method-name-to-id table.** A profile of `wordfreq` puts all of
  `method_run`, string compares included, at 4.5% of the instructions; the
  rest of the cost is the instruction loop, which belongs to lane 190.
- **A faster hash function.** Hashing is about 4% of `wordfreq`; a
  word-at-a-time hasher did not reduce the instruction count.
- **A custom open-addressing table.** More code for a gain this change
  already mostly gets.
