# 0088. Programs start at `fn main()`; `print` is in the prelude and ends the line

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 12 — Modules (program structure); 21 — Standard library
- **Spec rules:** [mod.main.1–3](../spec/12-modules.md)
- **Evidence:** `tools/tokbench/corpus/r4/` `p1_print` 10.8, `p2_println` 11.2, `p3_io_print` 12.2

## Deciding criterion

**Token usage:** a prelude `print` is 13% cheaper than `io.print`. `print`
and `println` nearly tie, so **AI understanding** decides: one printing
function whose common case (a full line) needs no suffix. Writing without a
newline belongs to the `io` module.

## Decision

1. Execution starts at a function `main` with no parameters, declared as
   `fn main()` or, when it can fail, `fn main() !` (an inferred error set,
   ADR 0046) or `fn main() ! E`.
2. If `main` returns an error, the program prints the error report (error,
   context, return trace, per ADR 0050) to standard error and exits with
   status 1. A trap in `main` exits as defined by `err.trap.3`. Otherwise
   the exit status is 0.
3. `print(value)` writes the value's text form plus a newline to standard
   output. It's in the prelude, one of the few prelude functions (ADR 0084).

```
fn main()
    print("Hello, world!")
```

## Consequences

- Command-line arguments and explicit exit codes come through standard-library
  APIs (to be designed), not `main`'s signature.
- "Text form" requires a standard formatting trait. Its name is set with the
  standard library.
