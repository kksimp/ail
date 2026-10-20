# 0565. A native `str` method is one runtime call, and its traps carry constant text

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** native backend (lane 172, native strings and chars)
- **Spec rules:** native.subset.1, native.str.1, type.str.3, type.str.4, type.str.7, type.str.9-10, type.str.13-15

## Context

The native subset (ADR 0390) had string literals, `len`, `chars`, `join`,
`to_int() else` and interpolation inside `print`. The compiler needs the rest
of the `str` methods, interpolation as a value, and comparison, with the
interpreter's results and traps byte for byte (native.build.1). A string
method can trap (`slice`, `split`, `replace`, `repeat`, `pad_left`), and a
trap's first line and source excerpt come from the call's position (ADR 0405),
which the runtime doesn't know.

## Options considered

### Option A — one runtime call per method; a trapping call takes the position's text as two constants (chosen)
- **Pros:** No new MIR operation and no change to either code generator. The
  runtime is plain AIL, shared by both targets. The lowering builds the head
  (`PATH:LINE:COL: trap[RULE]: `) and the excerpt as string literals, so the
  trap output is the interpreter's.
- **Cons:** A trapping call passes two extra constant arguments (two address
  loads). Negligible next to the allocation these methods do.

### Option B — a new `strtrap` MIR operation with a site, like `get`
- **Pros:** Same shape as the existing checks; no extra arguments.
- **Cons:** Touches `mir.ail`, `verify.ail`, both code generators and the stub
  tables, which are lane 120's critical path.

### Option C — check in the lowering (a `bool` call, then a branch to a trap block)
- **Pros:** The trap block is explicit in MIR.
- **Cons:** Every method that can trap needs its check written twice, once in
  the runtime to compute and once in the lowering to test; blocks multiply.

## Decision

Each `str` method in the native subset lowers to one `call rt.NAME`. A method
that can trap also takes the constant head (one per rule it can raise) and
the constant excerpt of the method name's position, and the runtime prints
them with the message through `rt.str_trap`.

## Rationale

Decided by criterion 4 (speed of the compiler and of the programs) and by
risk: it adds nothing to the shared files of lane 120.

## Consequences

- Both targets share the code; the tests run on x86-64 and on aarch64 (qemu).
- Strings stay immutable and shared; every method that builds text allocates.
  The collector (lane 171) frees it.
- Methods whose result is an optional (`find`, `char_at`, `strip_prefix`,
  `strip_suffix`, `to_float`) wait for optionals in the subset (lane 175).

## Rejected alternatives

Option B costs lane 120 merge conflicts for no gain in output. Option C
duplicates every check.
