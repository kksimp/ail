# Program measurements, round 8: the agent corpus with the round 5–7 features

> **Non-normative design evidence,** 2026-10-20. Five new agent-tool programs
> that use what rounds 5–7 added, each with a Python twin that prints the same
> bytes ([ADR 0337](../decisions/0337-agent-corpus-round-two.md)).

## Method

Same as [round 5](program-measurements-r5.md): counts are the mean of the five
offline tokenizers (`tools/tokbench/run.sh`), the `.ail` files are the
examples without comment lines, and both sides keep their tests. Twins use the
standard library and are written as an agent would write them (`subprocess`,
`urllib`, `queue`, `concurrent.futures`, `dataclasses`). Every pair was run:
`python3 NAME.py` and `ail run examples/NAME.ail` give identical output.

| Program | Features exercised | AIL | Python | AIL − Python |
|---|---|---|---|---|
| `scan` | `fs.glob`, `proc.run` with input, `path.ext`, `strip_prefix` | 291.6 | 326.0 | −34.4 (−10.6%) |
| `tail` | `Channel`, `spawn`, `for v in ch`, `[f(x) par for x in xs]` | 318.6 | 332.4 | −13.8 (−4.2%) |
| `merge` | `impl FromJson`, `Json.set`, `as_obj`, `else` in a `for` header | 366.0 | 369.0 | −3.0 (−0.8%) |
| `digest` | `net.get`, `else err`, `net.NetError`, `env.args` | 289.4 | 357.0 | −67.6 (−18.9%) |
| `runner` | `proc.run` with a time limit, error `kind`, `if … then … else` | 291.2 | 318.6 | −27.4 (−8.6%) |
| **New five** | | **1,556.8** | **1,703.0** | **−146.2 (−8.6%)** |

Together with the five older agent programs (`logscan` 497.2 / 482.2,
`tasks` 597.2 / 675.2, `build` 430.4 / 463.0, `snapdiff` 324.2 / 309.8,
`csvreport` 408.6 / 398.6; AIL / Python), the ten-program agent corpus is
**3,814.4 AIL against 4,031.8 Python (−5.4%)**. The whole corpus under
`corpus/programs/` (all 20 programs with a Python twin) is 19,431.6 against
20,336.8 (−4.5%).

`digest` and `scan` gain most because the failure path (`else err`, `?` with
context) replaces Python's `try`/`except` clauses that name the exception
types; `merge` gains least (under 1%) because Python's dicts need no declaration while
AIL declares two structs, which buys the typed `from_json` checks the twin
lacks (a wrong type in the JSON is an error in AIL and goes unnoticed in
the dataclass twin).

## Behaviour checked

- `digest` was also run against `python3 -m http.server` on loopback
  (`http://127.0.0.1:PORT/sales.csv`); both versions print the same report
  as the offline fallback.
- `runner` uses a time limit; cancellation (lane 72) had not landed when the
  program was written, so a step that overruns is killed by `proc.run`'s
  `timeout_ms` and reported as `timed_out`.
- `scan` shells out to `git hash-object --stdin`, so it needs `git` on the
  `PATH`, as the twin does.

## Bug found

Writing `scan` hit a checker panic: assigning to a still-untyped `{}` map
with a value that reads the same map (`m[k] = (m.get(k) else []) + [x]`)
settled the local twice. Fixed in `check.rs` (`assign`), with the case
`pending_map_value_reads_it`; the program is now an ordinary diagnostic.

## Older programs

The newly available features do not shorten the five older agent programs.
Their remaining gap to Python (`logscan` +3.1%, `snapdiff` +4.6%, `csvreport`
+2.5%) is the declared structs and the escaped `\{` in JSON test strings
that [round 5](program-measurements-r5.md) already measured. The one change
found is a missing method, added in round 9 as `Set.union` ([ADR 0340](../decisions/0340-set-operations.md)).

## Where AIL still costs more than Python

Resolved in round 9 (lane 91): `Set.union` and friends (−9.6 tokens on
`snapdiff`, [ADR 0340](../decisions/0340-set-operations.md)) and `m.push(k, x)`
(−4.8 on `scan`, [ADR 0341](../decisions/0341-map-push.md)); named arguments
for `proc.run` measured +1.4 on `runner` and were not added
([ADR 0342](../decisions/0342-named-arguments-for-std-optionals-rejected.md)).
The ten agent programs are 3,800.0 tokens against Python's 4,031.8 (5.7%
under, from 5.4%).
