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
A card that meets 2,000 needs a split per module, which is not done yet.
