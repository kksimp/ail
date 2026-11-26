# 1185. A bare `check`, `run`, `test` or `build` means the package here

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); command behaviour and diagnostics, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [tool.help.1](../spec/22-tools.md), [test.run.1](../spec/19-testing.md), [native.build.3](../spec/23-native.md), [gram.foreign.1](../spec/03-grammar.md), type.enum.2

## Context

A capture of what the four everyday commands print (2026-11-26, a project made by `ail new`) showed answers that
cost an agent a turn each and teach nothing:

- `ail check`, `ail run` and `ail test` with no argument said "unknown command check"; `ail build` printed an
  internal usage line. Other toolchains accept the bare word inside a project, so agents type it.
- `ail test .` ended in an `IsDir` error with a stack of the test runner; `ail build .` said "can't write .".
- `ail test FILE NAME` printed the runner's whole option list.
- `assert a == b` was reported as "expected end of line after the statement", with help about long statements.
- `Num(int)` in an enum was reported as a missing type at the `)`, with no help.

## Decision

- With no argument, `check`, `run`, `test` and `build` act on `.` when the current directory has an `ail.pkg`.
  Anywhere else they print `usage: ail CMD FILE|DIR` and say that there is no `ail.pkg` here.
- `ail test DIR` tests `DIR/main.ail`, or the one `.ail` file of `DIR`; any other directory is refused with its
  files listed. The tests of imported modules run as for a file.
- `ail build DIR` with no `-o` writes `DIR/NAME`, NAME being the directory's name, and checks first as for a file.
- `ail test FILE NAME` says to write `--filter NAME`.
- `assert a == b` is a gram.foreign.1: `assert` is a call. The fix adds the parentheses and is certain
  (ADR 1130), since the rewrite has one meaning.
- `Num(int)` is reported at the type's name: a variant's field needs a name, such as `value int`. No fix, because
  a fix would pick the name.

## Options considered

- **A bare command acts on `.` everywhere.** Rejected: outside a package, `.` is often a directory of unrelated
  programs, and the answer is a list of `main` clashes (mod.unit.1).
- **A bare `test` runs the tests of every file below.** Rejected for now: a test run is one program
  (test.run.1), and a package's `main.ail` already reaches its modules' tests.
- **Take `ail test FILE NAME` as a filter.** Rejected: a second file is as likely as a name, and one spelling
  (`--filter`) is easier to teach than two.
- **Accept `assert a == b` as syntax.** Rejected: a second form for one meaning, against the designer's rule of
  no syntax to mimic other languages; the certain fix gives the same result with one form.
- **Accept `Num(int)` as an unnamed field.** Rejected: a language change, and construction names the fields.

## Consequences

- No new rule IDs: each behaviour extends the rule that already governed the command or the construct.
- `ail.ps1` does not have the bare-command rule or the directory forms yet (Windows parity is deferred).
- The gate suite `agent_tools` holds a case for each behaviour.
