# Outside-agent kit: do agents build and maintain programs better in AIL?

A free test that needs no API credits. A chat assistant or coding agent you already have (ChatGPT with local shell access,
Codex CLI, an IDE agent) works through the same tasks in AIL and in C, from clean unfamiliar folders. The kit counts its tries
and scores the result against hidden cases. It uses the agentbench workspaces
([ADR 0916](../../docs/decisions/0916-agent-end-to-end-benchmark.md)), so the numbers are comparable with the paid runs.

## What it measures

Per task and language: whether the hidden cases pass (**correct**), how many distinct source versions the agent built or
tested before `./test` went green (**versions to green**, 1 is best), whether the first check was clean, and minutes. The claim
under test is that the agent finishes correct software in no more tries in AIL than in C, on whole programs and on changes
to multi-file code.

Default tasks (8, each in AIL and C, so 16 sessions):

| Kind | Tasks |
|---|---|
| Build from a spec | `ledger`, `depgraph` |
| Maintain an existing multi-module codebase | `stock-bugfix`, `stock-locations`, `stock-refactor` (4 modules), `till-bugfix`, `till-returns`, `till-rename` (14 modules, about 1,100 lines of AIL and 1,570 of C) |

These tasks are in the repository, so a model that has seen the repository could know them; label results from them
"practised". For unseen tasks, put new task folders in a directory and set `AGENTBENCH_HELDOUT` to it when running `make`.

## Running it

On the Mac, from the repository root (needs `./ail` working, `cc`, `python3`):

```sh
tools/outside-test/run.sh make ~/ail-outside-test          # KIT must be outside the repository
```

This writes `KIT/work/TASK-ARM-1/` (what the agent sees), `KIT/prompts/TASK-ARM-1.txt` (what you paste), and `KIT/score/`
(hidden cases; keep the agent away from it).

For each folder in `KIT/work/`:

1. Start a **new** conversation or session. One task per session, no earlier context.
2. Give the agent access to that one folder only (not the repository, not `KIT/`). Paste the prompt from `KIT/prompts/`.
3. Let it run until it stops. Do not help, hint or answer questions beyond "keep going"; write any other message you had to
   send into `KIT/notes.txt` as `NAME: what you said`. Those count as interventions.
4. Do the AIL and C folder of the same task in the same way, in alternating order across tasks (AIL first for the first
   task, C first for the second), so tiredness and context warm-up do not favour one language.
5. Use the same model and settings for all 16 sessions and write them in `KIT/notes.txt`.

Then score, any time (it is safe to repeat):

```sh
tools/outside-test/run.sh score ~/ail-outside-test
```

It prints one row per session and one line per language, and writes `KIT/results.json`. A session never started shows `-`.

## Reading the result

- Compare `correct` first, then `mean versions to green`, then minutes. With 8 tasks per language, a difference of one
  version on the mean is noise; look for the pattern across kinds (build vs maintain).
- Report both languages' numbers, whichever way they fall. The project's rule is to publish misses
  ([benchmark page](../../docs/agent-benchmark.md)).
- Open the agent's AIL sessions' first failing builds: each one is a card, diagnostic or auto-fix candidate. Paste them into
  an issue or the lane that owns first-draft failures.
- A model that has never seen AIL will start behind in AIL. The kit tests that gap, not the language's ceiling; say so.

## What the scorer trusts

The wrappers in `work/*/.bench/bin/` log each build, test and compiler call with a checksum of `src/`; the scorer counts
versions from that log. If the agent runs the compiler by an absolute path or rewrites `./build`, the count is lower than the
truth, so check `calls` in `results.json` against what you saw. Hidden cases are scored by rebuilding the final `src/`.
