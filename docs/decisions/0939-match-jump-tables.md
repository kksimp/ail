# 0939. Native: `match` through a jump table, with the dispatch repeated in each arm

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.match.2](../spec/23-native.md)
- **Builds on:** [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (pass 5, the decision tree), [ADR 0611](0611-overflow-checks-by-range-guards.md) (loop rotation), [ADR 0848](0848-idle-values-leave-registers-in-loops.md) (parked values)

## Context

ADR 0890 pass 5 lowers a `match` of five or more int or variant arms to a
binary search (native.match.1). That gave the vm drag-race program -10% on the
Mac and -1.2% on x86-64, leaving it 1.59x the C twin. The tree still costs
about log2(arms) compares and branches per dispatch, and the one dispatch
point shares its branch history between every arm. C interpreters write
`goto *labels[op]` at the end of each arm so the indirect branch predicts the
next op from the one that ran (threaded code); gcc -O2 emits one jump table
for a `switch` and never duplicates it.

## Decision

1. **A `switch` terminator in MIR.** `switch cond, [t0, ..., tn-1, other]`
   goes to `t[cond]` when `0 <= cond < n` (compared unsigned), else to `other`.
   Edges may pass arguments; `split_edges` gives each such edge a block, as for
   `branch`, so the code generators see none.
2. **Lowering** (`lower_matchtree.ail`): where native.match.1 applies and the
   sorted values span at most 1024 with at least a third of the span taken,
   the index is the value minus the smallest (wrapping, so one unsigned
   compare takes every value outside the span) and gaps go to the `_` arm, or
   to an `unreachable` block for an exhaustive `match`. Sparser matches keep
   the tree. `AIL_OPT_OFF=switch`.
3. **Both backends.** x86-64: `cmp`, `jae other`, `lea` of the table,
   `movsxd` of the entry, `add`, `jmp reg`; the table of 32-bit distances from
   its start follows the jump. ARM64: `cmp`, `b.hs other`, `adr`, `add` of the
   entry address, `ldrsw`, `add`, `br`; each entry is the distance from itself.
4. **Tail duplication at code generation.** A jump to a block chain of at
   most four blocks and 32 instructions, with no call and no value the
   allocator parks across it (ADR 0848), that ends in a `switch` repeats the
   chain's instructions, the moves between its blocks and the `switch` with
   its own table, instead of jumping to it. A rotated loop test (ADR 0611)
   whose body is such a chain repeats it too. A copy needs a table of at most
   64 entries, and one function gets at most 4 KB of copies (about 8 bytes an
   instruction, 4 an entry), so a compiler full of large matches does not grow
   by a table per arm. Every value keeps its one
   location, so the copy does exactly what the original does.
   `AIL_OPT_OFF=taildup`.

No trap, trace frame or order of effects changes: the arm chosen is the one
the in-order test chooses (native.match.1).

## Measured

First run (ail-linux, 870d0571 against main d40e5092, no caps on the copies):
vm 3863 → 2958 ms (gcc -O2 1671, Python 122593); with `AIL_OPT_OFF=switch`
4033, with `AIL_OPT_OFF=taildup` 3328; the rest flat. The stage-2 compiler
went 8.7 → 18.3 MB with this and ADR 0940 together; the caps above came after. To be filled from the runner (ADR 0890 protocol): vm, lzr and the perfbench
set against gcc -O2 and Python on ail-linux and the Mac, and the stage-2
compiler's size and build time.

## Rejected alternatives

- **A table of branch instructions** (`br` into a run of `b` on ARM64): one
  fewer load, but a second taken branch on every dispatch.
- **Tail duplication in MIR** (copying the dispatch blocks into each arm):
  needs SSA repair for every value the dispatch defines; repeating the
  allocated code in the generator needs none.
- **Tables for sparse matches** (a third of the span or less): table size
  grows with the span while the tree's cost grows with log2 of the arms.
- **Keeping only the tree** (ADR 0890's earlier choice): measured gain too
  small, as that ADR anticipated.
