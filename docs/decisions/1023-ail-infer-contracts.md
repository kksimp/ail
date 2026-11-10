# 1023. `ail infer --contracts`: proposed `requires` and `ensures` from test calls

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076), implementing the item the designer chose in [ADR 0895](0895-ail-infer.md)
- **Topic:** 22 — Tooling
- **Spec rules:** infer.cand.1-6, infer.out.1-4
- **Builds on:** [ADR 0885](0885-contracts-checked-by-fuzzing.md), [ADR 0895](0895-ail-infer.md), [ADR 0397](0397-ail-edit-structured-edits-by-id.md)
- **Amends:** [ADR 0895](0895-ail-infer.md) (the command takes the function and a file, and observes by wrapping the function; the instrumentation layer of ADR 0871 is not used)

## Context

Agents skip contracts because writing them is work, yet a `requires` or `ensures` is the
one local spec a later edit can read (ADR 0885 then fuzzes it). The tests already show
which values each function sees.

## Options considered

### Option A: a stand-in function in a copy of the file (chosen)
`ail infer --contracts FN FILE` writes a copy of FILE beside it with FN renamed and a
function of the same signature in its place. The stand-in calls the original, appends one
row of numbers per call to a scratch file, and returns the result. `ail test` runs the copy.
- **Pros:** no change to the lowering or the runtime, so nothing for a program that does not
  use it; works the day it lands on both architectures; the test runner is the one users run.
- **Cons:** only a top-level, non-generic function that cannot fail, with no `mut`, `move` or
  default parameter; only the tests of FILE are run.

### Option B: return values in the `AIL_INSTRUMENT` call trace (ADR 0871)
- **Pros:** any function, any test file.
- **Cons:** the trace records no return values and no lists, and its 1000-event default cuts a
  long test run. Adding both means new runtime slots and lowering in a lane that must land on two
  code generators at once.

### Option C: random inputs, as `ail test --contracts` does
- **Pros:** needs no tests.
- **Cons:** random inputs have no domain: every range they show is the generator's own, so
  they would propose nothing true or something false.

## Decision

`ail infer --contracts FN FILE [--apply] [--min N] [--cases N] [--json]` (tools/infer/infer.ail).
It records the `int` arguments, the length of each `str` and `List` argument, and the same for
the result, once per call (infer.cand.1). Property tests count: their inputs are the fuzzing
(`--cases N` sets how many). It keeps a condition only when every call satisfies it and at
least 20 were seen (infer.cand.2-3). The templates are a range (`> 0`, `>= 0`, `!= 0`), and
the order of two recorded values (`<`, `<=`, `>`, `>=`), for the parameters (`requires`) and
for the result (`ensures`). A value that never changes is not proposed: tests pass one
constant by habit. At most 3 are shown, parameters first (infer.cand.4-5), and one already
under the signature is not repeated (infer.cand.6).

The answer is a patch in `ail edit` hunk form on standard output, so `ail edit FILE < patch`
applies it, and one line per condition with its call count on standard error (infer.out.1).
The patch is dry-run through `ail edit` first and nothing is proposed if it does not check
(infer.out.2). `--apply` hands it to `ail edit`, which formats and checks (infer.out.3).
Nothing is written without `--apply`, and the output says the conditions were observed, not
proved (infer.out.4).

## Rationale

Token usage ranks first (ADR 0076): one call returns a three-line patch, where an agent would
read the tests and the function and then write the same lines. Wrapping the function reuses
`ail test`, `ail edit` and the property generators, so the tool is one file and needs no
compiler change (the speed lane's code generators stay untouched). The cap of 3 and the
minimum of 20 calls keep the proposals worth a reviewer's glance: ADR 0895's rule that a guess
must never reach the optimizer unreviewed holds, because `--apply` is a choice made per
function.

## Consequences

The check the contracts get next is ADR 0885's: `ail test --contracts FILE` fuzzes them and
shrinks a counterexample. Methods, generic and fallible functions, and the other tests of a
package come in a later version, with the tracing of return values (Option B). The
optimizer may use only contracts that were accepted into the source (ADR 0895).

## Rejected alternatives

- **Writing contracts without `--apply`:** guesses become promises the optimizer trusts (ADR 0895).
- **Proposing more than 3 conditions:** a reviewer reads each; past three they are noise.
- **Static inference:** weak for the numeric ranges that tests show at once (ADR 0895).
