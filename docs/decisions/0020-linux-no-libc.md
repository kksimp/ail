# 0020. The Linux runtime does not use libc; it calls the kernel directly

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 22 — Runtime architecture; 21 — Standard library; 14 — FFI
- **Spec rules:** none
- **Resolves:** [OQ-014](../open-questions.md)
- **Constraints:** [design/toolchain-constraints.md](../design/toolchain-constraints.md)

## Context

Linux is the only Tier 1 OS with a stable kernel syscall ABI. macOS and
Windows require going through system libraries (`libSystem`, `ntdll`/`kernel32`).
On Linux, libc (glibc or musl) is optional.

## Options considered

### Option A — No libc; raw syscalls (chosen)
- **Pros:** Full control. Fully static binaries. No glibc-version breakage
  across distributions. Fits ADR 0009.
- **Cons:** AIL must implement thread creation, thread-local storage, DNS
  resolution, time functions (including loading the kernel's vDSO) and
  similar itself. Calling C libraries through FFI requires a libc to be loaded.

### Option B — Link libc
- **Pros:** Easier, and C interop works out of the box.
- **Cons:** Depends on libc behaviour and versions. Fails ADR 0009's "meets
  every requirement" test.

### Option C — No libc by default, opt-in libc
- **Pros:** Best of both.
- **Cons:** Two runtime configurations to test.

## Decision

AIL's Linux runtime and standard library talk to the kernel directly through
syscalls. AIL programs on Linux don't depend on libc.

## Consequences

- AIL must implement its own thread runtime, TLS, `vDSO` loading, DNS resolver,
  and the other services libc normally provides.
- **FFI interaction (open question):** C libraries expect libc to be present
  and initialised (malloc, errno, TLS layout, pthreads). How AIL programs that
  call C libraries on Linux get a libc (e.g. load one only when FFI is used,
  and make AIL's threads compatible with it) must be settled in the FFI topic.
  Tracked as OQ-021.
- Linux binaries can be fully static and portable across distributions.

## Rejected alternatives

- **Link libc:** an external dependency that fails the custom-tooling test.
- **Opt-in libc:** not chosen. The FFI case is handled as OQ-021 instead.
