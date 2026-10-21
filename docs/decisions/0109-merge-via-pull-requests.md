# 0109. Code branches merge through GitHub pull requests, and GitHub deletes them on merge

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process
- **Amends:** [ADR 0028](0028-git-workflow.md) (merge mechanism); works with the interim gate of [ADR 0085](0085-interim-local-testing-gate.md)

## Context

The designer wants a merged branch deleted automatically when it's merged:

> "It should auto delete when you merge."

The AI's environment can push branches but can't delete remote branches.
A GitHub repository setting can delete a pull request's branch automatically
when the pull request is merged.

## Options considered

- **Merge through pull requests, with GitHub's "Automatically delete head
  branches" setting (chosen):** the branch disappears the moment it merges,
  and each merge leaves a readable record. Con: a one-time settings change,
  and every change appears as a pull request.
- **Direct merges, cleaned up by `tools/verify.sh`:** nothing to change, but
  merged branches linger until the next verify run.

## Decision

1. Code changes still use a short-lived branch, gated by the ADR 0085 local
   runs (Linux by the AI, macOS by the designer with `tools/verify.sh`).
2. Once both pass, the AI opens a pull request for the branch and merges it
   through GitHub. There's no GitHub-hosted testing (ADR 0085), and the
   repository stays private.
3. The repository setting **Settings → General → Pull Requests →
   "Automatically delete head branches"** is on. GitHub deletes the branch on
   merge.
4. Safety net: `tools/verify.sh` also deletes any remote branch already merged
   into `main` (`tools/prune-merged.sh`).

## Consequences

- Documentation-only changes still go directly to `main` (ADR 0028).
- Pull request descriptions summarise the change, the spec rules affected, and
  the gate results.

## Rejected alternatives

- Direct merges, with `tools/verify.sh` deleting merged branches: nothing to change, but merged branches linger until the next verify run. Rejected in favour of GitHub deleting the head branch on merge, which keeps the merge record readable. The verify script stays as the safety net.
