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
| [0008](0008-isa-priority-policy.md) | ISA priority policy: ARM64 and x86-64 in strict lockstep | Accepted | 2026-10-04 |
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
| [0021](0021-debug-info-formats.md) | Emit AIL's own debug format plus DWARF and PDB | Accepted | 2026-10-04 |
| [0022](0022-staged-self-hosting.md) | Staged self-hosting: backend written in AIL from the start | Accepted | 2026-10-04 |
| [0023](0023-rust-bootstrap.md) | Bootstrap front end and interpreter written in Rust | Accepted | 2026-10-04 |
| [0024](0024-semantic-api-from-stage-2.md) | AI semantic API starts in Stage 2 and grows each stage | Accepted | 2026-10-04 |
| [0025](0025-memory-model-atomics.md) | Abstract weak memory model; data-race-free safe code; SeqCst-default atomics | Accepted | 2026-10-04 |
| [0026](0026-hardware-feature-acceleration.md) | Portable runtime core; ISA-specific features only as optional accelerators | Accepted | 2026-10-04 |
| [0027](0027-ci-gating-all-targets.md) | All five Tier 1 targets gate every merge | Accepted | 2026-10-04 |
| [0028](0028-git-workflow.md) | Git workflow: `main` is the only long-lived branch | Accepted | 2026-10-04 |
| [0029](0029-token-efficiency-benchmarks.md) | Token-efficiency benchmarks: corpus, multiple tokenizers, AI task runs | Accepted | 2026-10-04 |
| [0030](0030-performance-benchmarks.md) | Performance benchmarks: standard suites, real apps, automated drag races | Accepted | 2026-10-04 |
| [0031](0031-hosts-and-cross-compilation.md) | Compiler runs on all five platforms and builds for all of them | Accepted | 2026-10-04 |
| [0032](0032-keep-rust-interpreter-as-oracle.md) | Keep the Rust interpreter permanently as an independent oracle | Accepted | 2026-10-04 |
