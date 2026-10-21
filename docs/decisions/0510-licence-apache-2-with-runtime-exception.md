# 0510. Licence: Apache 2.0 with a Runtime Library Exception

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** 27 — Dependencies and tooling; project governance
- **Spec rules:** none
- **Files:** `LICENSE`, `README.md`, `bootstrap/Cargo.toml`

## Context

The repository carried the Apache License 2.0 with no exception. A
compiler's runtime is linked into every program its users build, and the
Apache attribution terms (section 4) would otherwise reach those programs.
Licensing is the designer's call (AGENTS.md, escalation criterion 4). The
designer stated on 2026-10-20 that the project is "Apache 2.0 with a
Runtime Exception".
The exception wording in `LICENSE` was drafted by the AI and reviewed by
the designer, who approved it on 2026-10-20 ("The license is good"), so it
is the project's final text rather than a draft.

## Options considered

### Option A — Apache 2.0 with a Runtime Library Exception (chosen)
- **Pros:** the toolchain stays Apache-2.0 (patent grant, permissive);
  programs compiled with AIL carry no attribution burden for the embedded
  runtime; the same shape as other compiled languages' licences.
- **Cons:** a custom exception text, so no single SPDX identifier
  (`bootstrap/Cargo.toml` keeps `Apache-2.0` for the crate itself, which
  is accurate: the exception only widens what users may do).

### Option B — plain Apache 2.0
- **Cons:** attribution reaches every compiled program.

### Option C — MIT or dual MIT/Apache
- **Cons:** loses the patent grant; still no runtime exception.

## Decision

Option A. The exception text is appended to `LICENSE`. "Runtime Code" is
the runtime library plus anything the toolchain embeds into, links into
or generates for a user's program (`compiler/rt/` and the native
backend's emitted code; the interpreter is not embedded and stays plain
Apache-2.0).

## Consequences

- Every contribution and every ported algorithm must be
  Apache-2.0-compatible; nothing copyleft, and vendored code keeps its
  notice in `NOTICE` if one is ever needed.
- Code under `compiler/rt/` and the emitted code of the native backend
  (lanes 120, 122, 126 of the 2026-10-20 batch) fall under the exception;
  the lane rules say so.
- The README's licence section and `AGENTS.md`'s escalation rule are the
  places a contributor sees this.

## Rejected alternatives

- Plain Apache 2.0 (option B): rejected because the attribution terms would reach every compiled program through the embedded runtime.
- MIT or dual MIT/Apache (option C): rejected because it loses the patent grant, and it still has no runtime exception.
