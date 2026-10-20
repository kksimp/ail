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
6. Implementation limits (a construct the current implementation doesn't
   support yet) use the code `bootstrap.unsupported`. That is not a rule ID. The
   diagnostic names the spec rule of the unsupported construct.

### Examples and conformance tests

Every fenced block tagged `ail` in a chapter is a **conformance test**
(ADR 0002). `bootstrap/tests/spec.rs` runs `ail check --json` on each one
during `./ail selftest` and reports failures by chapter and line. The
same rules apply to `docs/quick-reference.md` and `docs/language-card.md`, where a block
without `fn main` is a fragment without a tag (to save tokens). The fence
line says what to expect:

| Fence | Expectation |
|---|---|
| ```` ```ail ```` | A whole program: must check without errors (warnings are fine). |
| ```` ```ail fragment ```` | A snippet. Column-0 declarations (`struct`, `enum`, `error`, `unique`, `fn`, `trait`, `impl`, `const`, `use`, `type`, `@attribute`, `pub`, and `#` comments before them) stay top level; every other line goes into a generated `fn main()`. Must then check without errors. |
| ```` ```ail error=RULE ```` | Must report `RULE` among its errors. Handled as `fragment` when it has no `fn main`. |
| ```` ```ail skip=bootstrap ```` | Uses a feature the bootstrap doesn't implement yet. Counted, not run. Remove the tag when the feature lands. |
| ```` ```ail skip=context ```` | An illustration that uses declarations the chapter doesn't show. Counted, not run. Prefer making the example self-contained. |

An unknown tag fails the suite. Blocks in other languages (`text`, `ebnf`)
aren't run.

## Chapters

| # | Chapter | Key | State |
|---|---|---|---|
| 01 | [Platforms and targets](01-platforms.md) | `plat` | Drafted (partial) |
| 02 | [Lexical structure](02-lexical.md) | `lex` | Drafted (partial) |
| 03 | [Grammar](03-grammar.md) | `gram` | Drafted (partial) |
| 04 | [Types](04-types.md) | `type` | Drafted (partial) |
| 05 | [Expressions and statements](05-expressions.md) | `expr` | Drafted (partial) |
| 06 | Memory model and allocation | `mem` | Not started |
| 07 | [Ownership and value semantics](07-ownership.md) | `own` | Drafted (partial) |
| 08 | [Errors](08-errors.md) | `err` | Drafted (partial) |
| 09 | [Generics](09-generics.md) | `gen` | Drafted (partial) |
| 10 | [Methods and traits](10-traits.md) | `trait` | Drafted (partial) |
| 11 | [Concurrency](11-concurrency.md) | `conc` | Drafted (partial) |
| 12 | [Modules and programs](12-modules.md) | `mod` | Drafted (partial) |
| 13 | [Unsafe](13-unsafe.md) | `unsafe` | Drafted (partial) |
| 14 | [FFI and C interoperability](14-ffi.md) | `ffi` | Drafted (partial) |
| 18 | [Effects and capabilities](18-effects.md) | `eff` | Drafted (partial) |
| 19 | [Testing](19-testing.md) | `test` | Drafted (partial) |
| 20 | [Contracts](20-contracts.md) | `contract` | Drafted (partial) |
| 15 | [Compile-time execution and metaprogramming](15-compile-time.md) | `ct` | Drafted (partial) |
| 16 | [Reflection](16-reflection.md) | `refl` | Drafted (skeleton) |
| 21 | [Runtime traces and trap snapshots](21-trace.md) | `trace` | Drafted (partial) |
| 22 | [Tools: api, explain, fix, budgets, diff, edit, replay, snapshots](22-tools.md) | `api`, `explain`, `fix`, `budget` | Drafted (partial) |
| 22 | [Tools: api, explain, fix, budgets, diff, edit, replay, snapshots](22-tools.md) | `api`, `explain`, `fix`, `snap` | Drafted (partial) |
| 22 | [Tools: api, explain, fix, budgets, diff, edit, replay, snapshots](22-tools.md) | `api`, `explain`, `fix`, `budget`, `diff` | Drafted (partial) |
| 17 | [Standard modules](17-std.md) | `std` | Drafted (partial) |
| std | [Standard library: json](std-json.md) | `json` | Drafted |
| std | [Standard library: regex](std-regex.md) | `std` | Drafted |

Chapters and keys are provisional until the designer approves the spec outline
(see [open-questions.md](../open-questions.md)).
