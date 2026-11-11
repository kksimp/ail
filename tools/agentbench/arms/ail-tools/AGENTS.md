# Working in this AIL project

The language card is `DOCS/language-card.md`. Read it once. Look up a std function with `ail ask std WORDS`
(or `ail card std.MODULE`) before you write it from memory, and a card section with `ail card TOPIC`.

Every change is one loop:

1. `ail task start FILE GOAL` opens a task. `ail context FILE FN` prints one function with its callees, callers,
   types and tests, so you need not read whole files.
2. Edit with `ail edit FILE --test < HUNKS`: one call edits, checks and runs the tests. Or edit in place.
3. `ail check --fix FILE` applies the repairs a diagnostic already carries.
4. `ail task verify` runs the check, the tests and the review of your change; `ail task finish` closes the task and
   writes the change certificate, `.ail/cert.json`.

Rules:
- Every diagnostic carries a rule ID. Run `ail explain RULE` for its meaning and a fix before you guess.
- With two candidate fixes, `ail try FILE` checks and tests each on a copy and gives one verdict per set.
- Never grep to find code: `ail ask FILE what calls f`, `ail api --text FILE` lists declarations.
- The same tools are an MCP server (`ail mcp`, registered in `.mcp.json`).
- `./build` and `./test` are this benchmark's own commands; use them as well.
