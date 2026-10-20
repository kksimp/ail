# Self-compile status

Lane 181 of the native self-compile push. Owner of the survey:
[tools/self-compile/run.sh](../../tools/self-compile/run.sh) (ADR
[0585](../decisions/0585-self-compile-survey-tool.md)). Ranking and owners:
[ADR 0586](../decisions/0586-self-compile-blocker-ranking.md). Goal: the native
compiler builds itself (ADR [0022](../decisions/0022-staged-self-hosting.md)),
then the stdlib, module loading and the `ail` command leave Rust.

## Snapshot: whole-compiler build (module loading landed)

Measured on `origin/lane/120-native-modules` at 395e6df (PR #183, stacked on
#178). `ail build compiler/main.ail` now loads the modules and lowers the
whole compiler in one run. It exits 1 with **1079** `native.subset.1`
problems in 30 files. The survey script builds the whole compiler, not each
module alone (see ADR 0585).

Earlier snapshots, for comparison: the base `lane/120-native-opt` at 512cc79
had 937 problems when each module was lowered alone, and
`lane/120-native-structs` at 5680f91 had 953 in the same mode.

Top files: `mir/lower.ail` 166, `syntax/parse.ail` 117,
`check/checker.ail` 111, `syntax/lex.ail` 85, `syntax/recover.ail` 68,
`ast/ast.ail` 51, `x64/gen.ail` 48, `a64/gen.ail` 48.

To reproduce, from the repository root:

    tools/self-compile/run.sh /path/to/outdir

## Owners of what blocks it

Lane 120 measured 1,067 problems on the batch head plus #196 (its
re-survey, 2026-10-20 20:26 UTC). Comprehensions are done in #191. The 29
indexing reports should clear once enums and optionals land. Owners follow
the split the coordinator recorded:

| Group | Problems (1079 on #183) | Owner (PR) |
|---|---|---|
| `str` equality `==` and `!=` | 140 | lane 172, #179 (strings) |
| String interpolation as a value, `char`, `str` methods | 81 | lane 172, #179 (and #184 for Unicode tables) |
| Enum types, enum declarations, `match`, values of functions that must end with one | 251 | lane 179, #190 (enums and match) |
| Optional types, `if x := e`, `none`, `else` | 244 | lane 175, #177 (optionals and results) |
| List methods (`.extend`, `.contains`, `.remove_at`, `.sorted`, ...) | 56 | lane 173, list methods (about 60 in lane 120's count) |
| `List[T]` types and generics | 85 | lane 120 |
| Non-local names and enum variants used as values | 23 | lane 174 (function values, about 25 in lane 120's count) |
| Comprehensions | 63 | done in #191 |
| Map and Set types and operations | 36 | lane 173 (maps and sets), #180 |
| Indexing a list or map | 29 | #181 (lane 176) for lists; maps with #180 |
| Non-literal top-level constants | 19 | lane 179 (announced) |
| Other struct, generic and call forms | 19 | lane 120 |
| Calls with more than 6 integer or 8 float parameters | 9 | lane 120 (calling convention) |
| Calls of functions not in the subset, and type mismatch cascades | 19 | follow the types |

Note that the list-method row and the maps row both name lane 173. The
coordinator's split gives lane 173 list methods; #180 is its maps and sets
PR.

## What is left

Once #177, #179, #190 and #191 land on the batch head, the groups above
cover all but the lane 120 rows (lists and generics, about 90 problems, plus
struct and call forms and the calling convention, about 30) and the
cascades. The true count needs the rerun, because cascades can move the
numbers.

## Notes

- Earlier, `syntax/parse.ail:91` reported "internal error: invalid MIR". Lane
  120 says this was a follow-on of the `str ==` problem. It is no longer
  reported on the structs branch.
- The qualified constant `codegen.SLOT_BASE` was a cross-module artifact on
  the base. It is no longer a problem in the whole-compiler build.

## Next steps

1. Rerun the survey as #177, #179, #190 and #191 land on the batch head,
   and update the owners table.
2. Lane 120 keeps `List[T]` and generics (about 90 problems).
3. When `ail build compiler/main.ail` succeeds, diff the native `mir` and
   `build` output against the interpreted compiler's on every perfbench
   program, and report the result here.
