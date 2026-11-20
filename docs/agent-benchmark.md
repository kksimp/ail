# Agent benchmark: AIL against C

Can an AI agent write and change working software in AIL in as few tries as in C? This page reports what
the project measured, losses included. The harness is [`tools/agentbench`](../tools/agentbench/README.md) and the rules
are [ADR 0916](decisions/0916-agent-end-to-end-benchmark.md) with its amendments.

**Short answer.** AIL programs were not shown to be less correct than C programs, and cost less to produce: in the final run
232 of 232 against 228 of 231 and 25% less per trial; in the rerun on eight new unseen tasks 78 of 80 against 79 of 80
and about 20% less. On task shapes the project had practised on, AIL needed as few tries as C. On unseen tasks it
needed more: 1.23 tries a task against 1.06 in the final run, 1.35 against 1.12 in the rerun. Both runs therefore
**missed** their pre-registered target. Almost all of AIL's extra tries were first drafts that did not compile because
the agent guessed a name or form it had never seen. Agents are less efficient in AIL than in C today. We expect that to
change as models learn AIL; see [What we expect](#what-we-expect).

## What is measured

One model, the same in every trial, gets a task in a fresh workspace: a spec, a reference card for the language, and
`./build` and `./test` commands that run a subset of the task's cases. It works until it stops. The harness then
scores the final program against every hidden case.

| Measure | Definition |
|---|---|
| Tries (primary) | Distinct versions of `src/` the agent built or tested up to the first passing `./test`, told apart by a checksum of `src/` at every logged call. 1.00 means the first draft passed. The untouched seed is not a try. |
| Correct | The final program builds and passes every hidden case (stdout, exit code and, where the task says so, stderr). All or nothing. |
| Cost | Total API cost of the trial, from the agent's own result events. |
| Call per class | "lost" when the 95% bootstrap interval of (AIL tries minus C tries) lies wholly above zero; "won" when it lies wholly below zero and AIL is not less correct; otherwise "tied". |

Rules that keep the arms equal (ADR 0916):

- Same model, prompt (apart from the language's name), task spec, cases, turn and time caps in every arm. The agent
  CLI is version 2.1.318 in E14 and later runs; the final run also pins the model by the hash of its id.
- Expected outputs come from a Python oracle, so neither language's author writes them.
- AIL gets the front of its language card (through Modules, under 24 KB; `ail card WORD` prints the rest). C gets a
  libc and POSIX reference card sized to the whole AIL card. With the spec, the final run's AIL agents read a median
  28 to 31 KB before their first write, C agents 45 to 48 KB. Cards hold signatures and pitfalls, never solutions. No
  arm has web access.
- C builds with `-Wall -Wextra -fsanitize=address,undefined`, so memory faults count as failures, as AIL's traps do.
- A trial that reads outside its workspace (the harness, a hidden case, another trial) is void and left out. A trial
  that hits an API error is invalid and rerun. Nothing else is dropped.
- AIL's compiler applies certain fixes during a build ([ADR 1130](decisions/1130-certain-fixes-applied-by-check.md)).
  A draft it repaired counts as passing in the primary measure, as the product is designed to work. A secondary figure
  counts each repair as one more try.

Task classes:

- **From scratch:** build a command-line program from a spec (`ledger`, `depgraph`, `stock`).
- **Change:** fix, extend or refactor an existing program the agent did not write (`ledger-fix`, four `stock` changes,
  three changes to `till`, a 14-module shop program).
- **Held out:** from-scratch tasks no one tuned the language, card or compiler against. Each held-out set is spent once
  its transcripts have been read.

## Runs so far

| Run | Date | Commit | Trials | Cost | Arms | Result |
|---|---|---|--:|--:|---|---|
| E13 | 2026-11-15 | `40eec283` | 480 | $41.82 | AIL plain, AIL tools, AIL core card, C, Rust | C and Rust needed fewer fix rounds and cost less; completion tied |
| E14 | 2026-11-17 | `511d5efa` | 840 | $97.53 | the same five | AIL lost from scratch, lost or tied on change tasks |
| E14 confirmation | 2026-11-17 | `1636ca45` | 108 | $12.62 | AIL plain, AIL core card, C | `ledger-fix` and `ledger` level with C; calculator gap unchanged |
| Calculator rerun | 2026-11-18 | `0384e848` | 36 | $4.91 | AIL plain, AIL core card, C | calculator gap smaller, not closed |
| Final-run pilot | 2026-11-19 | `4ecc5c8d` | 42 | $4.64 | AIL plain, C | smoke check, no call |
| **Final run** | 2026-11-19 | `978793e8` | 464 | $54.81 | AIL plain, C | **target not met** |
| **Rerun** | 2026-11-20 | `767759a3` | 160 | $22.44 | AIL plain, C | **target not met**: held out lost again |

Dates are when each run finished. E13 counted fix rounds, not tries, and is described in ADR 0916's result of 2026-11-15. Each run's review led to
compiler, diagnostic and card fixes before the next.

## E14: five arms

840 trials on commit `511d5efa`, $97.53, 19 void trials (every void trial ended correct; counting them moves no class
mean by more than 0.014).

Mean tries over all scored trials:

| Class | AIL plain | AIL tools | AIL core card | C | Rust |
|---|--:|--:|--:|--:|--:|
| From scratch (36 per arm) | 1.28 | 1.28 | 1.19 | 1.03 | 1.00 |
| Change (90 to 95 per arm) | 1.22 | 1.20 | 1.17 | 1.11 | 1.22 |
| Held out (36 per arm) | 1.42 | 1.50 | 1.44 | 1.11 | 1.00 |

Calls against C: every AIL arm lost from scratch; on change tasks AIL plain lost and the other two tied; Rust tied
everywhere. The held-out calls were computed on correct trials only, a method dropped after this run because it hid the
tools arm's 12 failed calculator trials.

Correct final programs: AIL plain 162 of 162, C 162 of 163, Rust 161 of 164, AIL core card 162 of 167, AIL tools 153 of
165. All 17 wrong AIL programs were on the held-out calculator task. The 14 programs that called `checked_rem` all
failed one case: the card said it returns `none` on overflow, but AIL defines `MIN % -1` as 0 (`expr.arith.5`). The
card now says so. The other misses, in every language, were a line with leading spaces or a keyword case.

Mean cost per from-scratch trial: AIL core card $0.082, AIL plain $0.100, Rust $0.116, AIL tools $0.124, C $0.142.

What followed: AIL plain stayed the headline arm (it never shipped a wrong program). The tools arm was dropped: no trial
in it used the agent tools, and it cost more than plain for no gain. Fixes landed for the main causes of failed AIL
versions: a function that returns `T ! str`, an empty `{}` typed by a store that reads the same map, and certain fixes
for several guessed forms. The calculator was spent as a held-out task.

## The final run

464 trials on commit `978793e8`, 2026-11-19, $54.81 (AIL $23.49, C $31.22). Two arms, AIL plain and C. One C trial was
void. Target, fixed before the run (ADR 0916, amendments of 2026-11-18 and 2026-11-19): no class lost; the pooled 95%
upper bound of the tries gap under +0.10; AIL not shown to be less correct; lower cost per trial.

| Class | Arm | Trials | Correct | First version green | Mean tries | 95% interval of AIL minus C | Call |
|---|---|--:|--:|--:|--:|--:|---|
| From scratch | AIL | 72 | 72/72 | 63/72 | 1.14 | -0.08 to +0.14 | tied |
| From scratch | C | 72 | 70/72 | 65/72 | 1.11 | | |
| Change | AIL | 80 | 80/80 | 73/80 | 1.09 | -0.06 to +0.11 | tied |
| Change | C | 79 | 79/79 | 74/79 | 1.06 | | |
| Held out | AIL | 80 | 80/80 | 63/80 | 1.23 | +0.06 to +0.27 | **lost** |
| Held out | C | 80 | 79/80 | 75/80 | 1.06 | | |

Pooled over all classes, 232 AIL and 231 C trials: mean tries 1.15 against 1.08, 95% interval of the gap +0.02 to
+0.13; correct 232 of 232 against 228 of 231; cost per trial $0.101 against $0.135.

| Target line | Result |
|---|---|
| No class lost | **no** (held out) |
| Pooled upper bound under +0.10 | **no** (+0.13) |
| Not shown to be less correct, pooled and in every class | yes |
| Lower cost per trial | yes |
| **Verdict** | **NOT MET** |

Secondary figures (not part of the verdict):

| | AIL | C |
|---|--:|--:|
| Mean tries, a build repair counted as a try | 1.36 | 1.08 |
| First drafts that passed with no repair | 150/232 | 214/231 |
| Cost per trial without reading the card (estimate) | $0.077 | $0.096 |
| Source versions that compiled and failed a visible test | 1 | 14 |

With a repair counted as a try, the pooled gap is +0.20 to +0.36: the compiler's certain fixes did a large share of the
work. When an AIL draft compiled, it was almost always right; C's misses were mostly logic or misread specs. No trial
hit a cost cap, and no AIL trial needed more than three versions.

### Why the target was missed

An independent review read every non-sealed transcript and the compiler messages of the sealed ones. 33 AIL trials did
not pass with their first version; 32 of them had a first draft that did not compile (C: 4).

| Cause | Trials |
|---|--:|
| A zero-padded integer: `n.pad_left(2, '0')`, or a helper method declared on `int` | 10 |
| Other guessed std names (`char.to_int`, `list.get`, `to_list` on a list, `BigInt` names) | 8 |
| Mixing `T` and `T?` in `if` branches, `?` placement, `! str` | 6 |
| A one-line `if` in a scripted edit; a local name that clashes with a module | 6 |
| Other (destructuring, a tuple argument) | 2 |
| Logic | 1 |

These are the misses training data fixes: names and forms the model has not seen. The part training data does not fix,
logic, is where AIL was already ahead. The review's estimates, which were not run: without the ten padding misses the
target would have been met on the line; with none of the compile misses AIL would have won every class. Agents did
keep editing after the first green test, in both arms (AIL 23 trials, C 30), mostly to remove a warned unused variable
or fix an edge case found by hand-testing; the tries measure stops at the first green test, so this did not affect the
verdict.

## Changes after the final run

[ADR 1169](decisions/1169-final-run-first-draft-misses.md), merged 2026-11-20, addresses the largest causes without
changing syntax or the standard library:

- `n.pad_left(w, c)` on an `int` is repaired for any receiver (a field, a call, a parenthesised expression), not only a
  bare name. Guessed pad names (`zfill`, `to_string_pad` and similar) get help naming `"{x:02}"`.
- A method declared on a built-in type gets help to write a free function.
- `BigInt.to_int()` is repaired to `as_int()`; `to_list()` on a list is dropped.
- `if` branches that differ only by one being optional get help to type the binding.

A free replay of the final run's failed first drafts showed that help text makes the second draft right but does not
save the first try. [ADR 1172](decisions/1172-first-draft-build-fixes.md) therefore turned the commonest forms into
certain fixes, so the first draft builds: a helper declared on a built-in type becomes a free function, a few guessed
names with one meaning are renamed, and some binding and index forms are rewritten. After it, 17 of the 29 replayed
failed drafts built, against 5 before.

Re-measuring the same tasks after fixing their misses would flatter AIL, because the fixes were written from those
tasks' transcripts. Only new sealed tasks can show whether the held-out gap is closed.

## The rerun

Design chosen by the designer on 2026-11-20, recorded in ADR 0916:

- **Stage 1, held out:** 8 new sealed tasks, written by an author who read no transcript or fix, 10 trials each, both
  arms (160 trials). This is the only part that can show the held-out loss closed, so it runs first.
- **Stage 2, practised tasks:** AIL only, `ledger`, `depgraph` and `stock` 24 trials each and the 8 change tasks 10
  each (152 trials), compared with the saved C trials of the final run. The C trials are reused only if the model, the
  agent CLI and every file the C agent reads are unchanged, checked by checksum before launch.
- **Stops:** after round 5 of stage 1 if AIL's mean tries is more than 0.20 above C's; at the end of stage 1 if the
  held-out class is lost. Cost cap $38.
- **Target:** unchanged from the final run. The four sealed tasks of the final run are left out of the verdict.
- The final run's NOT MET stays on the record. A pass here would have read "met on the rerun, after fixes, on new
  unseen tasks".

### Result

Frozen commit `767759a3`, 2026-11-20, $22.44 of the $38 cap. 8 new sealed tasks, 10 trials each, 80 trials per arm,
both arms run on the same commit.

| | AIL | C |
|---|--:|--:|
| Mean tries | 1.35 | 1.12 |
| 95% interval of AIL minus C | +0.07 to +0.38 | |
| First version green | 56/80 | 70/80 |
| Correct final program | 78/80 | 79/80 |
| Cost per trial | $0.114 | $0.144 |
| Mean tries, a build repair counted as a try | 1.65 | 1.12 |
| Cost per trial without reading the card (estimate) | $0.090 | $0.106 |

The held-out class was lost again (the interval lies wholly above zero), so the plan's stop at the end of stage 1
applied and the target was **not met**. The designer chose to stop there: the 152 AIL-only trials on the practised
tasks were not run. Correctness was level (95% interval of the difference in share -0.06 to +0.03) and AIL cost about
20% less per trial. Without the validation task, which had sat in the project's files before the run, the held-out
line reads 1.39 against 1.14, still lost.

One task's first five rounds were 0 of 5 correct in both arms, which pointed at the task, not the languages. Its
oracle and spec were corrected and those rounds run again; the first rows are kept aside, not counted, and their cost
is in the total. The fixes after the final run did not close the gap on tasks nobody had studied: they
removed the misses they were written for, and new tasks brought new guesses. This matches the caution recorded before
the run, that every run so far has surfaced names the model had not seen.

## What we expect

The results above measure a model that has seen a great deal of C and almost no AIL. The gap they show is a gap in
familiarity, not in the programs: when an AIL draft compiled it was nearly always right, AIL was not shown to be less
correct than C, and it cost less. The misses are the kind training data removes: a guessed method name, a form from
another language, a standard-library call that does not exist.

We therefore expect the number of tries in AIL to fall a long way once models have learned AIL from public code, and
the gap to C to close. That is an expectation, not a measured result. The project will publish a corpus of correct AIL
programs and of common mistakes paired with their fixes, and will repeat this benchmark with new sealed tasks as
models change. Until then, the compiler's certain fixes, diagnostics and the language card carry the agent past most
misses, at the cost of extra tries.

## Limits

- One model, the same in every run. A model that has seen more AIL could do better; another model could do worse.
- The tasks are small command-line programs, 4 to 12 visible cases each. They test first drafts and small changes, not
  long projects.
- The practised classes are tasks the project studied and fixed against. Only the held-out class predicts unseen work.
- A model's prior knowledge of C is part of what is measured; the benchmark does not try to cancel it.
- Sealed task names and transcripts are not published, so the tasks cannot leak into later runs.
