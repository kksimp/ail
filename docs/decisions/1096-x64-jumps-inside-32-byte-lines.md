# 1096. Native x86-64: keep jumps inside 32-byte lines

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.jump.1](../spec/23-native.md)
- **Builds on:** [ADR 0385](0385-x86-64-code-generation.md) (the x86-64 encoder), [ADR 0940](0940-inliner-cost-model.md) (whose sixth round found this), [ADR 1063](1063-function-wide-value-ranges.md) (whose vm and lzr result this explains)

## Context

Wide inlining (ADR 0940) measured lzr 4% slower on ail-linux although, in lzr,
it copied nothing: it dropped one dead function, which moved `find` 635
bytes. Both builds ran the same instructions (15,549,829 thousand, equal
branches and branch misses); the moved one took 4.4% more cycles.

The x86-64 generator placed code as it fell: 6-byte conditional jumps, no
alignment of functions or loops, and a static Linux executable's code
starting 16 bytes past a 32-byte line. ail-linux is an Intel processor of
2017. Intel processors of 2015 to 2019 (Skylake to Cascade Lake), with the
microcode that fixes their "jump conditional code" erratum, do not keep a
32-byte line of code in the decoded cache when a jump, call or return, or a
comparison fused with its conditional jump, crosses the line's end or ends
on it; the line is decoded again on every pass.

Moving all code by 0 to 56 bytes, with nothing else changed (ail-linux, best
of 5, main 55ba9d85):

| moved by | 0 | 8 | 16 | 24 | 32 | 40 | 48 | 56 |
|---|---|---|---|---|---|---|---|---|
| lzr, ms | 3358 | 3584 | 3490 | 3296 | 3519 | 3575 | 3547 | 3255 |
| vm, ms | 430 | 463 | 478 | 459 | 453 | 468 | 486 | 465 |

So any change to any function moved the times of the functions after it by
up to 10% (lzr) and 13% (vm). The slowdowns of ADR 1063 on ail-linux (vm 8%,
lzr 4%, "the same instruction counts") and of ADR 0940's fifth round are
this. In lzr 979 of 4976 jumps crossed or ended on a line, 17 of 68 in
`find`.

## Options considered

### Option A — every jump inside a line (chosen)
- **Pros:** removes the cause; times stop depending on placement; no
  analysis needed; the same rule for hot and cold code.
- **Cons:** padding instructions run (lzr 4.5% more instructions, vm 11%);
  executables 6% larger.

### Option B — only jumps of blocks inside loops
- **Pros:** less padding (executables 5% larger).
- **Cons:** recursive and straight-line hot code is not covered: wordfreq 19
  ms against 16, binarytrees 19 against 17, lzr 3103 against 3071; vm alone
  was faster (331 against 346).

### Option C — align functions, or loop heads, only
- **Pros:** the usual advice; simple.
- **Cons:** does not remove the cause. Functions on 32-byte lines alone: lzr
  3457 and 3515 ms, vm 484 and 478, worse than main's lucky placement.

### Option D — leave it
- **Pros:** nothing to build; gcc and clang leave it too unless asked
  (`-mbranches-within-32B-boundaries`).
- **Cons:** up to 13% of chance in every measurement, and in every user's
  program on these processors.

## Decision

In x86-64 code, no jump, call or return, and no comparison together with the
conditional jump right after it, crosses a 32-byte line or ends on one: the
assembler (`Asm.keep_in_window`, `compiler/x64/asm.ail`) moves such an
instruction to the next line behind long `nop`s. Each function, and each
hand-assembled kernel, starts on a line; the ELF writer loads x86-64 code at
a multiple of 64 (the PE writer already loads it at a page). The stubs that
report a trap are left as they fall. `AIL_OPT_OFF=jumpline` turns it off.
ARM64 is unchanged: its instructions are four bytes and never cross a line.

## Rationale

Measured on ail-linux (x86-64, best of 5, interleaved; `main` is 55ba9d85):

| program | main | jumps in lines | change |
|---|---|---|---|
| lzr | 3369 ms | 3071 | -9% |
| vm | 430 | 346 | -20% |
| fannkuch | 27 | 22 | -19% |
| binarytrees | 21 | 18 | -14% |
| collatz | 26 | 22 | -15% |
| wordfreq | 20 | 16 | -20% |
| regex_scan | 1608 | 1603 | 0 |
| raytrace | 1688 | 1693 | 0 |
| matmul 1400 | 743 | 742 | 0 |

lzr's cycles fell from 9352 to 8426 million while its instructions rose from
15.55 to 16.24 thousand million: the padding is cheap beside a line that is
decoded again each pass. Moved by 32 bytes the times were the same (lzr 3056,
vm 347), so they no longer depend on placement. After the change 77 of 4975
jumps in lzr cross or end on a line, all of them in trap stubs, the
hand-assembled kernels (brought under the rule after this count), jump-table
data and the start-up code.

On the Mac (ARM64) no generated code changes; the numbers there are those of
main.

By the decision order (ADR 1077) nothing above speed is touched: no source,
diagnostic or result changes. Speed gains 9 to 20% on integer loops for 6% of
size, which the designer has ruled is not a concern (2026-11-12).

Processors without the erratum (AMD, Intel from 2019's Ice Lake on) run the
padding for nothing; a `nop` there costs a fraction of a cycle and no
measured result depends on it yet. That is the price of one build for every
x86-64-v3 machine (ADR 0005).

## Consequences

- x86-64 speed comparisons on ail-linux stop carrying placement chance; a
  change is measured by what it does.
- Executables grow about 6%; the compiler's self-build is to be read from
  the PR's table.
- Every x86-64 byte position changes once: goldens that hold addresses or
  sizes are regenerated.
- Follow-ups, each to be measured: 2-byte jumps for near targets (fewer
  crossings and less padding), padding before a block instead of inside it,
  and reading the processor's kind to decide at build time (rejected below
  for now).

## Rejected alternatives

- **Loops only** (B): slower on four of six programs it changes, see above.
- **Function or loop alignment only** (C): leaves the crossings in place;
  measured no better than chance.
- **Leave it** (D): keeps up to 13% of chance in every x86-64 number.
- **Decide by the building machine's processor**: the executable would
  differ by where it was built, against reproducible builds, and a program
  built on one machine runs on another.
- **Padding with prefixes on earlier instructions** (as gas can): no `nop`
  runs, but every encoder needs a padded form; left until the simpler
  padding is shown to cost something.
