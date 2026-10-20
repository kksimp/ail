# 0438. `std.regex` is a linear-time Pike VM, without backreferences or lookaround

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.regex.4](../spec/std-regex.md), [std.regex.21](../spec/std-regex.md)
- **Criterion that decided:** AI understanding (no pattern has a surprising cost, so an agent can write one without a worst-case analysis); runtime speed was a tie at this size.

## Context

AIL has no regular expressions, and agents reach for them constantly in
log, text and config work. A backtracking engine (Perl, Python, Java, the
JavaScript family) takes exponential time on patterns such as `(a+)+b`; a
program that runs a pattern over untrusted text is then a denial of service.
Python's `re` didn't finish in two minutes on the vector `(a+)+b` against
thirty `a`s while building the test vectors for this ADR.

## Options considered

### Option A — a backtracking engine with backreferences and lookaround
- **Pros:** the familiar feature set; fast on easy patterns.
- **Cons:** exponential cases; an agent can't see them from the pattern.

### Option B — a Thompson NFA run as a Pike VM, with captures (chosen)
- **Pros:** time is pattern size times text length for every pattern;
  leftmost-first priorities, so results equal a backtracker's for the
  syntax it supports; submatches in the same pass; small (about 600 lines).
- **Cons:** no backreferences or lookaround (they can't be matched in that
  time); slower than a DFA on huge inputs.

### Option C — a lazy DFA with a Pike fallback for captures
- **Pros:** faster on large inputs.
- **Cons:** much more code; speed is not what the first slice needs.
  Left as a later optimisation behind the same API.

### Option D — a crate
- **Cons:** ADR 0009 prefers custom tooling and the bootstrap carries no
  dependency that isn't justified.

## Decision

Option B. `std.regex` is a Pike VM in the bootstrap (`bootstrap/src/regex`):
a parser to a tree, a compiler to a small instruction list (`Char`, `Class`,
`Split`, `Jmp`, `Save`, `Assert`, `Match`), and a simulation that steps one
thread per program counter across the text. Empty transitions are followed
with an explicit stack, so long chains can't overflow the native stack. A
counted repetition is unrolled, so a program is capped at 50 000
instructions (`too_large`). Backreferences and lookaround are errors of kind
`unsupported` that point to `std.regex.21`.

An iteration of a loop that adds nothing is dropped (a thread can reach a
program counter once per position). This differs from Perl for
`(a*)*` and `(|a)+` submatches, never for whether or where the whole
pattern matches; the difference is in `std.regex.4` and in the vectors.

## Source and licence

The design is the textbook Thompson/Pike construction described in public
papers and RE2's documentation. No code was copied from any project. The
test vectors were written for this repository; expectations were produced
with an independent engine (Python's `re`, as an oracle) and reviewed by
hand, and no third-party test file is included, so the repository's Apache
2.0 with Runtime Exception licence is not affected.

## Consequences

- Native backend and the AIL port later need a matcher in AIL; this VM is
  small enough to port, and the vectors are the contract.
- No `(?=`, `\1`: programs that need them test the captured text in AIL.
