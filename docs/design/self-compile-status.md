# Self-compile status

Lane 181 of the native self-compile push. Owner of the survey:
[tools/self-compile/run.sh](../../tools/self-compile/run.sh) (ADR
[0585](../decisions/0585-self-compile-survey-tool.md)). Ranking and owners:
[ADR 0586](../decisions/0586-self-compile-blocker-ranking.md). Goal: the native
compiler builds itself (ADR [0022](../decisions/0022-staged-self-hosting.md)),
then the stdlib, module loading and the `ail` command leave Rust.

## Snapshot

- Base: `origin/lane/120-native-opt` at 512cc79 (PR #164, stacked on #158).
- `./ail build compiler/main.ail` exits 1 with 5 problems, all in `main.ail`.
  Module loading is not in yet, so the other 29 modules are never lowered.
- Survey (`tools/self-compile/run.sh`): 30 modules, all fail, **937 problems**.
  Of them, 54 are calls of functions in other modules, and 6 are qualified
  constants like `codegen.SLOT_BASE`; both are cross-module artifacts, not gaps.
- Interpreted-vs-native comparison of `mir` and `build` output on the
  perfbench programs: not run yet, because the build does not succeed.

To reproduce, from the repository root:

    tools/self-compile/run.sh /path/to/outdir

## Problems per file (top ten)

| File | Problems |
|---|---|
| syntax/parse.ail | 113 |
| mir/lower.ail | 99 |
| syntax/lex.ail | 79 |
| x64/asm.ail | 76 |
| a64/asm.ail | 60 |
| check/checker.ail | 55 |
| a64/gen.ail | 54 |
| x64/gen.ail | 51 |
| ast/ast.ail | 42 |
| obj/macho.ail | 39 |

The full list of 30 files is in the survey's `summary.md`.

## Top blocking constructs

Counts are from the survey. Owners are from the lane briefs.

| # | Construct | Problems | Owner |
|---|---|---|---|
| 1 | Methods and generic functions | 454 | 120 (structs and methods; generics by monomorphisation) |
| 2 | Parameter, result and field types that are structs | ~150 (88 struct declarations) | 120 |
| 3 | `List[T]` as parameter or result | ~116 | 171 (heap), 176 (lists) |
| 4 | Calls of functions in other modules (cross-module) | 54 | 120 (multi-file modules) |
| 5 | Enums as types, and `match` on them | 20 enum types | 120 (enums and match) |
| 6 | Functions that end in `match` on a `str`, "must end with its result" | 17 | 120 (match on string arms) |
| 7 | Optionals `str?` and `else` on bindings | ~10 | 175 |
| 8 | String operations: `==` on `str`, `.code()`, interpolation outside `print` | ~10 | 172 |
| 9 | Top-level constants that are not literals | 8 | unowned; proposed 120 |
| 10 | More than 6 integer or 8 float parameters | 4 | 120 (calling convention, native-abi.md) |

Also seen, smaller: `.extend()` on `List[int]` (4, 176), `for mut` and
`else` handlers (in 175), indexing a list or map (in 176 and 173), and
`?` with `else` forms (175).

## Bugs found by the survey

- `syntax/parse.ail:91`: "internal error: invalid MIR: not_yet: b2 passes a
  str to b1's bool". An internal error in the lowering, not a subset rule:
  the function is `not_yet(k str) bool`. Reported to lane 120.
- Misattributed messages: `x64/gen.ail:180` and `a64/gen.ail:203` say
  "must end with its result" for `is_reg`. The cause is the qualified
  constant `codegen.SLOT_BASE` on the next line, which is cross-module.

## Next steps

1. Lane 120 lands structs, methods and enums; rerun the survey after each
   merge into the batch branch.
2. Module loading lands; rerun, and drop the cross-module rows.
3. When `ail build compiler/main.ail` succeeds, diff the native `mir` and
   `build` output against the interpreted compiler's on every perfbench
   program, and report the result here.

## Repro cases and owners (lane 226, 2026-10-21, batch head fcc7f33d)

Minimal native cases for the blocked constructs live in
`bootstrap/tests/native/selfc/` (`selfc_NAME.ail`, with `selfc_NAME.out` from
`ail run`). They are not in `tests/native/run/`, so they do not gate
`cargo test --test native`; a case that fails is a blocker, not a broken test.
`tools/self-compile/selfc-cases.sh` builds each one and prints the pass count.

Current: **16 of 29 pass**. The survey is stale in one large way: the
`parameter \`f\`: its type isn't in the native subset` count (43 of 169 on this
head) comes from lowering one file alone. `Func` is declared in
`compiler/mir/mir.ail`, not in `addr.ail`, so the survey reports it as unknown.
A struct, enum or list-of-struct parameter already builds natively when the type
is in the same file (cases `struct_param`, `struct_list_param`, `enum_param`,
`struct_return`, `struct_field`, `struct_method`). Module loading would clear
those rows, not a native lane.

| construct | case | owner (queue order) | status |
|---|---|---|---|
| `==` on `str` (and `<`) | `str_eq`, `str_lt` | lane 172 (#184, then #204) | fail: `expected bool, found str` |
| string interpolation outside `print` | `str_interp` | lane 172 (#184) | fail |
| comprehension whose element type isn't known | `comprehension_str` | lane 172 (#184) | fail |
| `Map` as a parameter, field or local | `map_param`, `map_field`, `map_get`, `for_over_map` | lane 173 (#180) | fail |
| `Set` (`add`, `len`) | `set_add` | lane 173 (#180) | fail |
| `.extend()` / `.contains()` on `List[int]` | `list_extend`, `list_contains` | lane 173 (#197) | fail |
| `sorted(by: closure)` (method with a closure) | `sorted_closure` | lane 174 (#170, #207, #208, #224) | fail |
| calling `unreachable(...)` | `unreachable_call` | lane 170 (#170) | fail |
| struct / enum / list-of-struct parameter, return, field, method | `struct_*`, `enum_param`, `struct_method` | same file: pass | pass (cross-module: module loading) |
| optional return and field | `optional_param`, `optional_field` | lane 175 (#177, #203) | pass |
| `.join()`, `.len()` on `str`, nested lists, indexing a struct list, `if` yielding `str`, `enum` match bindings | `list_join_str`, `str_len`, `nested_list`, `index_struct_list`, `if_value_str`, `enum_match_binding` | — | pass |
| indexing a list of ints (control) | `index_int_list` | — | pass |

Owners follow the native queue in `/mnt/project-files/lanes/170-native-rules.md`. A row changes
status when its owner's PR lands on the batch head: rerun
`tools/self-compile/selfc-cases.sh` and `tools/self-compile/run.sh`, then update
the table. Last rerun: 16 of 29 pass on fcc7f33d.
