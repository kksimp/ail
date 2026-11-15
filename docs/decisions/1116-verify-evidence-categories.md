# 1116. `ail verify` names its evidence precisely; certificate version 2

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); from the external review of 2026-11-15 (item 4)
- **Topic:** 22 — Tooling
- **Spec rules:** verify.run.1-6, verify.cert.1-4
- **Builds on:** [0909](0909-ail-verify-diff.md), [1052](1052-change-certificate.md), [1055](1055-contracts-proved-by-interval-facts.md)

## Context

`ail verify --diff` printed *proved* for "`ail check` accepts the program", which a reader takes for a proof of
behaviour. `ail prove` (ADR 1055) discharges contracts, but its answer was in neither the report nor the
certificate. A failed `ail test` run, a program that did not check and a revision that could not be unpacked all
read as "nothing failed". The certificate recorded the base commit, the changed files and the compiler version, but
not the host, the run-time settings that change results, or inputs that git does not track, and the gate tested
only that a changed source file invalidates it.

## Options considered

### Option A: split the evidence into named categories; certificate 2 records its environment and inputs (chosen)
- **Pros:** each word means one thing; an agent reads a category and knows what stands behind it; a certificate
  differs for every input that can change the result.
- **Cons:** a breaking change to the `verify` JSON and the certificate (version 1 becomes 2).

### Option B: keep `proved` and add a note
- **Cons:** the wrong word stays the first thing read.

### Option C: run a solver for contracts
- **Cons:** out of scope; ADR 1055 chose no solver.

## Decision

The report has five parts: *checked* (static check only, never called proved), *contracts* (what `ail prove`
discharges, with the unproved checks of changed declarations, or `unavailable` with a reason), *tested* (a
`state` of `passed`, `failed`, `none-ran`, `could-not-run` or `not-tested`, plus the names and whether the base
revision could be tested), *unverified* and *failures*. `--json` is `"verify":2`.

The certificate is `"cert":2`. It adds `env` (host and the `AIL_THREADS`, `AIL_PARALLEL`, `AIL_MAX_MEMORY_MB`,
`AIL_TIMEOUT_MS`, `AIL_ALLOW`, `AIL_OPT_OFF` settings) and `inputs` (a digest of every `.ail` and `.h` file under
the program's root, its `ail.pkg` and its `ail.lock`, tracked or not). `--check` compares them with the other
members; a version 1 certificate is refused and must be made again. The `cert` gate suite now changes a source
file, a test, `ail.lock`, the recorded compiler version, the base and two settings in turn and requires the right
member to differ, and requires that committing the change does not invalidate the certificate.

## Rationale

AI understanding: a category that means one thing. Honesty: no claim that a check is a proof. A check made on
another host differs in `env` only, which says what changed and is not hidden.

## Consequences

Consumers of `verify --json` and of certificates read `checked`, not `proved`. A second machine's `--check` reports
`differs: env`; the other members still reproduce. Gitignored files outside the `.ail`, `.h`, `ail.pkg` and
`ail.lock` set are not inputs. Signing stays out (ADR 1052 Option B).

## Rejected alternatives

- **Options B and C:** see above.
- **Comparing `env` leniently:** a setting such as `AIL_THREADS` can change what an auto-parallel program does, so
  it is compared.
