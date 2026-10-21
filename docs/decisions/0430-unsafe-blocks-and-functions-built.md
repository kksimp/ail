# 0430. `unsafe` blocks and `unsafe fn`, with a manifest opt-in, are built

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Unsafe semantics
- **Spec rules:** [unsafe.block.1–5](../spec/13-unsafe.md), [mod.pkg.2](../spec/12-modules.md), [eff.pure.1](../spec/18-effects.md)
- **Builds:** [ADR 0102](0102-unsafe-blocks.md), [ADR 0106](0106-unsafe-operations.md)

## Context

ADR 0102 decided that unsafe operations need an `unsafe` block, and that a
package may contain one only if its manifest opts in. It left open the
manifest syntax and the single-file opt-in (its Consequences). Nothing was
built. The FFI lane (ADRs 0430–0437) needs the block first, because every C
call and every raw-pointer operation is an unsafe operation (ADR 0104).

## Decision

1. **Syntax.** `unsafe` followed by an indented block is a statement.
   `unsafe fn name(...)` declares a function callable only inside an
   `unsafe` block. Both forms were already reserved words.
2. **Value.** An `unsafe` block is checked like any other block: its last
   expression is its value when the block is the last statement of a body,
   and `return` leaves the function from inside it (`unsafe.block.5`).
3. **What it checks.** Calling an `unsafe fn` outside an `unsafe` block is
   `unsafe.block.2`. Slices 2 and 3 add `ffi.call.1` (C calls) and the
   raw-pointer operations on the same mechanism. The body of an `unsafe fn`
   is not an implicit `unsafe` block: each unsafe operation in it needs its
   own, so the unsafe lines stay exactly the marked lines.
4. **Effect.** An `unsafe` block gives its function the effect `unsafe`
   (`eff.infer.4`), which propagates to callers, so `@pure` rejects a
   function that contains one (ADR 0164). A safe wrapper is therefore not
   `@pure`, which is deliberate: the compiler can't prove what the C code does.
5. **Opt-in (`unsafe.block.3`).** A line of `ail.pkg` takes `unsafe allow`:
   `package app unsafe allow` for the package's own code, and
   `dep geo path ../geo unsafe allow` for a dependency. A package that
   contains an `unsafe` block or `unsafe fn` without it gets an error at the
   block. A program with no `ail.pkg` has no packages to restrict, so a
   single file or a plain directory may use `unsafe`; running it is the
   opt-in. This answers the open "file-level attribute" question of ADR 0102
   without a new construct.
6. **Listing (`unsafe.block.4`).** Every `unsafe` block and `unsafe fn` is
   in the syntax tree (`ail api --ast`: `Unsafe` statements, `unsafe_fn`).
7. **Bytecode tier.** A function containing an `unsafe` block runs on the
   tree walker (like `with` and `defer`): FFI calls are not hot, and the
   tier is not widened for them.

## Options considered

### Option A — `unsafe` statement block, manifest word `unsafe allow` (chosen)
- **Pros:** the exact lines are marked (ADR 0102); one new statement and one
  manifest key; tokens: one keyword and the block's indentation, which any
  block-structured form needs; reuses the `caps` grant machinery's reading
  of `ail.pkg`.
- **Cons:** a safe wrapper looks like an `unsafe`-effect function to the
  effect inference.

### Option B — `unsafe` as the effect `caps unsafe`
Reuse `caps LIST` for the opt-in.
- **Cons:** `caps` means "may reach", and effects propagate through callers,
  so every caller of a safe wrapper would need the grant, which defeats the
  purpose of a safe wrapper (ADR 0104).

### Option C — a value-less `unsafe` statement only
- **Cons:** wrappers need `return` in every one; the spec's own example
  yields a value from the block.

### Option D — `unsafe` blocks as expressions anywhere (`x := unsafe f()`)
- **Cons:** a new expression form in the parser, the checker and both front
  ends for little gain over `return` or the tail position; can be added
  later without breaking this form.

## Consequences

- The syntax tree gains `Stmt::Unsafe` and the function fields `unsafe_fn`
  and `ext` (`ext` is used from ADR 0431); `compiler/syntax/` and
  `compiler/ast/` carry them and agree with `ail api --ast`.
- `docs/decisions/README.md` row, `docs/status.md`, the glossary, the quick
  reference and the language card describe the form.
- Measurement (`tools/tokbench`): `unsafe` is one reserved word and no
  punctuation; there is no competing spelling to compare, so there is no
  before and after table.

## Rejected alternatives

- `unsafe` as the effect `caps unsafe` (option B): rejected because `caps` means "may reach" and effects propagate through callers, so every caller of a safe wrapper would need the grant, which defeats the purpose of a safe wrapper (ADR 0104).
- A value-less `unsafe` statement only (option C): rejected because wrappers would need `return` in every one, and the spec's own example yields a value from the block.
- `unsafe` blocks as expressions anywhere (`x := unsafe f()`) (option D): not chosen. It would be a new expression form in the parser, the checker and both front ends for little gain over `return` or the tail position, and it can be added later without breaking this form.
