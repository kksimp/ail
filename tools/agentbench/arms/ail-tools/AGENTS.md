# AGENTS.md

AIL project: source is `.ail`, the CLI is `ail`.

- Load the core card once: `ail card`. More: `ail card TOPIC`, `ail ask std WORDS` (std and methods), `ail ask how WORDS`.
- Every change is one loop:
  1. `ail task start src/main.ail GOAL`
  2. `ail context src/main.ail FN --budget 800` before you edit FN
  3. edit with `ail edit` or your editor
  4. `ail check --fix src/main.ail`
  5. `ail task verify`
  6. `ail task finish` (writes the certificate `.ail/cert.json`)
- Two candidate fixes: `ail try src/main.ail` gives one verdict per set.
- For any diagnostic, run `ail explain RULE`.
- Never grep. Use `ail ask src/main.ail what calls FN` (also: who uses T).
- `ail fmt FILE.ail` formats; `ail test FILE.ail` runs the `@test` functions.
- `.mcp.json` registers `ail mcp` on stdio; the same tools are MCP calls.
