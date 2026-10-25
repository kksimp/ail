# 0584. Native coverage: status rules, construct keys and counting

- **Status:** Accepted
- **Amended by:** [ADR 0906](0906-retire-native-coverage-tracker.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1](../spec/23-native.md) (output and exit status equal the interpreter's; a trap's first line is the interpreter's), [native.subset.1](../spec/23-native.md), [native.build.2](../spec/23-native.md)
- **Builds on:** [ADR 0583](0583-native-coverage-tracker.md)

## Context

The tracker needs one status per program and a count of missing constructs that
lanes can act on. Two choices shape the numbers: which differences count as a
mismatch, and how to name a construct.

## Decision

**Status.** Each program gets exactly one status, checked in this order:

1. `not-subset`: `ail build` exits non-zero and prints `native.subset.1` problems.
2. `backend-error`: the build prints `internal` or a `trap[` line. A trap while the
   native compiler runs means a backend bug, not a user error.
3. `check-rejected`: the build prints a `error[...]` or `warning[...]` diagnostic
   from the front end.
4. `build-error`: any other non-zero build, including a timeout.
5. Otherwise the program built. It is `built-match` when its standard output,
   exit status and first trap line equal those of `ail run` on the same program,
   with no arguments and standard input from `/dev/null`. Otherwise it is
   `built-differs`, with the first differing stdout line on each side.

Stderr is compared only by its trap line, because native.build.1 lets the
interpreter's frame lines be absent.

**Construct key.** Each problem message is grouped by a key. A message of the form
`not in the native subset yet: X` is keyed by `X` with backticks removed. Any other
message is keyed by its text with each backticked name replaced by `…`, and digits
replaced by `N`. Names of user functions, types and fields therefore group together.

**Counting.** The "problems" column counts every `native.subset.1` problem once. The
"first" column counts only each program's first problem, because that is the
construct to fix to unblock the program. The percentage that matches is taken over
programs that pass `ail check`, so negative tests do not lower it.

**Negative tests.** A case with a sibling `.err` or `.rules` file is marked
`negative`. The report counts them inside `check-rejected`. The status does not
change.

**Delta.** Each run compares its statuses with the previous `results.json`, by
file, and lists the programs that changed status.

## Rejected alternatives

- **Counting only each program's first problem.** It hides how often a construct
  appears in programs that are already blocked elsewhere, and so understates the
  value of a fix.
- **Comparing full stderr.** The interpreter's trap frames may legitimately differ
  (native.build.1), so a full comparison would report false mismatches.
- **Grouping by exact message text.** Every user name would be its own row, and the
  table would not show which construct to build next.
- **Passing each case's `.args` file to both runs.** Those files hold interpreter
  flags such as `--no-contracts`, which `ail build` does not accept. Such programs
  run with no arguments, so a `built-differs` row for one of them may be the
  harness and not the compiler. Its name is listed so a reviewer can check it.
