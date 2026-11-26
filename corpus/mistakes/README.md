# Mistakes pairs

Wrong and right pairs of small AIL programs: each wrong program holds one mistake a person or a model coming from
another language makes, and the right program is the same code with only that mistake repaired.

The files are in [tests/mistakes](../../tests/mistakes), where the gate suite `mistakes` checks them:

- `NAME.ail`: the wrong program. `ail check` refuses it.
- `NAME.rules`: the rule IDs `ail check` reports for it, one per line (`ail explain RULE` describes each).
- `NAME.fixed`: the right program. It is what `ail fix` makes of the wrong one, and it passes `ail check`.
- [why.tsv](why.tsv): `NAME`, a tab, and one line that says what is wrong and what to write instead.

A program with no `NAME.fixed` has no certain repair and is not a pair. Do not learn from a `NAME.ail` alone: it is
wrong on purpose.

`python3 tools/corpus/corpus.py export OUT` writes the pairs to `OUT/mistakes.jsonl`, one object per line with
`name`, `rules`, `why`, `wrong` and `right`.
