# 0281. `std.fs` gets `walk`, `glob`, `metadata`, `rename` and `copy`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard modules; AI-first operation of the computer
- **Spec rules:** std.fs.8–12

## Context

Agents operating a computer list and search directory trees, check sizes and
times, and move files. `std.fs` (ADR 0157) had `list_dir`, `make_dir` and
`remove` only, so a program that wanted "every `.ail` file under `src`"
wrote a recursive function. The lane brief asked for `list`, `walk`,
`glob`, `metadata`, `mkdir_all`, `remove`, `rename` and `copy`.

## Options considered

### Option A — add `walk`, `glob`, `metadata`, `rename`, `copy`; keep the existing names (chosen)
- **Pros:** one name per operation: `list_dir` and `make_dir` (which already
  creates parents) stay what they are; `walk` returns files only, sorted,
  as paths ready to pass back to `fs`; `glob` covers the common "find by
  pattern" in one call; nothing new fails silently.
- **Cons:** the brief's `list` and `mkdir_all` spellings aren't there.

### Option B — also add `list` and `mkdir_all` as aliases
- **Cons:** two names for one operation is ambiguity an AI must resolve on
  every read (ADR 0076 criterion 2) and costs card tokens.

### Option C — a lazy directory iterator (`fs.entries(dir)` yielding entries)
- **Pros:** constant memory on huge trees.
- **Cons:** needs iterator types the bootstrap doesn't have; agents' trees are
  small; a sorted list is deterministic and easy to print and diff.

## Decision

Add `walk(dir) List[str] ! IoError` (files, any depth, sorted, symlinked
directories not followed), `glob(pattern) List[str]` (`*`, `?`, `**`;
hidden names only when asked for; never fails), `metadata(path) Metadata !
IoError` (`size`, `modified_ms`, `is_dir`), `rename(from, to)` and
`copy(from, to)` (files only). All have the effects `fs` and `io`.

## Rationale

Criterion 1 (tokens): `fs.glob("src/**/*.ail")` replaces a ten-line
recursive walker. Criterion 2: sorted, total results and failures carrying
the path make outputs reproducible and errors self-explaining. `glob` is
infallible because a pattern that matches nothing is an answer, not an error;
`walk` on a missing directory is an error because the caller named it.
`modified_ms` matches `time.now_ms`, so ages are one subtraction.

## Consequences

- Copying or removing a tree is still a loop over `walk`; a recursive
  `remove_all` stays out until a capability-scoped design for destructive
  bulk operations exists.
- No `[...]` character classes in `glob` yet.

## Rejected alternatives

- Aliases `list` and `mkdir_all`: two names for one thing (Option B).
- A lazy iterator: needs iterator types; unordered output (Option C).
- `walk` listing directories too: callers then filter with `metadata`;
  files are what agents process.
