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
pub struct attach.Job
    pub id int
    pub due int  ## The second, on the window's timer, when it runs.
    pub tool str  ## The tool's name and its arguments as JSON text.
    pub args str
    pub fn attach.Job.line(self, now int) str
    pub fn attach.Job.request(self) str  ## The JSON-RPC request that runs the job.
fn attach.quote(s str) str
pub struct attach.Watch
    pub id int
    pub url str  ## The page to fetch each time, or "" for the live tab.
    pub contains str  ## Text to look for; "" watches the whole page text.
    pub every int  ## Seconds between looks.
    pub due int  ## The second of the next look.
    pub last str  ## What the last look saw.
    fn attach.Watch.state(self, text str) str  ## What a look at page text `text` notes for this watch.
    pub fn attach.Watch.start(id int, url str, contains str, every int, now int, text str) attach.Watch
    pub fn attach.Watch.look(mut self, text str, now int) str [mutates]  ## Looks at `text` at second `now`; the change it sees, or "". A look before
    pub fn attach.Watch.line(self) str
```
