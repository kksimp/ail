# 0295. Editor themes recolor the default colors by kind, from an AIL module

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); uses [std.ui.17](../spec/17-std.md), [std.env.5](../spec/17-std.md)
- **Extends:** [ADR 0181](0181-editor-screen-as-text-runs.md), [ADR 0294](0294-std-ui-styles-colors-caret.md)

## Context

The editor should offer light and dark themes (`--theme`, Ctrl-Shift-T),
defined in AIL. Its headless core renders a `Screen` of runs colored with
about twenty constants (`BG`, `KEYWORD_FG`, ...), used throughout the
render code that tests check. Deciding criteria: **AI understanding** (one
place to read a theme; the core and its tests unchanged), then tokens.

## Options considered

### Option A — recolor at draw time (chosen)
A theme is a `Map[str, int]` from a color kind (`"bg"`, `"keyword"`, ...)
to a color, in the module `tools/edit/themes`. The window front end maps
each run's colors through it: text colors by the text kinds, backgrounds
by the background kinds. A kind a theme leaves out keeps the dark default.
- **Pros:** the core, its render and its tests don't change; a theme is a
  plain table; adding a kind is one line in each place.
- **Cons:** two kinds with the same default color can't differ (only
  `OP_FG` and `FG` today, so `op` isn't a kind).

### Option B — a `Theme` struct threaded through `render`
- **Pros:** every color explicit.
- **Cons:** touches every color use in the core and every render test.

## Decision

Option A. The themes are `dark` (the defaults), `light`, `solarized` and
`high-contrast`; `themes.NAMES` gives the Ctrl-Shift-T order. `--theme
NAME` picks one for the run; Ctrl-Shift-T moves to the next and remembers
it in `~/.ail-edit` ([ADR 0296](0296-env-home-and-editor-settings.md)). An
unknown name keeps `dark` and says which names exist. The themes live in a
directory module (`tools/edit/themes/themes.ail`, `use themes`) because a
file run alone doesn't see its sibling files (ADR 0097).

## Rejected alternatives

- **Threading a theme through render:** large change to code another lane
  owns, for no visible difference.
