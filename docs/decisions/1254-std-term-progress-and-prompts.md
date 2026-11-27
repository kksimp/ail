# 1254. `std.term`: styled text, progress bars, spinners and prompts

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.term`)
- **Spec rules:** std.term.9-12 ([std-term.md](../spec/std-term.md))
- **Number:** the integrator gives the ADR number; rename this file and replace `ADR 1254` in `std/term/term.ail`, `docs/spec/std-term.md` and `docs/status.md`, and add the index row.

## Context

`std.term` (ADR 1012) has styles, cursor moves and raw input, but a command-line tool that only wants a coloured word, a progress bar, a yes or no question or a password has to build them from escape sequences, and each tool must remember `NO_COLOR` and not drawing into a pipe. Agents write exactly these tools.

## Decision

Add to `std.term`, in `std/term/term.ail`: `color_enabled` (`NO_COLOR`, `TERM=dumb`, `FORCE_COLOR`, standard output a terminal), `paint`, `paint_if` and named colours; `bar`, `progress_text`, `Progress` and `Spinner`, which write nothing unless colour is on; and the prompts `confirm`, `ask`, `choose` and `password`, which read a line of standard input when not on a terminal. On a terminal `choose` and `password` use raw mode without the alternate screen. Failures are values (`none`), not traps.

## Options considered

### Option A: functions in `std.term` (chosen)
- **Pros:** one module, no registration lines, no runtime piece; the raw mode, key decoder and probes of ADR 1012 are reused, on macOS, Linux and Windows alike.
- **Cons:** `std.term` grows by about 300 lines.

### Option B: a new module `std.cli`
- **Pros:** a smaller `std.term`.
- **Cons:** a second place to look for terminal output, five registration lines, and it needs the same raw mode helpers.

### Option C: `termios` flags for the password
Rejected: `c_lflag` sits at byte 24 on macOS and 12 on Linux, and `cfmakeraw` is already here.

### Option D: progress lines when not on a terminal
Rejected: a log full of bars. Callers log milestones themselves.

## Consequences

`select` is a keyword (`lex.kw.1`), so the menu is `choose`. The prompts that read a line are not run in the gate (no stdin there); their parsers, the bar text and the no-output behaviour are (`tests/cases/std_term_style.ail`).
