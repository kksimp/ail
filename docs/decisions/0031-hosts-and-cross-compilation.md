# 0031. The compiler runs on all five Tier 1 platforms and builds for all of them

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 27 — Build system; 24 — Cross-platform
- **Spec rules:** none
- **Resolves:** OQ-005

## Context

The compiler's *host* platforms (where it runs) are separate from its
*targets* (what it produces). With AIL's own linker (ADR 0009), any host can
in principle produce binaries for any target.

## Options considered

### Option A — All five hosts, each able to build for all five targets (chosen)
- **Pros:** Symmetric. One machine can produce every release, and developers
  on any OS can build for every target.
- **Cons:** AIL must generate its own link-time descriptions of system
  libraries (macOS `libSystem` stubs, Windows import tables) instead of
  using vendor SDKs.

### Option B — All five hosts, each building only for its own OS
- **Pros:** Can link against the real system SDK files.
- **Cons:** Releases need machines for every OS, and Linux can't build for
  macOS.

## Decision

The `ail` compiler runs on `linux-x64`, `linux-arm64`, `macos-arm64`,
`windows-x64` and `windows-arm64`. From any host it can produce binaries for
all five targets, with no vendor SDK installed.

## Consequences

- AIL ships its own **system-library interface descriptions** (the symbols
  and versions needed to link against `libSystem`, `kernel32`, `ntdll`, etc.).
  **Constraint:** Apple's SDK licence restricts using Apple SDK files on
  non-Apple hardware, so these descriptions must be generated
  independently, not copied from the SDK. This needs a legal review before
  release.
- AIL's linker writes the ad-hoc macOS code signature itself, so unsigned
  development builds for `macos-arm64` work from any host.
- **Distribution signing remains external**
  ([toolchain-constraints.md](../design/toolchain-constraints.md)). Apple
  notarization uses Apple's online service, which can be called from any OS.
  Windows Authenticode needs a certificate.
- Cross-compiled binaries can only be **run** on their native target, so the
  tests themselves still run on all five CI targets (ADR 0027).

## Rejected alternatives

- **Own-OS builds only:** gives up single-machine releases and symmetric
  developer workflows.
