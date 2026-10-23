# garlic: attach

## attach
```
pub struct attach.Bridge  ## The window side of the bridge: `lock` serialises requests (the browser
    pub lock Mutex[int]
    pub replies Channel[str]
    pub fn attach.Bridge.new() attach.Bridge
pub fn attach.make_token() str [io proc random]  ## A token an agent must present: 24 bytes from the system's random source,
fn attach.fallback_token() str [random]
pub fn attach.authorized(req net.Request, token str) bool  ## Whether `req` carries the bearer token.
fn attach.reply(ex net.Exchange, status int, body str) [io net]
fn attach.handle(ex net.Exchange, inbox std.app.Inbox, bridge attach.Bridge, token str) [io net]
pub fn attach.serve_loop(srv net.Server, inbox std.app.Inbox, bridge attach.Bridge, token str) [io net]  ## Accepts requests until the server is shut down.
```
