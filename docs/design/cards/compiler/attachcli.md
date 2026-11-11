# compiler: attachcli

## attachcli
```
attachcli.ATTACH_USAGE := ...
struct attachcli.Running  ## A program that publishes an attach socket.
    port int
    token str
fn attachcli.tmp_dir() str [env]  ## Where programs publish their discovery files, as `std.attach.serve.token_path` has it: `TMPDIR`,
fn attachcli.running() List[attachcli.Running] [env fs io]  ## The programs whose discovery files (`attach.wire.1`) are in the temp directory, by port.
fn attachcli.field_json(key str, value str) str  ## A field's JSON: a number stays a number, `names` is a list, anything else is a string.
pub fn attachcli.command(args List[str]) int [env fs io net]  ## `ail attach ARGS`; the exit status: 0 for a reply, 1 for an error reply, 2 for a usage or connection error.
fn attachcli.pick(apps List[attachcli.Running], port int) attachcli.Running? [io]  ## The program `port` names, or the only one running; `none` after saying why (attach.cli.2).
error struct attachcli.SendError  ## Why a request failed to get a reply.
    message str
fn attachcli.send(app attachcli.Running, body str) str ! attachcli.SendError [fail io net]  ## Sends request line `body` to `app` and gives its reply line (attach.wire.3).
attachcli.MCP_PROTOCOL := "2025-03-26"
fn attachcli.jm(key str, value Json) JsonMember
fn attachcli.prop(name str, ty str, about str) JsonMember  ## A JSON Schema property of type `ty` ("string", "integer", "number" or "boolean").
fn attachcli.mcp_tool(name str, about str, props List[JsonMember], needs List[str]) Json
fn attachcli.fixed_tools() List[Json]  ## The fixed tools: the read queries and `input` (attach.wire.4, attach.input.1).
fn attachcli.schema_type(ty str) str  ## The JSON Schema type of a tool parameter of AIL type `ty` (attach.tool.1).
fn attachcli.app_tools(port int) List[Json] [env fs io net]  ## The app's own tools, from its `tools` reply (attach.tool.3); none when it can't be reached.
fn attachcli.request_for(name str, args Json) List[JsonMember]  ## The attach request for MCP tool `name` with `args`, without the token (attach.wire.4, attach.tool.4).
fn attachcli.rpc_reply(id Json, result Json) [io]
fn attachcli.rpc_failure(id Json, code int, message str) [io]
fn attachcli.text_result(text str, bad bool) Json  ## A tool result: the reply line as text, an error when it carries `error` (attach.tool.5).
fn attachcli.mcp_call(id Json, params Json, port int) [env fs io net]
fn attachcli.mcp_handle(req Json, port int) [env fs io net]  ## One message: `initialize`, `ping`, `tools/list`, `tools/call`; a message without an id gets no reply.
pub fn attachcli.serve_mcp(port int) int [env fs io net]  ## `ail attach [--port N] mcp`: serves until stdin closes (attach.tool.5).
```
