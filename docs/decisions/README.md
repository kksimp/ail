# Architecture Decision Records

Each design decision has its own file (see [ADR 0001](0001-record-decisions-as-adrs.md)).
New ADRs copy [0000-template.md](0000-template.md) and take the next free number.
ADRs are never deleted. Replaced decisions are marked `Superseded by NNNN`.

| # | Title | Status | Date |
|---|---|---|---|
| [0001](0001-record-decisions-as-adrs.md) | Record design decisions as numbered ADRs | Accepted | 2026-10-04 |
| [0002](0002-normative-spec-with-rule-ids.md) | Normative specification with stable rule IDs and tested examples | Accepted | 2026-10-04 |
| [0003](0003-documentation-in-repo-markdown.md) | Documentation as Markdown in the repo; AIL docs tool later | Accepted | 2026-10-04 |
| [0004](0004-macos-arm64-only.md) | Support macOS on ARM64 only | Accepted | 2026-10-04 |
| [0005](0005-x86-64-baseline-v3.md) | x86-64 baseline is x86-64-v3, with runtime dispatch | Accepted | 2026-10-04 |
| [0006](0006-arm64-baseline-v8-2.md) | ARM64 baseline is ARMv8.2-A | Accepted | 2026-10-04 |
| [0007](0007-64-bit-little-endian-only.md) | 64-bit little-endian targets only | Accepted | 2026-10-04 |
| [0008](0008-isa-priority-policy.md) | ISA priority policy (ARM64 vs x86-64) | **Proposed** | 2026-10-04 |
| [0009](0009-custom-tooling-policy.md) | Build custom tooling unless an existing tool matches every requirement | Accepted | 2026-10-04 |
| [0010](0010-no-llvm-backend.md) | No LLVM or other external backend; existing compilers for comparison only | Accepted | 2026-10-04 |
| [0011](0011-ai-familiarity-not-a-constraint.md) | AI training-data familiarity is not a design constraint; open-source intent | Accepted | 2026-10-04 |
| [0012](0012-memory-rd-tracks.md) | Pursue four memory R&D tracks (allocation ladder, task-local heaps, in-place reuse, inferred arenas) | Accepted | 2026-10-04 |
| [0013](0013-compiler-chosen-layout.md) | Data layout compiler-chosen by default, fixed layout on request | Accepted | 2026-10-04 |
| [0014](0014-compressed-managed-references.md) | Managed references may be compressed; raw pointers stay 64-bit | Accepted | 2026-10-04 |
| [0015](0015-always-whole-program-incremental.md) | Always whole-program optimisation, with fine-grained incremental caching | Accepted | 2026-10-04 |
| [0016](0016-optimized-and-observable-debugging.md) | Debug builds are fully optimised and fully observable | Accepted | 2026-10-04 |
| [0017](0017-no-c-emission.md) | Never emit C as a path to native code | Accepted | 2026-10-04 |
| [0018](0018-optimizing-backend-from-start.md) | Build an optimising native backend from the start | Accepted | 2026-10-04 |
| [0019](0019-miscompilation-defence.md) | Four-layer defence against miscompilation | Accepted | 2026-10-04 |
| [0020](0020-linux-no-libc.md) | Linux runtime uses raw syscalls, no libc | Accepted | 2026-10-04 |
