# 0499. Every checked spec program runs under both interpreter tiers and must print the same

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Conformance (lane 151, batch 2)
- **Spec rules:** none new
- **Confirms:** [ADR 0361](0361-bytecode-tier-design.md) (the tree walker is the oracle for the bytecode tier),
  [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)

## Context

The bytecode tier (ADR 0361) is only checked where the selftest's example
programs and cases exercise it. The spec examples are the language's own
statement of behaviour, so they should also agree between the two tiers.

## Decision

A block that checks clean, has a `main`, and has no `norun`, `flags=`, `tool=`
or `fixture=` tag is run twice with `ail run`: once on the bytecode tier and
once with `--no-bytecode`. Both must exit 0 and print the same stdout. Each run
starts in a scratch directory, so file examples can make and remove files.

`norun` marks a block that traps on purpose or needs input the harness doesn't
have. Such a block is checked and not run.

## Rejected alternatives

- **Run only on the bytecode tier.** Rejected: the tree walker is the oracle
  (ADR 0032), and a difference is a bug in whichever tier is wrong.
- **Compare against an expected-output block.** Rejected for now: it doubles
  the spec's text, and the two tiers already agree on what they print. A later
  change can add `out=` blocks where a printed value matters.
- **Run blocks with `flags=` too.** Rejected: the flags (`--deterministic`,
  `--deny`) reject the program, so there is nothing to run.

## Consequences

Examples that depend on the machine (the clock, the environment, a window, the
network) are either `norun` or print a value that doesn't depend on it.
