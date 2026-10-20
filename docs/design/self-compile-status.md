# Self-compile status

Lane 181 of the native self-compile push. Owner of the survey:
[tools/self-compile/run.sh](../../tools/self-compile/run.sh) (ADR
[0585](../decisions/0585-self-compile-survey-tool.md)). Ranking and owners:
[ADR 0586](../decisions/0586-self-compile-blocker-ranking.md). Goal: the native
compiler builds itself (ADR [0022](../decisions/0022-staged-self-hosting.md)),
then the stdlib, module loading and the `ail` command leave Rust.

## Snapshot

Two snapshots. The first is the base of this PR; the second is the
structs branch, which is what the next survey should be compared with.

- **Base** `lane/120-native-opt` at 512cc79 (PR #164): 937 problems in 30
  modules. Top: methods and generic functions (454), parameter types (184),
  struct declarations (88), result types (70).
- **Structs branch** `lane/120-native-structs` at 5680f91: 953 problems in
  30 modules. Methods are now accepted, so their count moved into
  "methods of X" (71) and "X outside a method" (1). Field types (112) and
  struct-typed parameters and results now dominate.
- `./ail build compiler/main.ail` exits 1 on both. Module loading is not in
  yet, so the other 29 modules are never lowered by it; the survey lowers
  each module alone instead.
- Cross-module artifacts: calls of functions in other modules (54 on base,
  69 on structs) and qualified constants like `codegen.SLOT_BASE` (6). They
  are real blockers only after module loading, so they are listed apart.
- Interpreted-vs-native comparison of `mir` and `build` output on the
  perfbench programs: not run, because the build does not succeed.

To reproduce, from the repository root:

    tools/self-compile/run.sh /path/to/outdir

## Top blocking constructs (structs branch)

Counts are from the survey on 5680f91. Owners are from the lane briefs and
lane 120's note of 2026-10-20.

| # | Construct | Problems | Owner |
|---|---|---|---|
| 1 | Parameters whose type is not in the subset (mostly `List[T]` and struct types) | 313 | 120 (structs), 171 and 176 (lists) |
| 2 | Struct fields whose type is not in the subset | 112 | 120, 171 (heap) |
| 3 | Results whose type is not in the subset | 83 | 120, 171 |
| 4 | Operators and methods on types not in the subset: `==` on `str`, `.extend()` on `List`, `.code()` on `str` | 77 | 172 (strings), 176 (lists), 173 (maps) |
| 5 | Methods of structs | 71 | 120 (lands on the structs branch; check its remaining cases) |
| 6 | Calls of functions in other modules (cross-module) | 69 | 120 (multi-file modules) |
| 7 | Locals-only restriction on assignment targets | 59 | 120 |
| 8 | Functions that end in a `match` on a string or enum: "must end with its result" | 37 | 120; 179 (enums, match) |
| 9 | Enum declarations | 19 | 179 (enums and match) |
| 10 | Top-level constants that are not literals | 17 | 179 (announced by lane 120) |

Smaller, still open: string interpolation outside `print` (13, 172),
indexing a list or map (14, 176 and 173), more than 6 integer or 8 float
parameters (8, 120 calling convention), comprehensions (6), `else` on a
binding (2, 175), maps (1, 173), and an error struct (1, 175).

## Problems per file (structs branch, top ten)

| File | Problems |
|---|---|
| mir/lower.ail | 175 |
| x64/gen.ail | 102 |
| a64/gen.ail | 89 |
| syntax/lex.ail | 87 |
| check/checker.ail | 86 |
| ast/ast.ail | 51 |
| obj/macho.ail | 39 |
| syntax/recover.ail | 36 |
| check/traits.ail | 27 |
| obj/elf.ail | 25 |

The full list is in the survey's `summary.md`.

## Bugs and notes

- `syntax/parse.ail:91` reported "internal error: invalid MIR: not_yet:
  b2 passes a str to b1's bool" on the base. Lane 120 says this was a
  follow-on from the `==` on `str` problem (lane 172). Since the
  lowering stops verifying MIR after a problem, only the `native.subset.1`
  line is printed now, on lane/120-native-structs.
- Misattributed message: `x64/gen.ail` and `a64/gen.ail` say "must end
  with its result" for `is_reg` and `is_slot`. The cause is the qualified
  constant `codegen.SLOT_BASE` on the next line, which is cross-module.

## Next steps

1. Lane 120 merges structs; rerun the survey on the batch branch, and
   compare with the two snapshots above.
2. Lane 179 lands enums, match and non-literal constants; rerun.
3. Module loading lands; rerun, and drop the cross-module rows.
4. When `ail build compiler/main.ail` succeeds, diff the native `mir` and
   `build` output against the interpreted compiler's on every perfbench
   program, and report the result here.
