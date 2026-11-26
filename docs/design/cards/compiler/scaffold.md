# compiler: scaffold

## scaffold
```
fn scaffold.valid(name str) bool
fn scaffold.lessons() List[str]  ## The lessons every agent session starts with (tool.new.3, ADR 1137). `ail mcp` sends the same lines as its
fn scaffold.agents_md() str  ## The agent kit (tool.new.3): at most 475 tokens, each tool with the situation it pays for.
fn scaffold.mcp_json() str  ## The MCP registration (tool.new.3): `ail mcp` on stdio.
fn scaffold.is_project() bool [fs io]  ## Whether the current directory holds an AIL project: a `.ail` file or `ail.pkg` (tool.new.5).
fn scaffold.here(agent bool) int [fs io]  ## `ail new .` (tool.new.5): writes `AGENTS.md` and `.mcp.json` into the current directory unless they exist.
scaffold.NEW_CAPS := "fs=.,fs=@tmp,env,clock,random"  ## The capabilities a new project starts with (tool.new.6): files under the project and the system temporary directories, the environment,
fn scaffold.manifest_text(name str) str  ## The `ail.pkg` of a new project (tool.new.1, tool.new.6).
pub fn scaffold.command(args List[str]) int [fs io proc]  ## The exit status of `ail new ARGS`.
```
