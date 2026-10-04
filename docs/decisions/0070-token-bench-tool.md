# 0070. Token-measurement tooling lives in `tools/tokbench/`, written in Python

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; project tooling
- **Spec rules:** none

## Context

ADR 0029 defines the token benchmark method. So far, measurements were run
ad hoc from a scratch directory and can't be reproduced. Only an old Claude
tokenizer is available offline. Current Claude counts require Anthropic's
token-counting API.

## Options considered

- **Python in `tools/` (chosen):** tokenizer libraries are easiest to use
  from Python. Results become reproducible, and anyone can add current-Claude
  counts with their own API key. Con: a Python development dependency.
- **Rust:** one development language (ADR 0023), but more work for a
  measurement tool.
- **Docs only:** not reproducible.

## Decision

`tools/tokbench/` contains the measurement harness, a fetch script for the
offline tokenizer files (downloaded from PyPI packages that bundle them, not
committed), and the measurement corpus. An optional mode counts tokens with
the current Claude tokenizer through Anthropic's `messages.count_tokens` API,
using the runner's own credentials.

## Consequences

- This is a **measurement instrument**, not part of AIL's product. ADR 0009
  doesn't apply (consistent with ADR 0029).
- Syntax ADRs from now on cite corpus files and harness output.
- Tokenizer files and API results are not committed. The exact tokenizer
  versions are pinned in the fetch script.

## Rejected alternatives

- **Rust harness:** effort without benefit for an instrument.
- **Docs only:** not reproducible.
