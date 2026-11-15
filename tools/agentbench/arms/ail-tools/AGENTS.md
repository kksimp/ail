# AGENTS.md

AIL project: source is `.ail`, the CLI is `ail`. Use a tool when its line applies, not as a ritual.

- `ail card` once (the core card). Then look up, never guess: `ail card TOPIC`, `ail ask std WORDS` (std and methods), `ail ask how WORDS`.
- A diagnostic `error[RULE]`: `ail check --fix FILE` applies the fixes it carries; `ail explain RULE` gives the meaning and a fix.
- Before editing a function you did not write: `ail context FILE FN --budget 800` (callees, callers, types, tests), not the whole file.
- Before changing a signature or a type: `ail ask FILE what calls FN` (also: who uses T). Never grep.
- Two candidate fixes: `ail try FILE` checks and tests each on a copy.
- A change across several functions in a program with `@test` functions: `ail task start FILE GOAL`, `ail task verify` (check, tests, what no test covers), `ail task finish` (writes `.ail/cert.json`).
- `ail task verify` runs only the check and the `@test` functions: run the project's own build and test commands as well.
- `ail fmt FILE` formats; `ail test FILE` runs the `@test` functions.
- `.mcp.json` registers `ail mcp`: the same tools as MCP calls (`ail_ask`, `ail_check`, `ail_context`, `task_start`, ...), kept warm between calls.
