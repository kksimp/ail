# garlic: jsdom

## jsdom
```
pub struct jsdom.Outcome  ## What running a page's scripts left for the browser to do.
    pub url str  ## A navigation the page asked for (the first one), or "".
    pub method str
    pub body str
    pub console List[str]  ## Console lines and uncaught errors, in order.
    pub missing List[str]  ## External scripts the page wanted that weren't in `scripts`.
    pub changes int  ## How many changes were replayed.
    pub cookies List[str]  ## What scripts assigned to `document.cookie`, in order.
    pub asks List[jsdom.Ask]  ## `fetch` and XMLHttpRequest requests that had no reply yet.
pub struct jsdom.Ask  ## A request from `fetch` or XMLHttpRequest, numbered `id`: `packed` is the
    pub id int
    pub method str
    pub url str
    pub packed str
pub struct jsdom.Session  ## A page whose scripts have run and whose engine is kept, so that what
    machine garlicjs.vm.Vm
    changes int  ## How much of the engine's logs earlier calls already handed back.
    requests int
    console int
    cookies int
    pub prevented bool  ## Whether the last `click` or `key` was cancelled by a listener.
    fn jsdom.Session.collect(mut self, mut d html.Doc) jsdom.Outcome [mutates]  ## What the engine did since the last call, replayed into `d`.
    pub fn jsdom.Session.reply(mut self, mut d html.Doc, id int, json str, budget_ms float) jsdom.Outcome [io mutates]  ## The browser's JSON answer to request `id` (see `Ask`): the page's
    pub fn jsdom.Session.click(mut self, mut d html.Doc, id int, budget_ms float) jsdom.Outcome [io mutates]  ## The user clicked node `id`: `click` is dispatched (it bubbles and can be
    pub fn jsdom.Session.input(mut self, mut d html.Doc, id int, text str, budget_ms float) jsdom.Outcome [io mutates]  ## The user typed `text` into field `id`, replacing its value: `keydown`,
    pub fn jsdom.Session.focus(mut self, mut d html.Doc, id int, budget_ms float) jsdom.Outcome [io mutates]  ## Focus moved to node `id`, or to nothing for -1 (`blur`/`focus` events).
    pub fn jsdom.Session.key(mut self, mut d html.Doc, id int, key str, budget_ms float) jsdom.Outcome [io mutates]  ## The user pressed `key` with focus in node `id`; `prevented` says whether
    pub fn jsdom.Session.submit(mut self, mut d html.Doc, id int, budget_ms float) jsdom.Outcome [io mutates]  ## The user submitted the form of node `id` (a button or a field in it):
pub fn jsdom.run(mut d html.Doc, url str, scripts Map[str, str], cookies Map[str, str], now_ms float, budget_ms float, max_steps int) jsdom.Outcome [io mutates]  ## Run the scripts of `d` (the page at `url`): inline ones, and external ones
pub fn jsdom.start(mut d html.Doc, url str, scripts Map[str, str], cookies Map[str, str], now_ms float, budget_ms float, max_steps int) (jsdom.Session, jsdom.Outcome) [io mutates]  ## Like `run`, and keeps the engine for `Session.click`, `Session.input`
fn jsdom.mirror(d html.Doc) List[garlicjs.vm.DomNode]  ## The engine's copy of the tree: the same nodes with the same ids.
fn jsdom.replay(mut d html.Doc, log List[garlicjs.vm.Mutation]) int [mutates]  ## Apply the engine's log to `d`; new nodes get the same ids in both trees.
```
