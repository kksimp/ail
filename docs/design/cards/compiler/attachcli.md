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
```
