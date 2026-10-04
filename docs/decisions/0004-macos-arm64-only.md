# 0004. Support macOS on ARM64 only

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 24 — Cross-platform ABI strategy
- **Spec rules:** [plat.target.1](../spec/01-platforms.md)

## Context

Apple has moved the Mac line entirely to Apple Silicon (ARM64) and has
announced that macOS 26 is the last release supporting Intel Macs. Rosetta 2
is being wound down in later releases. AIL will not reach a stable release
until well after this transition is complete.

## Options considered

### Option A — macOS ARM64 only (chosen)
- **Pros:** One ISA on macOS. No x86-64 Mach-O backend or testing. Matches
  where every current and future Mac is headed.
- **Cons:** Excludes users still on Intel Macs. That group shrinks every year
  and gets no new macOS releases.

### Option B — macOS ARM64 + x86-64
- **Pros:** Covers the remaining installed base of Intel Macs.
- **Cons:** Ongoing cost for a platform Apple itself is retiring. Intel Macs
  are also hard to source for CI.

## Decision

macOS is a first-class OS, supported on **ARM64 only**. There is no
`macos-x64` target.

## Rationale

By the time AIL ships, Intel Macs will be an unsupported legacy platform. The
effort is better spent on Tier 1 targets that are growing.

## Consequences

- The Mach-O backend only needs ARM64.
- macOS CI runs on Apple Silicon only.
- Developers on Intel Macs can still cross-compile *to* other targets only if
  the compiler itself runs there. Supported compiler **host** platforms are an
  open question (see [open-questions.md](../open-questions.md)).

## Rejected alternatives

- **macOS x86-64:** a declining platform that Apple is ending. The ongoing
  cost isn't justified.
