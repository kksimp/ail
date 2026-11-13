# 1102. The `select` statement over channels

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put the AI-first review's items in 0.1.0 (2026-11-12), this one as package S20
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.select.2-6](../spec/11-concurrency.md); amends [lex.kw.1](../spec/02-lexical.md)
- **Builds on:** [ADR 0427](0427-task-select.md) (`task.select`), [ADR 0996](0996-try-recv-and-group-fields.md) (`try_recv`, `recv_for`), ADR 1073 (generators)

## Context

`task.select(chans)` (ADR 0427) waits on a list of channels of one element
type and returns an index; the caller then calls `recv` on that channel. A
program that waits on a job channel of `Job` and a stop channel of `str`, or
that waits for room to send as well as for a value, or that needs a timeout,
had to poll with `try_recv` and `sleep_ms`, or funnel everything through one
channel of a hand-made enum. The pre-release AI-first review listed this as
item 14. Go's `select` is the habit most agents bring.

## Decision

1. **Form** (conc.select.2). `select` and an indented block of arms; each arm
   is a head, `=>`, and a block or one statement on the line, as a `match`
   arm. Heads: `v := ch.recv()`, `ch.send(x)`, `after ms`, `closed`. The
   receive and send heads are written exactly as the calls they stand for,
   so an agent reads them as what they do. `select` becomes a keyword
   (lex.kw.1); `after` and `closed` are words only at the start of an arm, so
   no name is taken from programs.
2. **Choice** (conc.select.3). The heads are evaluated once, in source order,
   then the `select` waits and runs the first arm in source order that can go
   ahead: lowest wins, as `task.select` already does, so a run is repeatable
   and an agent can predict which arm runs. The received value is a `T`, not
   a `T?`: closure has its own arm. Channels of different element types mix.
   Exactly one operation happens: the runtime takes or queues under the
   channel's lock and reports the arm.
3. **Timeout** (conc.select.4). `after ms =>` runs when `ms` milliseconds pass
   with no arm ready; `after 0` is the non-blocking form (Go's `default`).
   Under `--deterministic` it behaves as `recv_for` (ADR 0996).
4. **Closure** (conc.select.5). `closed =>` runs once every receive arm's
   channel is closed and empty; until then a closed channel's arm simply never
   runs. A `select` with a receive arm must have one, so a loop around it
   always has a written way out. Go runs a closed channel's arm again and
   again with a zero value, a classic bug; here it cannot happen.
5. **Waiting forever and cancellation** (conc.select.6). With no `after` arm,
   waiting when no other task can run is a deadlock (conc.task.7), as `recv`.
   A cancelled task never waits: when nothing is ready it runs `closed`, else
   `after`, else no arm, matching `recv` returning `none` (conc.cancel.2).
6. **Native.** `compiler/mir/lower_select.ail` builds a table (three words per
   receive or send arm, one for a trap message; heap kind 13 so the values
   stay alive) and calls `rt.sel_wait` (`compiler/rt/native/chansel.ail`),
   which tries each row in order under its channel's lock and sleeps on the
   channels' shared change counter, as `task.select` does. No runtime global
   slot is needed. A child process's output streams (std.proc.5) work in a
   receive arm and its `stdin` in a send arm.
7. `task.select` stays: it takes a list built at run time.

## Rejected alternatives

- **A `select` expression giving a value:** every arm would need a value of
  one type, and most arms `break`, `return` or send. A statement keeps the
  arms ordinary blocks; a value is one `mut` away.
- **Random choice among ready arms (Go):** fairness for long-running loops,
  but output that differs between runs and arms an agent can't predict. The
  order is written; an arm that must not starve goes first.
- **`v := ch.recv()` giving `T?` and no `closed` arm:** every arm would test
  for `none`, and forgetting the test loops for ever on a closed channel.
- **`default =>` for the non-blocking form:** `after 0` says what it does with
  the arm that already exists; the parser's message for `default` names it.
- **Reserving `after` and `closed`:** both are common names (`closed` is a
  field in programs and the std); arm-head position is unambiguous.
- **Desugaring in the parser** (as `while name := e`, ADR 1088): there is no
  existing construct that takes from one of several channels atomically, and
  diagnostics would name hidden code.
- **A runtime global slot for the select:** the per-statement table and the
  existing `CHAN_SEL` counter do the job.

## Consequences

- Agents write the waiting loop they already know, and the checker insists on
  the way out (`closed`).
- `select` as a name in user code is now an error (lex.kw.1); one std local
  (`std/audio/opus.ail`) was renamed.
- Every pass that walks statements handles `Stmt.Select`; the helpers
  `ast.select_heads`, `ast.select_bodies`, `ast.select_var` and
  `ast.select_always_runs` keep them short.
- Later: a send arm with `move x` for unique values; fairness as an opt-in if
  a measured program needs it.
