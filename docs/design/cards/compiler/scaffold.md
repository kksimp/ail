# compiler: scaffold

## scaffold
```
fn scaffold.valid(name str) bool
fn scaffold.agents_md() str  ## The agent kit (tool.new.3): at most 350 tokens, the loop and the lookups an agent needs.
fn scaffold.mcp_json() str  ## The MCP registration (tool.new.3): `ail mcp` on stdio.
pub fn scaffold.command(args List[str]) int [fs io]  ## The exit status of `ail new ARGS`.
```
