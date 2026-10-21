# 0751. `fs.copy` copies a directory tree

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20); the integrator approved the change on PR 294
- **Topic:** 17 — Standard library
- **Spec rules:** std.fs.12 ([17-std](../spec/17-std.md))
- **Amends:** [ADR 0281](0281-std-fs-walk-glob-metadata.md) (a directory `from` failed with `is_dir`)

## Context

Lane 192's twin t23 (back up a directory): Python `shutil.copytree` is one
call; in AIL a loop over `fs.walk` needs the parent directories made as well,
and agents got it wrong (`copy` into a missing directory fails `not_found`).

## Decision

`fs.copy(from, to)` with a directory `from` copies the tree: `to` and its
parents are made, files there are replaced, other files kept, entries copied
in name order. Links are copied as the files they point to; a link to a
directory, copying a directory into itself, and a failure part-way end with
an `IoError` naming the path that failed (kind `other`, or the file system's);
files copied before the failure stay. A file at `to` fails with `exists`.

## Token measurement

Twin t23: loop over `fs.walk` 27.4 tokens, `fs.copy("src", "backup")?` (`agent-r4/t23b_copy_tree_std.ail`) **15.0 (-45%)**; Python 14.4. Language card unchanged (the quick reference gains four words).

## Rejected alternatives

- **A new `fs.copy_tree`.** One more name to learn for what `copy` was
  refusing; Python's `shutil.copy` and `copytree` split is a known source of
  mistakes.
- **Undo on failure.** Needs a staging directory; `rename` plus `copy` cover
  atomic needs.
- **Following directory links.** Cycles are possible and cheap to hit.
