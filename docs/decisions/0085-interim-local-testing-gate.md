# 0085. Interim testing gate: local runs on Linux (AI) and macOS (designer); no hosted CI

- **Status:** Accepted (interim)
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; project process
- **Amends:** [ADR 0027](0027-ci-gating-all-targets.md) (five-target CI gate) and [ADR 0028](0028-git-workflow.md) (code changes gated by CI) until revisited

## Context

Code is approaching. ADR 0027 requires all five targets to gate every merge
through CI. The repository is private, and the designer chose not to use
GitHub-hosted CI or make the repository public:

> "For now we loosen up, I can test on mac and windows, you can test on
> linux, but we will allow just macos and linux passing for now. I don't
> want to use any github testing or make the repo public."

## Options considered

- **All five targets via GitHub Actions on free minutes:** follows ADR 0027,
  but the designer declined hosted CI.
- **Fewer targets on PRs, all five nightly:** still hosted CI.
- **Make the repository public:** declined, because it is earlier than ADR 0011's "once compelling".
- **Local testing on Linux and macOS (chosen).**

## Decision

Until this ADR is revisited:

1. **No GitHub-hosted CI.** The repository stays private.
2. **Gate:** a code change merges only when the test suite passes on
   - **Linux x86-64**, run by the AI in its environment, and
   - **macOS ARM64**, run by the designer with a single script (ADR 0071).
3. **Windows** is tested by the designer when convenient. It is **not**
   gating for now.
4. **Linux ARM64 and Windows ARM64** are not gated for now.
5. The two gating targets still cover **both ISAs** (x86-64 and ARM64), so
   ADR 0008's lockstep principle stays meaningful.

## Consequences

- When code exists, the AI provides one script the designer runs to test a
  pending change on macOS without disturbing their checkout (e.g. testing a
  branch in a temporary worktree), and reports results back.
- Windows-specific and ARM Linux breakage can go unnoticed for longer. Risk
  accepted for now.
- **Revisit triggers:** making the repository public, adding contributors,
  a Windows regression slipping through, or the start of native code generation
  (Stage 4), where per-target correctness matters most.

## Rejected alternatives

- **Hosted CI, or a public repository:** declined by the designer.
