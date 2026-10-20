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

## Latest measurement: #183 + #190 + #197 merged locally

Scratch merge of `origin/lane/120-native-comp` (8a73cc7, which contains
the #183 work at 395e6df) with `claude/project-thread-u99dsn` (#190, enums,
match and computed constants) and `lane/173-native-lists` (#197, list
methods). Only doc files conflicted; the source merged cleanly.

`ail build compiler/main.ail` exits 1 with **1054** problems.

Lane 120 reported 1,067 on the batch head plus #196, and #190 alone raised
the #183 total from 1,079 to 1,158 by exposing bodies that used to stop at
their signature. This figure comes from a different base, so the numbers
are not directly comparable.

| Group | Problems | Owner (PR) |
|---|---|---|
| `str` `==` and `!=` | 281 | lane 172, #179 (strings) |
| String interpolation as a value, `char`, `str` methods | 196 | lane 172, #179 (and #184) |
| Optional types (`str?`, `Val?`, `int?`, ...) in parameters, results and fields | 200 | lane 175, #177 |
| `if x := e`, `none`, `else` | 155 | lane 175, #177 |
| Map and Set types and operations | 54 | lane 173, #180 |
| Indexing a list or map | 30 | #181 (lists), #180 (maps) |
| `List[T]` types and generics | 54 | lane 120 |
| List method calls (`extend`, `contains`, `remove_at`, `sort`, ...) | 19 | #197 (lane 173) |
| Enum types, `match`, and values of functions that must end with one | 16 | #190 (lane 179) |
| Comprehensions | 4 | #191 (lane 120) |
| Other forms: calls, generics, cascades | 45 | lane 120; non-local names go to lane 174 |

Ownership follows the split lane 120 recorded: strings to #179, enums and
`match` to #190, optionals to #177, list methods to #197, and `List[T]` and
generics to lane 120.

## What is left

About 915 of the 1054 problems depend on #177, #179 and #180, which are not
merged yet. Of the rest, about 100 are lane 120's (`List[T]`, generics and
calls), and the remainder is #190, #197 and #191, which this merge already
includes. Rerun the survey as #177, #179 and #180 land; cascades can move
the groups.

## Notes

- Earlier, `syntax/parse.ail:91` reported "internal error: invalid MIR". Lane
  120 says this was a follow-on of the `str ==` problem. It is no longer
  reported on the structs branch.
- The qualified constant `codegen.SLOT_BASE` was a cross-module artifact on
  the base. It is no longer a problem in the whole-compiler build.

## Next steps

1. Rerun the survey as #177, #179 and #180 land on the batch head, and
   update the tables.
2. Lane 120 keeps `List[T]` and generics.
3. When `ail build compiler/main.ail` succeeds, diff the native `mir` and
   `build` output against the interpreted compiler's on every perfbench
   program, and report the result here.
