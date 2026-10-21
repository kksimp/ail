# 0720. `ail api --card --index` lists a package's modules

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 227)
- **Topic:** Tools (lane 227)
- **Spec rules:** api.card.7 (new)
- **Amends:** none

## Context

ADR 0719 gives an agent one module at a time. To choose a module, it first needs
the list of modules. The package card lists the same names, but only inside a
card that a budget trims, so at a small budget the list is cut.

## Options considered

1. **Use the package card at its names-only level.** The names are there, but the
   level also drops the entry, tests and manifest, and a budget that drops the
   names also drops the count. Rejected: not a stable index.
2. **`--index`: one line per module, `NAME  N items`, under a package heading.**
   Not trimmed, and small: each package here is well under 300 tokens. Chosen.

## Decision

1. `ail api --card --index PATH` prints `# PACKAGE` and one `NAME  N items`
   line per module, sorted as the card sorts them (root first). `--json` gives
   `api`, `card` and `modules` with `module` and `count`.
2. `--index` takes no `--module` and no budget; combining it with `--module` is
   exit code 2.

## Measured sizes

The largest index, `compiler`, has 9 modules and is 150 bytes, well under 300
tokens; the rest are shorter.

## Consequences

- An agent's first read of a package is the index (under 300 tokens), then one
  module card.
