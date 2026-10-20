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

## What blocks it, by the open PR that covers it

Problems are grouped by message and by the type named in the source line.
The coverage column is the open or announced lane that is expected to
remove the group. These are estimates, not a measurement of a merge.

| Group | Problems | Covered by |
|---|---|---|
| Optional types (`str?`, `Val?`, `int?`, ...) in parameters, results and fields | 164 | #177 (optionals and results) |
| `if x := e` bindings, `none`, `else` | 80 | #177 |
| `==` and `!=` on `str` | 140 | #179 (strings: comparison) |
| String interpolation outside `print`, `char`, `starts_with`, `split`, `code` | 81 | #179, and #184 for Unicode tables |
| Enum types (`Ty`, `ast.Expr`, `Tok`, `Def`, ...) and enum declarations | 200 | lane 179 (enums and match, announced, not open) |
| `match`, and functions that must end with a value | 60 | lane 179 |
| Non-literal top-level constants | 19 | lane 179 (announced) |
| Map and Set types and operations | 36 | #180 (maps and sets) |
| `List[T]` types (`List[char]`, `List[Ty]`, ...) | 85 | partly #177 and lane 179; `List[char]` needs #179 |
| List methods on `List[...]` and on unknown types (`.extend`, `.contains`, `.remove_at`, `.sorted`, ...) | 56 | no open PR (lists) |
| Indexing a list or map | 29 | #181 (lane 176) for lists; maps need #180 |
| Comprehensions | 63 | no open PR |
| Names that are not locals, parameters or constants, used as values | 23 | no open PR (function values, lane 174 or 130) |
| Other struct, generic and call forms | 19 | lane 120 |
| Calls with more than 6 integer or 8 float parameters | 9 | lane 120 (calling convention) |
| Calls of functions not in the subset | 7 | cross-check after the lanes above |
| Type mismatch cascades | 12 | follow the types |

## What is left after #177, #179, #180 and lane 179

Estimate: the groups covered by #177, #179, #180 and lane 179 come to about
770 of the 1079 problems. About **310** would remain, mostly these:

- Lists: about 170 (`List[T]` types not covered, list methods, indexing).
- Comprehensions: 63.
- Non-local names used as values: 23.
- Lane 120 leftovers (struct, generic and call forms, the 6-parameter
  calling convention): about 35.
- Cascades and calls, which should fall away with the types: about 20.

This is an estimate by message group, not a merge. The true count needs a
merge of #177, #179, #180 and lane 179's branch on top of #183, and a
rerun of the survey. Cascades mean the count can move by more than the
groups show.

## Notes

- Earlier, `syntax/parse.ail:91` reported "internal error: invalid MIR". Lane
  120 says this was a follow-on of the `str ==` problem. It is no longer
  reported on the structs branch.
- The qualified constant `codegen.SLOT_BASE` was a cross-module artifact on
  the base. It is no longer a problem in the whole-compiler build.

## Next steps

1. Rerun the survey after #177, #179, #180, #183 and lane 179 are merged
   into the batch branch, and update this table.
2. Lists: name an owner for `List[T]` types, list methods and comprehensions.
   These are about 220 problems and have no open PR.
3. When `ail build compiler/main.ail` succeeds, diff the native `mir` and
   `build` output against the interpreted compiler's on every perfbench
   program, and report the result here.
