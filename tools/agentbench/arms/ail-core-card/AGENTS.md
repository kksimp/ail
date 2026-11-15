# Working in this AIL project

The core card is `DOCS/language-card-core.md` (what `ail card` prints). Read it once. Use a tool when its line applies, not as a ritual.

- Look up, never guess: a card section with `ail card TOPIC`, a std function with `ail ask std WORDS` (or `ail card std.MODULE`), a how-to with `ail ask how WORDS`.
- A diagnostic `error[RULE]`: `ail check --fix FILE` applies the fixes it carries; `ail explain RULE` gives the meaning and a fix.
- Before editing a function you did not write: `ail context FILE FN --budget 800` (callees, callers, types, tests), not the whole file.
- Before changing a signature or a type: `ail ask FILE what calls FN`; `ail api --text FILE` lists declarations. Never grep.
- Two candidate fixes: `ail try FILE` checks and tests each on a copy.
- A change across several functions in a program with `@test` functions: `ail task start FILE GOAL`, `ail task verify`, `ail task finish` (writes `.ail/cert.json`).
- `ail task verify` runs only the check and the `@test` functions: `./build` and `./test` are this benchmark's own commands; use them as well.
- `.mcp.json` registers `ail mcp`: the same tools as MCP calls (`ail_ask`, `ail_check`, `ail_context`, `task_start`, ...), kept warm between calls.
