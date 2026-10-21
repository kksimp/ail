# 0735. Module-level list constants get a program global

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** [native.subset.1](../spec/23-native.md), mod.const.1, own.val.1
- **Builds on:** [ADR 0604](0604-native-computed-constants.md), [ADR 0384](0384-mir-shape-and-native-subset.md)

## Context

ADR 0384 inlines a constant that is a constant expression at each use, and
ADR 0604 gives every other constant a program global slot. `is_const_expr`
counted a list literal of constants as a constant expression, so a list
constant was rebuilt (`list_new` plus a `list_push` per element) at every use.

`tools/perfbench/programs/wordfreq.ail` reads `WORDS := [...]` twice per
iteration (`WORDS[i]`, `WORDS.len()`). The native binary rebuilt the 12-string
list twice for each of 1,000,000 words and ran slower than CPython:

| wordfreq, 1M words, Linux x86-64 | best of 21 | median |
|---|---|---|
| native AIL, before | 0.277 s | 0.306 s |
| native AIL, after | 0.037 s | 0.042 s |
| gcc -O2 twin | 0.009 s | 0.010 s |
| CPython 3 | 0.236 s | 0.256 s |

Callgrind at 100,000 words: 718 instructions per word before, 278 after. Before,
`main`, `rt.alloc`, `rt.list_alloc` and `rt.list_new` took about 70% of them.
After, the top three are `rt.map_find_h`, `rt.map_find` and `rt.map_hash`
(25%, 24%, 23%): the map probing itself. The C twin counts into a 12-slot
array and hashes nothing, so the remaining 4x is the cost of a real map.

## Decision

A list literal is not a constant expression for inlining. A list constant gets
a program global slot, computed once by `init.consts` (ADR 0604). Scalars,
strings, other constants and arithmetic on them stay inlined.

Reading the global and indexing it, taking its length or iterating it does not
copy; binding it to a `mut` variable, pushing it or returning it copies it
(own.val.1, ADR 0604 point 4). `bootstrap/tests/native/run/const_list.ail` covers
both.

## Rejected alternatives

- **Hoisting the rebuilt list out of the loop (loop-invariant motion):** the
  sequence is allocation and pushes, which a MIR pass can't prove free of
  effects without a purity model, and it would not help straight-line code
  that reads the constant in a function called from a loop.
- **Caching in the optimiser per function:** one rebuild per call instead of
  one per use; the global makes it one per program.

## Consequences

- One line in `compiler/mir/lower.ail` (`is_const_expr`), and `all_const` goes.
- The compiler's own list tables become globals too; the native tests (x86-64,
  and arm64 under qemu) and the selftest pass.
- Order is as ADR 0604 says: declaration order, with constants named in another's
  expression first. A constant reaching a later one through a function body is the
  case ADR 0604 already lists as differing from the interpreter.
