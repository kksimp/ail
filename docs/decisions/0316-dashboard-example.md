# 0316. `examples/dashboard.ail` prints by default and opens a window with `--window`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Examples and testing
- **Spec rules:** [std.ui.21](../spec/17-std.md)
- **Extends:** [ADR 0315](0315-widget-contract.md)

## Context

Every example runs in `./ail selftest` and on the designer's Mac with no
arguments and must print its `.out`. On macOS a window always opens, so an
example that opened one by default would hang the test run.

## Decision

The dashboard (files of `examples/data` with sizes, rescanned by a task,
and a preview pane) prints one screen through `widgets.text_rows` by
default; `--window` opens it. Rejected: a `--print` flag, which would make
the default the window and hang the tests on macOS.

## Consequences

- `./ail run examples/dashboard.ail --window` is the interactive demo.

## Rejected alternatives

- A `--print` flag on the dashboard, with the window as the default: rejected because the default would then open a window, and on macOS that would hang the test run, since every example runs with no arguments.
