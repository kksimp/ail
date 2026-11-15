# EN agent benchmark: RESULT IN ONE LINE

Run DATE, tree COMMIT, agent CLI VERSION, model MODEL. N trials, $SPENT of a $CAP cap. Plan `plans/PLAN.txt`.
How the run works and what it counts: [ADR 0916](../../docs/decisions/0916-agent-end-to-end-benchmark.md).

## Where AIL stands

Models have seen far more C than AIL. Until they have seen AIL, an agent writing AIL needs more iterations and more
tokens than one writing C, and this run says where that holds. We believe adaptation (models that have seen AIL, and the
card written for them) together with AIL's tooling (diagnostics that print the corrected line, `ail ask`, `ail card`)
can close the gap. Each run measures how far it has closed. This report makes no speed claims.

## Primary measure: iterations to working code

Fewer is better. Per arm and task class: trials correct on the hidden cases; among them, the mean source versions
checked up to the first passing `./test` (versions to green) and the mean versions that failed a build, check or test
(fix rounds). An AIL arm wins a class when its versions to green is lower than C's and its success is no lower. The
headline AIL arm is `ail-core-card`; every arm is shown. Held-out tasks are reported as their own class.

PASTE the first table of `agentbench.py report RESULTS --plan PLAN`.

STATE, per class, whether the headline arm won, tied (within one trial's difference) or lost, against C and Rust.

## Secondary: cost, tokens and what each trial read

PASTE the per-task table. Note the median DOCS KB received per arm: under the `reference` docs policy an arm may skip
its card, and the cost comparison depends on it.

## What changed since the last run

LIST the language, card and harness changes between the two trees, with their ADRs.

## Validity

- Void trials (read the harness, a hidden case or another trial), from `agentbench.py audit`: LIST, or "none".
- Invalid trials (API errors, rerun): LIST, or "none".
- Trials stopped by a cap: N.
- Limits: one model; the tasks are small command-line programs; per-cell differences under about 30% are within noise
  at 12 trials.
