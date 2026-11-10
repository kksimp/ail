# 0898. A database engine and its shell, as a showcase program

- **Status:** Deferred past 0.1.0
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the program on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0874](0874-purity-driven-speed-optimizations.md), [ADR 0891](0891-ail-context-budgeted-pack.md), [ADR 0896](0896-ail-test-mutate.md), [ADR 0897](0897-ail-check-perf.md)
- **Topic:** 24 — Showcase programs

## Context

The editor, PDF viewer and Garlic show AIL on interfaces. None shows it on a dense, speed-bound data structure program, where C is the usual choice and Python is too slow. A database also needs correctness work that AIL's tools are meant to help with.

## Decision

- Build `tools/db`: a single-file store with a page-based file format, a B-tree index, a write-ahead log, and a small SQL subset (`create table`, `insert`, `select ... where ... order by ... limit`, `update`, `delete`, joins on one key). A shell (`db FILE`) reads statements and prints rows.
- It is also the optimizer benchmark: insert, point lookup and range scan are run against SQLite through its C API for reference, and the result goes in `tools/bench` beside the other programs.
- It shows: contracts on the page and B-tree invariants; property tests that compare the engine with a plain-list model; `ail test --mutate` on the log and recovery code; `ail check --perf` on the scan loop; a recording of a crash during a write, replayed to the failing byte; and `@tool` (ADR 0892) so an agent can query it.
- Not in scope: concurrent writers, network protocol, full SQL.

## Consequences

It is a large program, so it lands in slices: pages and B-tree first, then the log, then SQL. Each slice has its own tests and a bench row.

## Rejected alternatives

- **A key-value store only:** shows less of the language's data handling and has no query text for agents to write.
- **Embedding SQLite through FFI:** shows nothing about AIL.
- **A server first:** adds networking work before the engine's speed is known.
