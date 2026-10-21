# Program cards

Output of `ail api --card` ([ADR 0310](../../decisions/0310-ail-api-card-format.md)),
checked in as examples and kept fresh by `cargo test --test api_card`
(`api.card.5`). Regenerate from the repository root:

```sh
./ail api --card tools/edit > docs/design/cards/edit.md
./ail api --card bootstrap/src/std > docs/design/cards/std.md
```

## Cards for the tools written in AIL (lanes 212, 222)

Each card is `ail api --card` at the default 4,000-token budget (ADR 0311). Sizes
are cl100k tokens from `tools/tokbench/run.sh` (ADR 0703).

| Card | Source | cl100k tokens at 4,000 | For |
|---|---|---:|---|
| [edit.md](edit.md) | `tools/edit` | 2,272 | Editing the code editor |
| [std.md](std.md) | `bootstrap/src/std` | 786 | The standard library |
| [garlic.md](garlic.md) | `tools/garlic` | 3,036 | The text browser's modules |
| [garlic-js.md](garlic-js.md) | `tools/garlic/js` | 1,803 | Garlic's JavaScript engine |
| [pdf.md](pdf.md) | `tools/pdf` | 109 (names only) | The PDF viewer |
| [compiler.md](compiler.md) | `compiler` | 2,661 | The native compiler |

`tools/perfbench` and `tools/self-compile` have no `.ail` files, so they have no
card. `bootstrap/tests/api_card.rs` checks that every card here matches a fresh run.

At a 2,000-token budget, garlic (131), garlic-js (96), pdf (109), compiler (116),
edit (90) and std (786) all drop to names only; the signatures do not fit. ADR 0703
added a level that keeps each public signature and its doc's first sentence. It is
chosen when the budget leaves room for it: garlic reaches it at 8,000 tokens (5,750).
A card that meets 2,000 needs a split per module: see below.

## Per-module cards (lane 227, ADRs 0719 and 0720)

An agent reads a package's index, then the one module it needs:

```sh
./ail api --card --index tools/garlic                       # one line per module
./ail api --card --budget 2000 --module html tools/garlic   # one module
```

Each package has an `index.md` and one file per module under `<package>/`
(`root.md` is the package's own module). They are checked by
`cargo test --test api_card` (`api.card.6`, `api.card.7`), at a 2,000-token
budget. Regenerate them with this loop from the repository root:

```sh
for d in garlic:tools/garlic garlic-js:tools/garlic/js pdf:tools/pdf compiler:compiler edit:tools/edit std:bootstrap/src/std; do
  name=${d%%:*}; dir=${d#*:}; mkdir -p docs/design/cards/$name
  ./ail api --card --index $dir > docs/design/cards/$name/index.md
  ./ail api --card --index $dir | tail -n +2 | awk '{print $1}' | while read m; do
    f=$m; [ "$m" = "(root)" ] && f=root
    ./ail api --card --budget 2000 --module "$m" $dir > docs/design/cards/$name/$f.md
  done
done
```

Sizes are `tools/tokbench` means (the five-tokenizer average). Every module card is
under 2,000; the largest are `garlic/root.md` (1,830), `compiler/a64.md` (1,779)
and `compiler/x64.md` (1,779). `tools/edit`'s root module (149 items) is the one
exception that cannot fit its signatures: its card gives the count only (84).

Garlic, one module at a time: the full package card at 8,000 tokens is 6,310
(mean). The index (55) plus `garlic/html.md` (1,160) is 1,215, about 81% less,
for a task that needs one module. Each further module costs its card.

