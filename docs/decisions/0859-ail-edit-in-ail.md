# 0859. `ail edit` is ported to AIL (`compiler/edit/`), byte-identical to the Rust tool

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076; allocated by the integrator)
- **Topic:** 23 — Rust removal (RR-5)
- **Spec rules:** edit.id.1-3, edit.op.1-7, edit.in.1-3, edit.out.1-4
- **Builds on:** [ADR 0397](0397-ail-edit-structured-edits-by-id.md), ADR 0855

## Context

RR-5 moves the formatter, `ail edit`, `ail fix` and the doc tools from
`bootstrap/src/*.rs` to AIL. The formatter went first (ADR 0857). `edit.rs`
works on text and tokens, so it needs no checker.

## Decision

1. **`compiler/edit/` (`json.ail`, `scan.ail`, `ops.ail`, `cli.ail`) implements
   every text operation of `edit.rs`:** `replace_body`, `replace`, `insert`,
   `delete`, `hunk` and `import`, the compact syntax, JSON and stdin edits, and
   the flags `--dry-run`, `--diff`, `--if-clean` and `--json`. Output, exit
   status and written files match the Rust tool. Changed files go through
   `compiler/fmt`.
2. **The check after an edit is syntax only for now,** with a note on stderr
   saying so. The semantic check returns when RR-1/RR-2 provide the module
   loader and the checker driver.
3. **`rename` and `--test` are refused with a message** until the same two
   pieces exist (`rename` needs resolved names, `--test` runs the project).
4. **Proof:** a scenario harness runs both tools on temp copies of repository
   files and compares stdout, stderr, exit status and the resulting files
   (867 runs, no difference).

## Rejected alternatives

- **Wait for the loader and port all of `edit.rs` at once.** The text
  operations do not depend on it, and this lets RR-1 route `ail edit` to AIL
  without waiting for the semantic check.
- **Fake the semantic check by accepting every edit.** An edit that breaks a
  program would pass silently; a stderr note about the syntax-only check is
  honest.
