# garlic: jsdom
tests: jsdom.jsdom.cookies_round_trip

## jsdom
```
pub struct jsdom.Cookie
    pub name str
    pub value str
    pub domain str  ## The host it was set by, or the `Domain` it names (no leading dot).
    pub shared bool  ## Whether `Domain` was given, so subdomains get it too.
pub struct jsdom.Jar
    pub cookies List[jsdom.Cookie]
    pub fn jsdom.Jar.for_url(self, url str) Map[str, str]  ## The cookies for `url`, by name, as a page's scripts see them.
    pub fn jsdom.Jar.header(self, url str) str  ## The `Cookie` header for a request to `url`, or "".
    pub fn jsdom.Jar.set(mut self, url str, line str, now_ms float) [mutates]  ## Store one `Set-Cookie` value (or a `document.cookie` assignment) from
    pub fn jsdom.Jar.set_all(mut self, url str, header str, now_ms float) [mutates]  ## Store every cookie of a response's `set-cookie` header, whose values
pub fn jsdom.new_jar() jsdom.Jar
pub fn jsdom.host_of(url str) str  ## The host of an `http`/`https` URL, in lowercase, or "".
fn jsdom.matches(c jsdom.Cookie, host str) bool
pub fn jsdom.split_set_cookie(header str) List[str]  ## Split joined `Set-Cookie` values: a ", " starts a new cookie only when
fn jsdom.http_date_ms(text str) float?  ## An `Expires` date such as "Wed, 21-Apr-2027 19:12:31 GMT" or
fn jsdom.days_from_civil(year int, month int, day int) int  ## Days from 1970-01-01 to a date (H. Hinnant's days_from_civil).
pub struct jsdom.Outcome  ## What running a page's scripts left for the browser to do.
    pub url str  ## A navigation the page asked for (the first one), or "".
    pub method str
    pub body str
    pub console List[str]  ## Console lines and uncaught errors, in order.
    pub missing List[str]  ## External scripts the page wanted that weren't in `scripts`.
    pub changes int  ## How many changes were replayed.
    pub cookies List[str]  ## What scripts assigned to `document.cookie`, in order.
pub fn jsdom.run(mut d html.Doc, url str, scripts Map[str, str], cookies Map[str, str], now_ms float, budget_ms float, max_steps int) jsdom.Outcome [io mutates]  ## Run the scripts of `d` (the page at `url`): inline ones, and external ones
fn jsdom.mirror(d html.Doc) List[garlicjs.vm.DomNode]  ## The engine's copy of the tree: the same nodes with the same ids.
fn jsdom.replay(mut d html.Doc, log List[garlicjs.vm.Mutation]) int [mutates]  ## Apply the engine's log to `d`; new nodes get the same ids in both trees.
```
