# 1092. Rust, Go and TypeScript habits get an AIL message and a fix

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076); the designer approved every AI-first review item for 0.1.0 on 2026-11-12.
- **Topic:** Diagnostics for other languages' constructs ([gram.foreign.1](../spec/03-grammar.md), [lex.str.8](../spec/02-lexical.md))
- **Spec rules:** gram.foreign.1, lex.str.8
- **Amends:** [ADR 0262](0262-report-foreign-syntax-once-with-help.md) (the habit tables covered Python and JavaScript method names)

## Context

A probe of 62 Rust, Go and TypeScript habits found generic parse errors or "unknown name ... declare it first"
for most, a wrong fix for `&T`, and `"${n}"` compiling to the text `$1`. An agent that writes AIL from
those languages' habits needs the AIL spelling in the message and a fix when the repair is certain.

## Decision

- Each habit in the table under `gram.foreign.1` reports its AIL spelling; a fix is attached where the
  source alone fixes the repair (`Ok(x)`, `let mut`, `if let Some(x)`, `vec!`, `println!`, `&T`,
  `Result[T, E]`, `n as float`, `for cond`, `for i, x := range`, `for (const x of xs)`, `===`, `??`, `loop`).
  Habits whose repair needs more than the source shows (`go f()`, `Err(e)`, `append`, `.unwrap_or(d)`,
  `a, b = b, a`, `ch <- x`, `fn (p P) name()`) get help text only.
- `${` in a string literal is an error (`lex.str.8`) with the fix `{`; the literal text is `$\{`.
- "Not supported by the bootstrap yet" messages say what to write instead; the rule id `bootstrap.unsupported` stays.
- `while let` gets help text here; ADR 1088 gives it the `while x := e` form.

## Options considered

- **Fix every habit (rejected):** a fix that leaves the program not checking (a Go receiver's body still names `p`) is worse than a clear message.
- **Accept `${n}` as interpolation (rejected):** two spellings of one thing; the review's evidence is silent wrong output.
- **Per-language tables in a data file (rejected for 0.1):** the checks need token positions, so they live in the parser and checker.
