# compiler: scaffold

## scaffold
```
fn scaffold.valid(name str) bool
fn scaffold.agents_md() str  ## The agent kit (tool.new.3): at most 350 tokens, the loop and the lookups an agent needs.
fn scaffold.mcp_json() str  ## The MCP registration (tool.new.3): `ail mcp` on stdio.
fn scaffold.is_project() bool [fs io]  ## Whether the current directory holds an AIL project: a `.ail` file or `ail.pkg` (tool.new.5).
fn scaffold.here(agent bool) int [fs io]  ## `ail new .` (tool.new.5): writes `AGENTS.md` and `.mcp.json` into the current directory unless they exist.
pub fn scaffold.command(args List[str]) int [fs io]  ## The exit status of `ail new ARGS`.
```
