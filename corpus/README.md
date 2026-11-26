# AIL corpus

Public, runnable AIL programs, and the mistakes pairs, for people and models learning AIL
([ADR 1174](../docs/decisions/1174-public-corpus-layout.md)).

Licence: Apache 2.0 with the Runtime Library Exception, the [LICENSE](../LICENSE) of this repository. The programs
carry no per-file header, so a copied or learned program starts with its task, not with boilerplate.

## Layout

```
corpus/NAME/               the first set, `core`: NAME.ail, NAME.py, NAME.out
corpus/sets/SET/NAME/      every later set, the same three files
corpus/waiting.txt         `NAME reason`: programs that wait on a compiler fix (skipped, and said so)
corpus/mistakes/           the mistakes pairs: README.md and why.tsv
```

## A program

- `NAME.ail`: opens with `## ` lines that state the task, then the program with `fn main()` and at least one
  `@test`. 10 to 300 lines.
- `NAME.py`: a Python twin that prints the same output. It is the oracle.
- `NAME.out`: the expected output, written by the twin (`python3 tools/corpus/corpus.py out NAME`).
- `NAME` is lower snake case, 2 to 40 characters, and unique in the whole corpus, so a program can change set and
  keep its name. A set name has the same shape; a set holds at most 500 programs.
- Deterministic: no clock, random numbers or network; files only under `fs.make_temp_dir`; no dependence on the
  working directory; output that does not depend on map iteration order (sort the keys).
- No `std.app`, `std.gfx`, `std.ui`, `std.net` or other module that needs a screen, a device or a network.
- Plain idiomatic AIL as the language card teaches it.
- Never a task of the agent benchmark (`tools/agentbench/tasks`) or a sealed task, by name or by text.

## Checks

| What | Where it runs | Command |
|---|---|---|
| Files, names, header, `main`, `@test`, size, banned modules, duplicates, benchmark names and texts | gate suite `docs`, any host | `python3 tools/corpus/corpus.py check` |
| Each twin prints its `.out` | the author, before the pull request | `python3 tools/corpus/corpus.py check --twins` |
| Each program builds natively, prints its `.out` and passes `ail test` | gate suites `corpus` (first set) and `corpus_sets` | `./ail selftest corpus corpus_sets` |

`python3 tools/corpus/corpus.py import DIR SET` copies the program directories of `DIR` into `corpus/sets/SET`,
leaving out any that break a rule. `python3 tools/corpus/corpus.py export OUT` writes `programs.jsonl` and
`mistakes.jsonl`, one JSON object per line, for tools that train or evaluate on the corpus.
