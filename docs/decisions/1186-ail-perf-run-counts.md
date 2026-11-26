# 1186. `ail perf FILE`: where a run spends its statements

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer chose to build it (2026-11-25); a new command form, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [tool.perf.3-5](../spec/22-tools.md), [native.instr.4](../spec/23-native.md)

## Context

The speed mandate has agents make AIL programs fast, and the only tool was `ail perf --explain FN FILE`, which
reads one function without running it. To find which function to read, an agent in the runs added timers or
counters by hand, or guessed. The native backend already places a probe at every statement for coverage and
tracing (`AIL_INSTRUMENT`, native.instr.1-3), but those probes record that a statement ran, not how often.

## Decision

- `ail perf FILE [ARGS...]` builds FILE with a counter at each statement, runs it, and prints on standard error
  the ten functions and the ten lines that ran the most statements, with shares and calls. The program's own
  output and exit status pass through.
- A new instrument mode `counts` (native.instr.4) reuses the statement probes: each adds one to an 8-byte cell
  in its own table, written out at exit as `ID\tCOUNT`.
- The report is `tools/ailperf/ailperf.ail`, a program the launcher runs, which joins the counts with the site
  table.
- The unit is statements run, not time. A count is the same on every run and every machine, costs no sampling
  and needs no timer; the report says so in its first line.
- Loops the compiler would split over cores run on one thread during the measured run, so the counts are exact.

## Options considered

- **Sampled time (a timer signal and stack walks).** Rejected for now: per-host signal and unwinding code on
  three systems, and results that vary by run; a count already answers "where does the work go" for the
  programs agents write. Time per line can follow under the same command.
- **A `--profile` option on `ail run`.** Rejected: agents reach for a command named for the question, and
  `ail perf` already is that command; one more form of it is easier to find than a `run` option.
- **Count in the existing hit table.** Rejected: coverage and tracing read a byte per site; a second table keeps
  them unchanged.
- **Atomic counters.** Rejected: the measured run is on one thread, and an atomic add at every statement would
  slow it for no gain. A program that spawns tasks may lose a count, which the spec states.

## Consequences

- The measured build is slower than a plain build (one memory add per statement) and is never cached.
- Statement counts weigh a cheap statement like a costly one; the next step for a hot function is still
  `ail perf --explain FN FILE`, and the report's lines point there.
- `ail.ps1` has no route yet (Windows parity is deferred).
