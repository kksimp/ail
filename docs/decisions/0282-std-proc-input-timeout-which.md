# 0282. `proc.run` takes input and a time limit; `proc.which` and `env.vars`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard modules; AI-first operation of the computer
- **Spec rules:** std.proc.2–3, std.env.5, std.err.1

## Context

`proc.run(cmd, args)` (ADR 0160) reads an empty standard input and waits
forever. Agents pipe text through tools (`sort`, `jq`, formatters) and must
not hang on a tool that never exits. They also check whether a tool is
installed before relying on it, and inspect the environment.

## Options considered

### Option A — optional `input` and `timeout_ms` on `run`, `TimedOut` as a `ProcError` kind (chosen)
- **Pros:** one function to learn; the common call stays two arguments;
  `proc.run("sort", [], text)` is 3 tokens more than without input; a time
  limit is a `ProcError` the caller already handles.
- **Cons:** needs trailing optional parameters on a standard function
  (ADR 0284).

### Option B — a `proc.Command` builder struct
- **Pros:** room for `cwd`, `env` and more later.
- **Cons:** ~12 more tokens per call (`proc.Command(cmd: ..., args: ...,
  input: ...)` plus `.run()`); two ways to run a process.

### Option C — a separate `TimedOut` error type
- **Cons:** callers then handle a union of two errors for one call;
  `kind == "timed_out"` fits std.err.1's scheme.

## Decision

`run(cmd, args, input str?, timeout_ms int?)`, the last two optional from
the end; a process past its limit is killed and `run` fails with
`ProcError(kind: "timed_out")`. `which(name) str?` finds a program as `run`
would. `env.vars() Map[str, str]` returns the UTF-8 environment, sorted.
`std.env.args()` already existed (std.env.1).

## Rationale

Criterion 1 then 2: the shortest call for the common case, and failures in
the error type the caller already matches on. Output is read on threads, so
a process filling one pipe can't deadlock the other.

## Consequences

- Setting the child's working directory or environment is not possible yet;
  a later change can add optional parameters or a builder.
- `which` has the `proc` effect: it answers "what would `run` start".

## Rejected alternatives

- A builder struct: more tokens, two ways to do one thing (Option B).
- A separate `TimedOut` type: a union to handle for one call (Option C).
- Keeping the partial output on timeout: makes `run`'s failure carry a
  value; the brief's agents retry or report, they don't parse half-output.
