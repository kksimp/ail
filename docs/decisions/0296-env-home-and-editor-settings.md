# 0296. `env.home()`, and editor settings in `~/.ail-edit`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture; tooling
- **Spec rules:** [std.env.5](../spec/17-std.md)
- **Extends:** [ADR 0157](0157-standard-os-modules.md), [ADR 0295](0295-editor-themes.md)

## Context

The editor remembers its theme between runs. A program can read `HOME`
with `env.get`, but Windows uses `USERPROFILE`, so every program would
repeat the same fallback. Deciding criteria: **tokens** and **AI
understanding** (one portable call).

## Options considered

### Option A — `env.home() str?` (chosen)
`HOME`, else `USERPROFILE`, else `none`. Effect `env`, like the rest of
`std.env`.

### Option B — leave it to `env.get`
- **Cons:** each program repeats the platform fallback, or gets Windows
  wrong.

### Settings format
- **A — `~/.ail-edit` with lines `KEY VALUE` (chosen):** trivial to read and
  write without a parser; other keys are kept when one changes.
- **B — JSON:** more tokens to read and write, no benefit for one setting.

## Decision

`std.env.home()` returns the home directory or `none`. The editor keeps
its settings in `HOME/.ail-edit`, one `KEY VALUE` per line; today only
`theme`. It writes the file only when the theme changes in the window,
never under `ail test`, and a failure to write is a message, not an error.

## Rejected alternatives

- **`env.get("HOME")` everywhere:** not portable.
- **JSON settings:** heavier for one key.
