# 0424. Streaming child processes: `proc.start`, with the child's streams as channels

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture; 15 — Concurrency
- **Spec rules:** [std.proc.4–5](../spec/17-std.md#processes-stdproc)
- **Extends:** [ADR 0160](0160-process-runs.md), [ADR 0282](0282-std-proc-input-timeout-which.md)

## Context

`proc.run` waits for the child and returns its whole output. An agent also
needs to start a long-running process (a build, a server, `tail -f`), read
its output as it arrives, feed its input, and do other work meanwhile. The
brief named `proc.spawn`, a `Child` with `stdin`, `stdout`/`stderr` as streams
readable from tasks, `wait`, `kill`, `pid` and a timeout.

## Options considered

### Option A — `Child` carries three `Channel[str]` fields (chosen)
`proc.start(cmd, args, timeout_ms?) Child ! ProcError`, with `c.stdin`,
`c.out`, `c.err` as channels. A reader thread per output stream queues lines;
the scheduler moves them into the channel (ADR 0426).
- **Pros:** no new stream type, no new methods: `recv`, `for line in c.out`,
  `close` and `task.select` (ADR 0427) already exist and are specified,
  checked, traced and cancelled (`conc.cancel.*`). One mechanism to learn.
  Measured: the reading loop `for line in c.out` costs the same tokens as
  one over any channel.
- **Cons:** only lines, no raw bytes yet (lane 123 adds `bytes`; a byte
  stream can follow the same shape).

### Option B — a new `Stream` type with `read_line()`, `read(n)`, `write()`
- **Pros:** room for bytes and partial reads.
- **Cons:** a second way to wait for data that `select` and `for` would each
  need to learn; a new handle kind through the checker, both interpreter
  tiers and the AIL checker. More tokens at every use.

### Option C — callbacks (`on_line(fn)`)
- **Cons:** AIL has no function values yet (language gap, lane 130).

## Decision

Option A. The function is named `start`, not `spawn`: `spawn` is a keyword
(`lex.kw.1`) and `proc.spawn(...)` doesn't lex. `Child.pid` is an `int`
field. The timeout is the same optional last parameter as `run`'s (`std.proc.2`).
Lines are split at `\n`, a `\r` before it dropped; text is lossy UTF-8 as in
`run`. Capability: `proc` (and `io`), as `run` (`eff.infer.4`).

## Rationale

Criterion 1, tokens: no new vocabulary beyond `start`, `wait`, `kill`.
Criterion 2, AI understanding: a child is "three channels and an id", which
a reader already knows.

## Consequences

- `ail run --deny proc` and `--deterministic` reject `start`, `wait`, `kill`.
- `--trace` shows a child as `proc.Child` without its process id, which
  changes between runs (`trace.run.1`).
- Windows: `Command` pipes work the same; the tests use `sh`, which lane 153
  maps to `cmd` stubs.

## Rejected alternatives

- A new `Stream` type with `read_line()`, `read(n)` and `write()` (option B): rejected because a second way to wait for data would need `select` and `for` to learn it, and every use costs more tokens.
- Callbacks such as `on_line(fn)` (option C): rejected because AIL had no function values yet (lane 130).
- The name `spawn`: rejected because `spawn` is a keyword (`lex.kw.1`), so `proc.spawn(...)` does not lex; the function is `start`.
