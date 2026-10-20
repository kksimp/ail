# 0477. Agent corpus round three: ten programs on today's standard library

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Token efficiency, agent programs
- **Spec rules:** none (measurement only, [ADR 0070](0070-token-bench-tool.md))
- **Builds on:** [ADR 0337](0337-agent-corpus-round-two.md)

## Context

The agent corpus ([ADR 0217](0217-agent-corpus-standing-benchmark.md), grown in
[ADR 0337](0337-agent-corpus-round-two.md)) measures programs an AI agent writes
for its own tooling. The designer's plan for this batch asks for the corpus to
grow with programs that use the features that exist now, so the measurement
says where AIL costs or saves tokens on realistic tools. Several planned
features (regex, dates, bytes, bit operators, server sockets, compression) are
not on this branch yet.

## Decision

Add ten programs (`logwatch`, `backup`, `poller`, `supervise`, `report_dates`,
`hexdump`, `route`, `perms`, `csv2json`, `matrix`) with Python twins. Each
program uses only what the branch provides, and where a planned library is
missing it uses the nearest existing one and says so in its header:

- `backup` and `hexdump` shell out to the system `tar` and `od`, since there is
  no compression or bytes module yet;
- `route` runs one route on sample request lines; the socket loop waits on
  lane 124;
- `report_dates` and `perms` build dates and bits from integers, since there is
  no date module and no bit operators yet.

The twins use the Python standard library as an agent would write them
(`datetime`, `tarfile`, `subprocess`, `json`, `&`), so a program that costs more
than its twin shows where the library or operator is missing, not a
hand-made disadvantage.

## Measurement

Mean of the five offline tokenizers, comments removed, both sides keep blank
lines. Total: 2,488.4 AIL against 2,148.6 Python (+15.8%). Four programs are
under Python (`matrix` −43%, `poller` −12%, `logwatch` −6%, `supervise` −0.4%);
six are above it, and four of those trace to missing library items. The
table and the causes are in [program-measurements-agent-r3.md](../design/program-measurements-agent-r3.md).
Proposals 7.1–7.4 in [ai-author-wishlist.md](../design/ai-author-wishlist.md)
name the four gaps.

## Options considered

### Option A — wait for the library lanes, then write the corpus (rejected)
- **Pros:** programs use the final APIs.
- **Cons:** the measurement would arrive after the batch, and the current gaps
  would not be visible to the designer.

### Option B — write the programs on today's library and measure the gap (chosen)
- **Pros:** the gaps are measured where they occur; when a lane lands, its
  program gets a second measurement against the same twin.
- **Cons:** some programs use a system tool (`tar`, `od`) in place of a native
  module; their header says so, and they are a weaker test of AIL itself.

### Option C — the ten-program corpus replaces the existing ones (rejected)
- **Pros:** one number.
- **Cons:** the existing round results would be lost; the corpus keeps growing
  instead ([ADR 0337](0337-agent-corpus-round-two.md) rejected the same).

## Consequences

- The corpus is 30 agent programs in `tools/tokbench/corpus/programs/`; the
  generator's agent set keeps the new ten's blank lines and tests.
- Programs that need a missing library are re-measured when that library
  lands; the twin stays the same.
- The new programs' data files live in `examples/agent-data/`, not
  `examples/data/`: `scan` and `dashboard` read `examples/data/` and their
  output would change.
- ADR numbers 0478 and 0479 in this batch's block are unused.
