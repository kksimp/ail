# Working in this AIL project

The language card is `DOCS/language-card.md`. Read it once, then work in this loop:

1. `ail task start FILE GOAL` opens a task. `ail context FILE FN` prints one function with its callees, callers, types
   and tests, so you need not read whole files.
2. Edit with `ail edit FILE --test < HUNKS`: one call edits, checks and runs the tests. Or edit in place.
3. `ail task verify` runs the check, the tests and the review of your change; `ail task finish` closes the task.

Rules:
- Every diagnostic carries a rule ID. Run `ail explain RULE` for its meaning and a fix before you guess.
- `ail fix FILE` applies the repairs a diagnostic already carries.
- Never grep to find code: `ail ask FILE what calls f`, `ail api --text FILE` lists declarations.
- The same tools are an MCP server (`ail mcp`, registered in `.mcp.json`).
- `./build` and `./test` are this benchmark's own commands; use them as well.
