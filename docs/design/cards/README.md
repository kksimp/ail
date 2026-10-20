# Program cards

Output of `ail api --card` ([ADR 0310](../../decisions/0310-ail-api-card-format.md)),
checked in as examples and kept fresh by `cargo test --test api_card`
(`api.card.5`). Regenerate from the repository root:

```sh
./ail api --card tools/edit > docs/design/cards/edit.md
./ail api --card bootstrap/src/std > docs/design/cards/std.md
```
