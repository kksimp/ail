# 1087. Std papercuts: assert message, `assert_err`, `assert_near`, more format specs

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), from the designer's approval of every AI-first review item for 0.1.0 (2026-11-12)
- **Topic:** 19 — Testing, 02 — Lexical, 04 — Types
- **Spec rules:** [test.assert.1-3](../spec/19-testing.md), [lex.str.7](../spec/02-lexical.md), [type.str.11](../spec/04-types.md)
- **Amends:** [ADR 0168](0168-assert-and-assert-eq.md) (assertions), [ADR 0173](0173-format-specifications-in-interpolation.md) (rejected a `0` flag, sign, `,` and `e`)

## Context

Probes of what an agent writes unprompted showed five gaps in the std surface: `assert(c, "why")`
is an error, `assert_err` doesn't exist (and `ail fix` offered to rename it to `assert_eq`),
`{n:08}`, `{n:b}`, `{n:,}`, `{n:+}` and `{x:e}` don't compile, and `s.to_int(16)` doesn't exist.

## Decision

- `assert(cond, "message")` traps with `assertion failed: message`, evaluating the message only on failure.
- `assert_err(call)` and `assert_err(call, E.Variant)`, and `assert_near(a, b, eps)`, join the prelude (`test.assert.3`).
- Format specs become `[[fill]align][+][0][width][,][.decimals][x|b|e]`. The `0` flag pads after the sign
  and replaces the `0>8` spelling only for numbers; `0>8` stays valid as a fill. `e` writes the
  shortest digits as `d.ddde` with a plain decimal exponent.
- `FormatSpec` gets one field, `flags`, a string of the marks `+0,be`, so the AST JSON gains `"flags"`.
- `s.to_int(radix)`, `n.to_str(radix)`, `n.to_str()` (`type.str.20`); `find`, `rfind`, `count`
  and `replace` take a `char` (`type.str.6`); `m.get_or(k, d)` (`type.map.12`); `xs.reversed()`
  (`type.coll.27`); the unknown-trait help for `Add`, `Index` and friends names the two operator
  traits there are, `Eq` and `Ord` (`trait.op.2`).
- Descending order is `xs.sorted().reversed()`; no `desc:` flag (one way to write it, and
  `reversed()` is needed anyway). Package S11 (collection vocabulary) keeps this.
- Deferred to a follow-up: enum `variants()`/`from_str()`/`name()`, `ToJson` on `Map`, `List` of
  optionals and `T?` at top level, and the unused-local warning (needs use tracking in the checker).

## Rejected alternatives

- Keep `0>8` only: agents write `08` (Python, Rust, C), and the error cost a round trip every time.
- A Python-style `+e03` exponent: more tokens and no gain; AIL's own float text uses a plain exponent.
- Separate boolean fields per mark in `FormatSpec`: five fields in four files for no readability gain.
- `assert_err` as a macro-like rewrite to `match`: the lowering already has `else` machinery that keeps the error value typed.

## Follow-up (S7b)

- Field-less enums get `variants()`, `from_str(s)` and `name()` without being written (`type.enum.20`);
  the checker answers them in `compiler/check/enumref.ail`, the lowering in `compiler/mir/lower_enumref.ail`.
  Rejected: an implicit derive spliced into every enum (token splicing per enum for three one-line functions),
  and a trait the user must implement (agents write `Color.variants()` unprompted).
- An unused `:=` binding is a warning with the fix `_` (`expr.bind.14`, `compiler/check/unused.ail`), for the
  main program's own functions only, so std and package modules stay quiet. Rejected: an error (an unused
  value is a likely bug but not wrong), and `@allow` (the `_` fix is the allowance).
  A program with an error gets none of these warnings (batch 8, 2026-11-13): they buried the error under
  noise and made a first draft noisier. Rejected: warning anyway (24 error cases gained an extra line).
- `ToJson` on `Map` and `T?` moved to its own lane: it needs generic `impl`s for a second builtin owner.
