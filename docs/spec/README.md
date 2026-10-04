# AIL Language Specification

> **Status:** early draft. Only the chapters marked *Drafted* below contain
> normative rules. Everything else is waiting on design decisions
> (see [status.md](../status.md)).

## Conventions

### Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in RFC 2119 / RFC 8174 when they appear in bold.

Text inside a block that begins with **Note:** or **Rationale:** is
non-normative.

### Rule IDs

Every normative rule has a stable ID, written as a heading or a bold prefix:

```
<chapter>.<section>.<n>
```

- `chapter` is a short lowercase key that is fixed per chapter (e.g. `plat`,
  `lex`, `type`, `mem`).
- `section` is a short lowercase key within the chapter (e.g. `ptr`, `isa`).
- `n` is a positive integer, assigned in order within the section.

Rules for rule IDs:

1. Once published, a rule ID is **never reused or renumbered**.
2. A removed rule stays in place with the marker `[removed in <ADR>]`.
3. A changed rule keeps its ID. The change is recorded in the ADR that
   justifies it.
4. Every rule cites the ADR(s) that justify it.
5. Compiler diagnostics and the AI semantic API will cite rule IDs.

### Examples and conformance tests

Examples in fenced blocks tagged `ail` are intended to become **conformance
tests** once a compiler exists (ADR 0002). An example that is expected to fail
compilation will be tagged with the expected diagnostic. That syntax will be
defined when the diagnostic format is designed.

## Chapters

| # | Chapter | Key | State |
|---|---|---|---|
| 01 | [Platforms and targets](01-platforms.md) | `plat` | Drafted (partial) |
| 02 | [Lexical structure](02-lexical.md) | `lex` | Drafted (partial) |
| 03 | [Grammar](03-grammar.md) | `gram` | Drafted (partial) |
| 04 | [Types](04-types.md) | `type` | Drafted (partial) |
| 05 | [Expressions and statements](05-expressions.md) | `expr` | Drafted (partial) |
| 06 | Memory model and allocation | `mem` | Not started |
| 07 | Ownership and lifetimes | `own` | Not started |
| 08 | [Errors](08-errors.md) | `err` | Drafted (partial) |
| 09 | Generics | `gen` | Not started |
| 10 | Traits / interfaces | `trait` | Not started |
| 11 | [Concurrency](11-concurrency.md) | `conc` | Drafted (partial) |
| 12 | Modules and packages | `mod` | Not started |
| 13 | Unsafe | `unsafe` | Not started |
| 14 | FFI and C interoperability | `ffi` | Not started |
| 15 | Compile-time execution and metaprogramming | `ct` | Not started |
| 16 | Reflection | `refl` | Not started |

Chapters and keys are provisional until the designer approves the spec outline
(see [open-questions.md](../open-questions.md)).
