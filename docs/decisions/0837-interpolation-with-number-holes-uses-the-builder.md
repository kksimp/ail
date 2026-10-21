# 0837. Interpolation with a number, `char` or `bool` hole is written into the builder and allocated once

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; performance
- **Spec rules:** type.str.3, lex.str.5
- **Supersedes in part:** [ADR 0768](0768-native-string-scans-by-words.md) (its rejection of a builder object for interpolation)
- **Builds on:** [ADR 0582](0582-number-text-builder-and-format-specs.md)

## Context

ADR 0768 joined an interpolation's pieces with one sized allocation (`rt.str_catN`) and rejected a builder (`rt.sb_*`) because the lengths are known when lowering and a builder needs a growth check per piece. That holds when every piece is a string. It does not hold for a number: `"key{n}"` made the number's text a string of its own (`rt.fmt_int`: write into the builder, count characters, allocate, copy) and then `rt.str_cat3` copied that string again. Two allocations and two copies for six bytes, 567 instructions per use in a loop, with `map_ops` at 40% string building.

## Decision

An interpolation with at least one `int`, `char` or `bool` hole (and no format spec on it) is lowered as:

1. every hole's value is evaluated first, in order (a hole may itself interpolate, and the builder is global, so none may run between `rt.sb_begin` and `rt.sb_str`);
2. `rt.sb_begin`;
3. one call per piece: `rt.sb_text` (a string), `rt.put_int`, `rt.sb_add_char` or `rt.put_bool`;
4. `rt.sb_str`, which allocates the result once.

Interpolations whose pieces are all strings (including holes that are already text, formatted holes and display calls) keep `rt.str_catN` as in ADR 0768. Two builder costs are fixed with it: `sb_bytes` copies with `copy_bytes` after one room check instead of one call per byte, and `sb_str` counts characters eight bytes at a time (`utf8_chars`). The result is byte for byte the text `ail run` produces.

## Measured (callgrind, native x86-64, same base)

| Program | before | after |
|---|---:|---:|
| `"key{n}"` in a loop | 113.5 M | 85.5 M (-25%, including a separate `alloc_raw` fast path worth about 6 M) |
| `map_ops` | 224.4 M | 182.0 M (-19%) |
| `lines_split` | 1,540 M | 1,377 M (-11%) |
| `fasta` | 68.7 M | 68.1 M |
| `wordfreq`, `print_many` | unchanged | unchanged |

## Rejected alternatives and criterion

Criterion: speed (4) with no token or understanding cost.

- **Keep `rt.str_catN` and add integer-aware variants** (`str_cat_int`, ...): one entry per combination of piece kinds, and the arity limit of six pieces applies to each.
- **A temporary string object in a scratch slot per piece:** a hole that interpolates itself would overwrite the slot before the join; evaluating the holes first avoids that for the builder too, but the scratch objects need a root and a global slot.
- **The builder for string-only interpolations:** copies every piece twice; `lines_split` (ADR 0768) is faster with the sized join.

## Consequences

- `compiler/mir/lower_str.ail` and `compiler/rt/native/fmt.ail` change; `lower.ail` is not touched, so this is not in the lower.ail queue.
- `bootstrap/tests/native/run/str_interp_scalars.ail` compares native and `ail run` on nested holes, wide characters, format specs, builder growth and long texts.
- Every other user of the builder (error and `fs` / `proc` error texts) gets the faster copy and character count.
