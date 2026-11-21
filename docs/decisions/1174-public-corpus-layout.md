# 1174. The public corpus: sets, one licence line, mistakes pairs and one checker

- **Status:** Accepted
- **Date:** 2026-11-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the licence is the designer's call and stays that of the repository until they change it
- **Topic:** 28 — Testing strategy
- **Spec rules:** none (no language change)

## Context

`corpus/` holds 300 programs, each `NAME.ail` with a Python twin and an expected output, built and run by the gate
suite `corpus`. The goal after 0.2 is at least 1,000 published programs and the mistakes corpus published as wrong
and right pairs, with one source that also feeds the model-training project, which has written about 2,200 more
programs in the same three-file form. The frozen formatter goldens name the 300 by path, and `checkbench` reads them.

## Decision

- **Sets.** The 300 stay where they are and are the set `core`. Every later set is `corpus/sets/SET/NAME/` with the
  same three files. A set holds at most 500 programs. A program name is unique in the whole corpus.
- **One format, written once.** [corpus/README.md](../../corpus/README.md) is the format: the three files, the `##`
  task header, `fn main`, an `@test`, 10 to 300 lines, deterministic, no module that needs a screen, device or
  network. A program lives in one repository only; the training project reads this tree and keeps no copy.
- **Licence line.** One line in `corpus/README.md` (the repository's Apache 2.0 with the Runtime Library Exception)
  and no per-file header, which a model would learn to write.
- **Never a benchmark or sealed task.** The checker refuses a program whose name or text is a task of
  `tools/agentbench`, and any name in a `--deny` file (the sealed names stay on the designer's Mac).
- **Mistakes pairs.** The pairs stay in `tests/mistakes/` (`NAME.ail` wrong, `NAME.fixed` right, `NAME.rules`), where
  the suite `mistakes` already proves the wrong half fails with its rules and the right half checks. The published
  form adds `corpus/mistakes/why.tsv`, one line of explanation per pair, and `corpus/mistakes/README.md`.
- **One checker.** `tools/corpus/corpus.py check` reads every file and is in the gate suite `docs`; `--twins` runs
  the twins; `import` copies a set in, leaving out what breaks a rule; `export` writes `programs.jsonl` and
  `mistakes.jsonl`. It replaces `tools/check-corpus-py.py`.
- **Gate.** `corpus` builds and runs the set `core` as before. The new suite `corpus_sets` does the same for every
  other set and takes shards (`corpus_sets@K/N`). The integrator raises its shard count as the sets grow.
- **Easy to move.** Everything but the checker and `tests/mistakes/` is under `corpus/`, and the checker takes
  `--root`, so the tree can become its own repository by moving one directory and one tool.

## Consequences

- No frozen golden changes and no program moves.
- `corpus_sets` costs gate time in step with the sets: about ten times the suite `corpus` at 3,000 programs.
- `why.tsv` starts with three lines; `check --why-complete` goes into the gate when every pair has one.

## Rejected alternatives

- **Move the 300 into `corpus/sets/core/`.** Uniform, but it refreezes 600 formatter golden rows and changes
  `checkbench` inputs for no gain to a reader.
- **One flat directory.** GitHub lists only the first 1,000 entries of a directory, and one suite would hold every
  program.
- **A committed JSONL or index of the programs.** A generated file that every corpus pull request changes, so two
  lanes always conflict. `export` writes it on demand.
- **Copies of the pairs under `corpus/mistakes/`.** 542 files held twice, and a second place to keep right.
- **A licence header in each file.** Models learn headers as part of the language.
- **A separate repository now.** The corpus would not be gated with the compiler that must keep building it.
- **Sets only in a release gate.** A compiler change that breaks a published program would land unseen.
