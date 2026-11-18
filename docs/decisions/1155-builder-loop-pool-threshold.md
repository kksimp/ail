# 1155. A loop that builds a list of objects needs 256 operations for the pool

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 to keep closing the speed gaps to C without syntax changes
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.14](../spec/11-concurrency.md) (amended)
- **Amends:** [ADR 0938](0938-auto-parallel-push-builder-loops.md) (builder loops on the pool, conc.par.16)

## Context

A loop the compiler chose that pushes one element per iteration builds its
list on the pool when an iteration costs at least 16 operations (ADR 0938).
The `aos` micro, with its structs kept as objects, ran in 109 ms with the
build in order and 197 ms with it on the pool (Mac, 8 threads, a million
elements): the loss is in the loops that read the list afterwards, because
objects made by eight threads no longer lie in the list's order.

Measured on the Mac (main 6d5556f8, two million elements, best of 7, wall ms
in order against on the pool, the threshold forced each way by a
measurement-only patch). "Work" is the count of conc.par.14.

| element | work | in order | pool | pool CPU |
|---|---:|---:|---:|---:|
| `float` | 12-16 | 4.6 | 4.5 | 2.7x |
| `float` | 20-24 | 6.2 | 4.7 | 2.0x |
| `float` | 128-192 | 44.8 | 12.2 | 1.5x |
| struct of floats, in place | 40-48 | 16.0 | 15.4 | 3.5x |
| struct of floats, in place | 256-512 | 29.4 | 16.7 | 2.3x |
| struct with a text | 12-16 | 24.9 | 31.6 | 6.8x |
| struct with a text | 128-192 | 29.2 | 31.9 | 5.4x |
| struct with a text | 256-512 | 38.6 | 32.5 | 4.5x |
| struct with a text | 512-1024 | 66.4 | 36.6 | 2.9x |
| struct with a new text | 12-16 | 66.4 | 68.6 | 6.2x |
| `List[float]` of two | 8-12 | 53.6 | 45.1 | 2.5x |
| the same, read back ten times | 8-12 | 79.0 | 102.1 | 2.5x |
| `str` | under 8 | 43.8 | 36.5 | 5.3x |
| the same, read back ten times | under 8 | 61.9 | 66.2 | 4.4x |

Numbers gain from the threshold already in place. Structs gain nothing below
about 256 operations, in place or not, at three to seven times the CPU and
30 to 90 MB more memory; lists and texts gain 15 to 20% on the build alone
and lose it as soon as the list is read.

## Options considered

### Option A — a second threshold for elements that are not numbers (chosen)

A builder loop the compiler chose runs in order below 16 operations an
iteration, and below 256 when an element of a built list is not an `int`,
`float`, `bool`, `char`, byte or sized number.

- Pro: follows the measurement; one constant and a test of the element type.
- Pro: a loop whose element is made by a callee with a loop or recursion
  counts 4096 and still goes to the pool.
- Con: two numbers to keep true as the allocator changes.

### Option B — raise the one threshold to 256

- Con: number builders of 16 to 255 operations lose a 1.3x to 3.7x gain.

### Option C — never build lists of objects on the pool

- Con: loses the 1.8x to 3.7x gain measured from 512 operations up.

### Option D — count an allocation as negative work

- Con: the count would need to know what allocates; the element type says
  the same with no new bookkeeping.

## Decision

Option A: `BUILD_MIN` 16 and `BUILD_OBJECT_MIN` 256 in
`compiler/mir/lower_build.ail`. A written `par for` is not affected. An
element type not yet known when the loop is judged counts as a number.

## Rationale

Speed is the criterion, and the result and traps are the same either way
(conc.par.1). The pool must not make a program slower while using several
times the CPU; where it only breaks even it should not run.

## Consequences

- `aos` with its structs as objects: 197 ms to 109 ms on the Mac.
- A builder of structs, lists or texts with 16 to 255 operations an
  iteration now runs in order; write `par for` to put one on the pool.
- The thresholds are from one eight-thread ARM64 machine. They are not yet
  measured on x86-64; the same job runs there (`AIL_OPT_OFF` is not involved,
  the constants are).

## Rejected alternatives

- **B:** gives up measured gains for number builders.
- **C:** gives up measured gains for heavy object builders.
- **D:** more machinery for the same decision.
