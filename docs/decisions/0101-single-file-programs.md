# 0101. A single `.ail` file can be run directly without a package manifest

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 12 — Modules; 27 — Build system
- **Spec rules:** [mod.unit.3](../spec/12-modules.md)

## Deciding criterion

**Token usage:** a manifest file is pure overhead for scripts and examples.
**AI understanding** is unaffected.

## Decision

`ail run path/to/file.ail` treats the file as a complete program, a module
of its own whose only dependency is the standard library. Multi-module
programs and external dependencies need a package (manifest format to be
decided with the build system).

## Consequences

- Hello World needs only `examples/hello.ail` (ADR 0086).
