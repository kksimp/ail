# Program cards

Output of `ail api --card` ([ADR 0310](../../decisions/0310-ail-api-card-format.md)),
checked in as examples and kept fresh by the `api_card` gate suite
(`api.card.5`). Regenerate from the repository root:

```sh
./ail api --card compiler > docs/design/cards/compiler.md
./ail api --card std > docs/design/cards/std.md
```

## Cards for the tools written in AIL (lanes 212, 222)

Each card is `ail api --card` at the default 4,000-token budget (ADR 0311). Sizes
are cl100k tokens from `tools/tokbench/run.sh` (ADR 0703).

| Card | Source | cl100k tokens at 4,000 | For |
|---|---|---:|---|
| [std.md](std.md) | `std` | 786 | The standard library |
| [compiler.md](compiler.md) | `compiler` | 2,661 | The native compiler |

`tools/perfbench` and `tools/self-compile` have no `.ail` files, so they have no
card. The `api_card` gate suite checks that every card here matches a fresh run.

At a 2,000-token budget, garlic (131), garlic-js (96), pdf (109), compiler (116),
edit (90) and std (786) all drop to names only; the signatures do not fit. ADR 0703
added a level that keeps each public signature and its doc's first sentence. It is
chosen when the budget leaves room for it: garlic reaches it at 8,000 tokens (5,750).
A card that meets 2,000 needs a split per module: see below.

## Per-module cards (lane 227, ADRs 0719 and 0720)

An agent reads a package's index, then the one module it needs:

```sh
./ail api --card --index std                      # one line per module
./ail api --card --budget 2000 --module app std   # one module
```

Each package has an `index.md` and one file per module under `<package>/`
(`root.md` is the package's own module). They are checked by
the `api_card` gate suite (`api.card.6`, `api.card.7`), at a 2,000-token
budget. Regenerate them with this loop from the repository root:

```sh
for d in compiler:compiler std:std; do
  name=${d%%:*}; dir=${d#*:}; mkdir -p docs/design/cards/$name
  ./ail api --card --index $dir > docs/design/cards/$name/index.md
  ./ail api --card --index $dir | tail -n +2 | awk '{print $1}' | while read m; do
    f=$m; [ "$m" = "(root)" ] && f=root
    ./ail api --card --budget 2000 --module "$m" $dir > docs/design/cards/$name/$f.md
  done
done
```

Sizes are `tools/tokbench` means (the five-tokenizer average). Every module card is
under 2,000; the largest are `compiler/a64.md` (1,779) and `compiler/x64.md` (1,779).

Garlic, one module at a time: the full package card at 8,000 tokens is 6,310
(mean). The index (55) plus `garlic/html.md` (1,160) is 1,215, about 81% less,
for a task that needs one module. Each further module costs its card.

