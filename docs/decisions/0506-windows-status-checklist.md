# 0506. Windows is checked by one checklist the designer can run top to bottom

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Platforms (Windows)
- **Spec rules:** none new; it cites [ADR 0071](0071-one-command-scripts.md)
- **Criterion that decided:** Speed of the designer's check (criterion 4) and understanding. The designer tests Windows when convenient, so the checklist must be copy-paste.

## Context

ADR 0071 names `run.ps1` for setup scripts. The CLI on Windows is `ail.ps1` at the
repository root, which says it is "tested under PowerShell 7 on Linux; not yet tested on Windows
itself". Lanes 111–154 add modules (sockets, dates, compression, FFI, regex, bytes) with
platform code. Each needs a Windows branch or a cited trap, and the designer needs one place to check them.

## Options considered

### Option A — a note in each ADR
- **Pros:** local to each module.
- **Cons:** the designer must hunt across ADRs.

### Option B — one checklist file, `docs/design/windows-status.md`, one command per line (chosen)
- **Pros:** one file to run top to bottom, and each item says what to expect.
- **Cons:** it must be kept current; each lane that adds platform code adds its item.

## Decision

Option B. Platform-specific Rust is reviewed for these classes, and each class is in the checklist:
path separators and drives (ADR 0504); process lookup (ADR 0505); TLS root store and randomness (already `cfg(windows)` branches in `net/x509/roots.rs` and `net/crypto/rand.rs`); console text (Rust's standard library writes console output as UTF-16 with `WriteConsoleW`, so no change is needed); line endings (the lexer and formatter accept CRLF); and the UI (`bootstrap/src/ui.rs` has Windows key and metric branches).

## Consequences

- `ail.ps1` remains the Windows CLI. ADR 0071 covers the setup scripts.
- Modules not yet on the batch branch (bytes, FFI loading, dates and zones, compression, regex, and the server sockets from lane 124) must each add a checklist item when they land.

## Rejected alternatives

- Option A, above.
