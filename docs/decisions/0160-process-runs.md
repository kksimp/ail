# 0160. `proc.run` runs a program directly, captures its output, and fails only if it can't start

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.proc.1](../spec/17-std.md)

## Context

Agent tools run compilers, test runners and version control. Plan §4 asks for
`run(cmd, args) Output ! ProcError` with no shell.

## Options considered

### Option A — a nonzero exit is a successful `run` (chosen)
- **Pros:** the tool sees the status and the captured `err` text, which is
  usually what it needs (`git` exits 1 for "no changes"); one failure type
  for one cause.
- **Cons:** a caller that forgets to check `status` treats failure as success.

### Option B — a nonzero exit is a `ProcError`
- **Cons:** the output of a failed run must ride in the error; many programs
  use nonzero statuses for answers, not failures (`grep`, `diff`, `test`).

## Deciding criterion

Tokens **tie**. **AI understanding** decides: `Output.status` is explicit
and printed in `Output`'s text form, and the error type means exactly
"couldn't start".

## Decision

`proc.run` runs `cmd` without a shell, inherits the working directory and
environment, gives it an empty standard input, captures standard output and
standard error as text, and waits. A signal ends with status 128 plus its
number.

## Consequences

- Streaming output, standard input, timeouts, a working directory and
  environment overrides are later additions (named arguments, ADR 0058).
- No shell means no quoting bugs and no injection through arguments.

## Rejected alternatives

- **Nonzero status as an error:** loses the output, and `grep`'s "no match"
  isn't a failure.
- **Inheriting standard input:** a child that waits for input hangs an
  unattended agent.
