# 0157. The machine is reached through five built-in standard modules

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules; standard library
- **Spec rules:** [std.use.1–3, std.fs.1–7, std.env.2–3, std.io.1–4, std.time.1–2](../spec/17-std.md)
- **Plan:** [ai-first-plan.md §4](../design/ai-first-plan.md)
- **Evidence:** tokbench on `examples/tool.ail` and two rewrites of it, mean of five tokenizers

## Context

Plan §4: AI-written tools need files, the environment, standard input and
error, processes and the clock, through typed functions whose failures are
values, not a string-typed shell. `use` (ADR 0099) was specified but not
implemented. User modules are implemented by a separate lane at the same
time, so the standard modules must follow the same import rules.

## Options considered

| Option | tool.ail tokens (mean) |
|---|---|
| **`use std.fs`, then `fs.read_text(p)` (chosen)** | 359.0 |
| Prelude functions, no import: `read_text(p)`, `run_process(...)` | 342.4 (−4.6%) |
| Shorter names: `fs.read(p)`, `proc.exec(...)` | 356.2 (−0.8%) |

## Deciding criterion

The prelude option is 4.6% cheaper on this 40-line tool, but almost all of
the difference is the three `use` lines, a fixed cost per file that shrinks
as programs grow; per call, the `fs.` prefix is about one token. That option
also isn't open to a delegated decision: it contradicts ADR 0099 (contents
of a module are used by qualified name) and ADR 0084's requirement that the
prelude stay small, so common names (`run`, `get`, `exit`, `args`) stay free
for locals. Within the qualified design, tokens **tie** (shorter names save
0.8%), and **AI understanding decides**:
- `fs.read_text` says what it returns, and leaves room for `read_bytes`;
- the `use` lines at the top of a file state which parts of the machine the
  file can touch, which the capability lane (plan §3.2) builds on;
- every name is one meaning (`proc.run` can't be confused with a local `run`).

## Decision

`use std.NAME` imports one of five built-in modules: `fs`, `env`, `io`,
`proc` and `time`, with the functions of spec chapter 17. Their types are
qualified too (`fs.IoError`, `proc.Output`). The brief's signatures are kept,
with three additions: `fs.make_dir` (a tool that writes files needs a
directory to put them in), `env.program` (ADR 0159), and `remove` accepting
an empty directory (so `make_dir` can be undone; it never removes a tree).

## Consequences

- An agent tool fits in one file with no dependencies (`examples/tool.ail`).
- The bootstrap declares the types in `bootstrap/src/std/types.ail` and
  renames each into its module, so they can't clash with a program's names.
- `std.json` (plan §4) is prelude-level tonight; making it `use std.json`
  is a one-line addition to `stdlib::MODULES` once its lane lands.
- Revisit: if the designer prefers no imports for the standard modules
  (−4.6% on small tools), that means amending ADR 0099 for `std`.

## Rejected alternatives

- **Prelude functions:** contradicts ADRs 0084 and 0099, and hides which
  files touch the machine; cheaper only by fixed `use` lines.
- **Shorter names (`read`, `exec`):** a 0.8% tie, and `read` doesn't say
  whether it returns text or bytes.
- **A shell function (`sh("...")`):** string-typed, platform-dependent and
  injection-prone; plan §4 rules it out.
