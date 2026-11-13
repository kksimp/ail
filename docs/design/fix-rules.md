# `ail fix`: classes with no single correct rewrite

Non-normative design note, 2026-10-21 (lane 245, fix round three).

`ail fix` resolves 151 of the 250 mistakes in `tests/mistakes/`
(60.4%), with no wrong fixes. This round went through every unresolved class
of three or more programs. None has a single correct rewrite, so none was
implemented. This note records why, so nobody retries them without new
evidence. A class is listed here only when its programs need a choice the
program does not make.

## Measurement

Driver: each program is copied, `ail fix` runs on the copy, and the result is
checked against the program's `.fixed` file (resolved), against a clean
re-check (unresolved), or against neither (wrong).

| State | Programs |
|---|---:|
| Resolved | 151 |
| Unresolved, error count unchanged | 94 |
| Unresolved, error changed (a different error shows) | 5 |
| Wrong | 0 |

Rate: 60.4% (151 of 250). Unchanged from round two, so the status row moves
only in wording.

## Classes with three or more programs

Each row says what the rewrite would have to choose. "Ambiguous" means the
choice changes what the program does.

| First rule | Programs | Why no single rewrite |
|---|---:|---|
| `expr.name.1` | 13 | The name is unknown in each case: `input`, `int`, `null`, `thread`, `undefined_name`, `unwrap`, `popcount`, `append`, `json`, `message`, `group`, `push` on a map. The help offers `io.read_line()` (returns `str?`), `s.to_int()` (returns `int?`), `e.to_str()` or interpolation, and `v := x else default`. Each one needs a type, a default or a choice of meaning. `null_literal` has a second error once the name is fixed. |
| `gram.foreign.1` | 9 | Foreign syntax: ternary, `switch`, `throw`, `try`, lambdas, tuple patterns, tuple shapes, a `numpy` import. |
| `expr.call.1` | 6 | Five are arity errors: the arguments to add or drop are the author's choice. `date_timedelta_python` gets `name: value` from the rewrite, but `timedelta` is not in scope, so it stays unresolved (`expr.name.1`). |
| `conc.task.5` | 5 | Task handles: where to put `wait`, whether to add to a group, and whether a dropped handle should be waited. Each changes when the work finishes. |
| `trait.op.3` | 4 | Bytes and text: `b[i]` as a character, a comparison of a byte with a character, `bytes + text`, and a CRC plus text. Each needs a conversion, and the choice changes the result type. |
| `err.use.1` | 4 | `main` is not fallible. A fix either makes `main` return `!` and adds `?`, or gives the `else` a body. The two change the program's exit behaviour, so the choice is the author's. |
| `expr.block.2` | 4 | A return with no value or a wrong type: change the return type, or add a value. Which one the author meant is not in the program. |
| `conc.par.5` | 4 | `par for` with `break`, `return`, prints or writes through a `ref`. Restructuring the loop to `out[i]` or to a sequential loop changes the program. |
| `expr.call.2` | 3 | Channel and argument type errors: a conversion, a different argument, or a different function. The conversion has more than one correct form. |

## Classes with fewer than three programs

Each singleton or pair was looked at. None had a rewrite that was clearly the
only correct one, so they are left for the same reason as the classes above.
Examples: `type.struct.7` (a `strftime` or `subtract` method that does not
exist), `mod.use.2` (names that come from a module not imported), `std.use.2`
(a missing hash or regex function with a different signature), `contract.ens.3`
(`old` in a contract, where the fix is to remove the contract or move the
value), `type.opt.4` (none used as a value), `std.tensor.1` (tensor construction
from strings or nested lists).

## What would change the decision

- A diagnostic that names the right rewrite. Some help text already suggests
  one (for example `expr.name.1` suggests `io.read_line()`); a rewrite is only
  safe when the help is the only correct reading of the error.
- A new fixable rule with one correct answer. Candidates have to be checked
  against `compiler/explain/fixable.txt` and the `ail fix` edit model
  (`77592931:bootstrap/src/diag.rs`, AfterExpr anchors, ADR 0695).
- A new mistakes corpus. The 250 programs were written to fail in known ways;
  a rewrite that fits them may not fit real code.

## Where the fixes are

Round two (ADR 0695, lane 216) added `gram.foreign.1` for `~name`. Round three
found no further single-answer rewrite. Lane 238 added the trailing-comment
and nested-call cases for `err.use.1` (PR #297); the baseline above already
includes it.
