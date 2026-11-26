# 1184. `ail eval`: the value and type of expressions, with no file

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); a new command, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [eval.part.1-3, eval.run.1, eval.out.1-3](../spec/22-tools.md)

## Context

The designer asked (2026-11-25) which tools the agent runs show are missing. In the 464 transcripts of the final
run no AIL tool beyond `build`, `card` and `ask` was called, so a tool is used only when it has the shape of
something agents already type. One such shape has no AIL form: `python3 -c`. In the app-build test
([agent findings](../design/agent-findings-2026-11-25.md), source B) the AIL build took 11 turns against Python's 3,
most of them settling from the docs what a call does on an edge input (does `pad_left` cut a longer string, what
`words()` splits on), and one such doubt was shipped untested. The card cannot hold every edge of every method;
running the call can answer any of them.

## Decision

- `ail eval PART...`: each argument is one line. An expression prints `PART  =>  VALUE  (TYPE)`; a binding, an
  assignment or a `use M` line sets up for later parts. Several questions cost one call.
- It is `tools/aileval/aileval.ail`, a program the launcher runs, like `ail try`. It adds no analysis: the parts
  become one `main` in a scratch file, `ail api --locals` checks it and gives the types, `ail run` runs it.
- The scratch file sits in the current directory, so `use` finds the project's own modules and an agent can call
  its own functions on an input.
- A value prints as inside a list, so a `str` shows its quotes and spaces.
- A part that does not check names itself (`part 2`), never the scratch file.

## Options considered

- **A REPL (`ail repl`):** rejected. An agent's shell call is one shot; a session that waits for input is the
  shape agents avoid.
- **`ail run -e CODE`:** rejected as the main form. It reads as "run a program", prints nothing unless the code
  prints, and gives no type. `eval` says what comes back.
- **In the native CLI (`compiler/main.ail`):** rejected for now. The tool needs no compiler internals, and
  `main.ail` is a hot file (ADR 0823). An in-process form is a later speed-up with the same output.
- **More edge facts on the card instead:** done where they are common (ADR 1176), but the card has a budget
  (ADR 1110) and the questions have no end.
- **Scratch file in the temporary directory:** rejected; `use` of the project's modules would not resolve.

## Consequences

- New `tools/aileval/aileval.ail`; one block in `ail`, two lines in `ail.ps1`, one entry in `compiler/help/help.ail`.
- One line on the language card and in `docs/for-agents.md`. The `help` and `explain` goldens and the cards need
  regenerating.
- Each call checks and builds a small program (about half a second). Not built in the cloud (ADR 0842); gated on
  the Mac.
