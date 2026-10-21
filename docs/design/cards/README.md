# Program cards

Output of `ail api --card` ([ADR 0310](../../decisions/0310-ail-api-card-format.md)),
checked in as examples and kept fresh by `cargo test --test api_card`
(`api.card.5`). Regenerate from the repository root:

```sh
./ail api --card tools/edit > docs/design/cards/edit.md
./ail api --card bootstrap/src/std > docs/design/cards/std.md
```

## Cards for the tools written in AIL (lane 212)

Each card is `ail api --card` at the default 4,000-token budget. Sizes are the
card's own estimate (ADR 0311) where it prints one, otherwise a characters-over-four guess.

| Card | Source | Size | For |
|---|---|---|---|
| [edit.md](edit.md) | `tools/edit` | about 1,800 tokens (chars / 4) | Editing the code editor |
| [std.md](std.md) | `bootstrap/src/std` | about 700 tokens (chars / 4) | The standard library |
| [garlic.md](garlic.md) | `tools/garlic` | between 3,000 and 4,000 tokens (over the 2,000 target) | The text browser's modules |
| [garlic-js.md](garlic-js.md) | `tools/garlic/js` | between 2,000 and 4,000 tokens (over the 2,000 target) | Garlic's JavaScript engine |
| [pdf.md](pdf.md) | `tools/pdf` | names only, about 90 tokens; the full card is over 4,000 | The PDF viewer (see the note) |
| [compiler.md](compiler.md) | `compiler` | about 3,100 tokens (the card's estimate; the 4,000 allowance) | The native compiler |

`tools/perfbench` and `tools/self-compile` have no `.ail` files, so they have no
card. `bootstrap/tests/api_card.rs` checks that every card here matches a fresh run.

Note: at a 2,000-token budget the trimmer drops every signature and keeps names
only, so `pdf.md` is names only at the default too. A middle trimming level is
needed before the tool cards can meet the 2,000-token target.
