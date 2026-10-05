# 0125. `if x := e` handles success or absence as a statement; `else err` binds the error

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 8 — Error handling; 2 — Core syntax (expressions)
- **Spec rules:** [expr.if.2](../spec/05-expressions.md), [expr.if.5–6](../spec/05-expressions.md), [err.use.2](../spec/08-errors.md)
- **Partly resolves:** OQ-029 (handling success or failure as a statement)
- **Extends:** [ADR 0049](0049-else-for-fallback-and-early-exit.md) (`else err`)
- **Evidence:** `tools/tokbench/corpus/r6/`; [design/program-measurements-r1.md](../design/program-measurements-r1.md), round 2

## Context

`x := e else ...` must supply a value or leave the enclosing block. Code that
should do one thing on success, another on failure, and then continue needed
a helper function. Both AI agents writing a config parser hit this, and one
first attempt ended `main` early by mistake.

## Options considered

Tokens are the mean of five tokenizers:
- "result": print a parsed config or the error, then continue;
- "optional": greet a user who may not exist, then continue.

| Option | Result | Optional |
|---|---|---|
| Today: a helper function with `else` and `return` | 57.2 | 49.2 |
| `match` with `ok(x)` / `err(e)` (or `some(x)` / `none`) arms | 47.4 | 38.8 |
| **`if x := e` … `else err` (chosen)** | **44.4** (−22%) | **37.2** (−24%) |

## Deciding criterion

**Token usage** decides: the `if` form is the smallest, 4–6% below `match`,
and the case is common. **AI understanding** agrees:
- the form is Swift's and Rust's `if let`, which AIs know well;
- `else err` is the binding form AIL already has (ADR 0049);
- no new keywords or patterns (`ok`, `err`, `some`) are needed.

## Decision

1. A condition of `if` or `elif` may be a binding `name := e`, where `e` has
   type `T?` or `T ! E`. If `e` is present or succeeded, `name` is bound to
   its value, immutably and only in that branch, and the branch runs.
   Otherwise the next `elif` or `else` is tried.
2. When an `if` has exactly one condition, and it binds a `T ! E`, its `else`
   may be written `else err` to bind the error value in the `else` block.
3. An `if` whose condition binds a `T ! E` must have an `else`, so a failure
   is never ignored silently. An optional's absence may be ignored.

```ail
if cfg := parse(lines)
    print("ok: {cfg.width}x{cfg.height}")
else err
    print("error: {err}")
```

## Rejected alternatives

- **`match` with success and error arms:** larger, and it needs new pattern
  words. It may still come with matching on specific errors (OQ-029).
- **`else` on an expression statement** (`save() else err ...`): discards the
  success value, so it doesn't solve the case.
- **Allowing a binding `if` on a result without `else`:** silently ignores
  failures, which ADR 0122 rules out.
