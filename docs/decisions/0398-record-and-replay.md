# 0398. Record and replay: effect results as JSON lines, answered back in a replay

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools; testing and debugging
- **Spec rules:** [replay.rec.1–5, replay.cap.1–2, replay.run.1–3](../spec/22-tools.md#record-and-replay)
- **Builds on:** [ADR 0206](0206-std-random.md) (seeded `std.random`), the trap snapshots of `trace.snap.1–4`

## Context

A failure an agent can't reproduce costs the most tokens of any: it reruns
the program, adds prints, reads the inputs into its context to build a
fixture, and still misses a failure that depended on the clock, a random
draw, an environment variable, standard input or a server's answer. The
plan (finish-plan.md §2.4) asks for `ail run --record` and `--replay`, a
trap in a replay at the same statement, and `ail test --record` on failure.

## Options considered

### Option A — record at the operating system (syscalls), as `rr` does
- **Cons:** per platform, large, and records the interpreter's own reads
  (the program text, packages) along with the program's; a recording can't
  be read or edited by an agent.

### Option B — record the results of the standard calls that reach the world (chosen)
- **Pros:** one choke point in the interpreter (`std_call`), the same on
  every platform; the recording is JSON lines an agent can read, grep and
  edit to try a variant ("what if the file had been empty?"); positions in
  it point at the source; replays need no files, network or processes.
- **Cons:** only effects through the standard library are captured (all
  there are, as AIL has no FFI yet); a handle (a process, a socket) can't
  be written down, so a replay stops at the call that made one.

### Option C — record inputs only (stdin, args, environment), as a fixture
- **Cons:** misses files, the clock, the network and processes, which are
  the usual causes of an unreproducible failure.

Details decided with Option B:

- **What is captured** (`replay.cap.1`): every `fs`, `env`, `time`, `tz`,
  `net` and `proc` function, the `io` reads and the clipboard. Writes are
  captured as well, so a replay writes nothing (re-running a recorded
  `fs.remove` is never what an agent wants).
- **What is not captured:** output (`print`, `io.eprint`), which the replay
  produces again to compare; `std.random`, which is repeated by recording
  the seed (a clock seed when none was given) instead of every draw; tasks;
  pure modules; methods on handles; and handles themselves (`replay.cap.2`).
  Task interleaving is not recorded: a recorded run keeps loops on one
  thread (`replay.rec.5`), and programs whose tasks race on effects replay
  only if the scheduler repeats its order (`--schedule=random:SEED` makes
  it repeat).
- **Format:** a header line, then one line per call in order with its
  position and result (`replay.rec.2–3`). Values are tagged by type name,
  not index, so a recording survives unrelated edits to the program; a
  float keeps its exact bits through Rust's shortest round-trip text.
  Rejected: one JSON document (can't be appended or streamed, and a cut
  file isn't valid); a binary format (agents can't read it).
- **Size limit:** 64 MiB of call lines (`replay.rec.4`). A run that reads
  more stops recording and marks the file truncated; the replay runs up to
  that point and then traps saying why. Rejected: no limit (one loop over a
  large directory fills a disk); a limit on calls (one call can be 1 GB).
- **Divergence** (`replay.run.3`): calls are matched in order by name. A
  mismatch traps at the program's call, naming the recorded call and its
  position, so the agent sees exactly where its edit changed the effects.
- **Tests** (`replay.rec.1`, `replay.run.1`): `ail test --record` keeps
  only failing tests' calls, each under `{"test":NAME}`, and starts every
  test's random draws from the recording's seed so one test replays alone.

## Measurements

Reproducing a failed run for a later session (or another agent) the old
way means carrying the run's inputs into context and rebuilding them; the
new way is one command, and the recording is never read. Estimated tokens
(ADR 0311) of the recorded inputs of examples, from their recordings:

| Example | Captured calls | Inputs to carry (old way, at least) | `ail run --replay r.json` | Saved |
|---|---|---|---|---|
| `logscan` | `env.program`, `fs.read_text` | 876 | 9 | 99% |
| `fetch` | `env.args`, `env.program`, `fs.read_text`, `net.get` | 336 | 9 | 97% |
| `digest` | `env.args`, `env.program`, `fs.read_text`, `net.get` | 218 | 9 | 96% |
| `config` | `env.program`, `fs.read_text` | 121 | 9 | 93% |

The old way's figure leaves out the reruns and prints needed to catch a
failure that depends on the clock or a random draw, which a recording
removes altogether.

## Decision

Option B with the details above.

## Consequences

- An agent can attach a recording to a bug report or a failing test, and
  any later session reproduces it with `--replay`, with the trap snapshot
  of `trace.snap.1`.
- `ail run --replay` never touches the file system or network, so it is
  safe to run on a recording from elsewhere.
- Handles end a replay; recording the messages exchanged over a process's
  pipes or a socket is future work.
- Windows: nothing in the format or the hook is platform-specific
  (ADR 0004).

## Rejected alternatives

- Recording at the operating system with syscalls, as `rr` does (option A): rejected because it is per platform and large, it records the interpreter's own reads, and an agent cannot read or edit the recording.
- Recording only the inputs (stdin, arguments, environment), as a fixture (option C): rejected because it misses files, the clock, the network and processes, the usual causes of an unreproducible failure.
- One JSON document as the format: rejected because it cannot be appended or streamed, and a cut file is not valid.
- A binary format: rejected because agents could not read it.
- No size limit on a recording: rejected because one loop over a large directory fills a disk. A limit on calls was also rejected, since one call can be 1 GB.
