# garlic: std.net.jar

## std.net.jar
```
pub struct std.net.jar.Cookie  ## One stored cookie.
    pub name str
    pub value str
    pub domain str  ## The domain it applies to, lowercase, without a leading dot.
    pub host_only bool  ## Sent only to exactly `domain`, not its subdomains (no `Domain=` given).
    pub path str
    pub expires_ms int  ## Unix milliseconds when it expires; 0 for a session cookie.
    pub secure bool
    pub http_only bool
    pub same_site str  ## `lax` (the default), `strict` or `none` (spec std.net.23).
    pub seq int  ## Order of arrival, to break ties between equally long paths.
pub struct std.net.jar.Jar
    pub items List[std.net.jar.Cookie]
    pub next_seq int
    pub fn std.net.jar.Jar.set_cookie(mut self, url str, line str) bool [clock mutates]  ## Stores one `Set-Cookie` line received from `url` (RFC 6265 5.3). A line
    pub fn std.net.jar.Jar.store(mut self, url str, r net.Response) [clock mutates]  ## Stores every `Set-Cookie` of a response received from `url`.
    pub fn std.net.jar.Jar.drop_expired(mut self) [clock mutates]  ## Forgets cookies whose time has passed.
    pub fn std.net.jar.Jar.matching(self, url str) List[std.net.jar.Cookie] [clock]  ## The cookies a request to `url` carries, longest path first and then oldest
    pub fn std.net.jar.Jar.matching_from(self, url str, from str, navigation bool) List[std.net.jar.Cookie] [clock]  ## The cookies a request to `url` carries when it is made from the page
    pub fn std.net.jar.Jar.header(self, url str) str [clock]  ## The value of the `Cookie` header for `url`: `a=1; b=2`, or `""`.
    pub fn std.net.jar.Jar.header_from(self, url str, from str, navigation bool) str [clock]  ## `header` for a request made from the page `from` (see `matching_from`).
    pub fn std.net.jar.Jar.value(self, url str, name str) str? [clock]  ## The value of the cookie `name` that a request to `url` would carry.
    pub fn std.net.jar.Jar.for_script(self, url str) Map[str, str] [clock]  ## The cookies a page's scripts at `url` can read, by name: those without
    pub fn std.net.jar.Jar.set_from_script(mut self, url str, line str) bool [clock mutates]  ## A `document.cookie = line` assignment from a script at `url`: like
    pub fn std.net.jar.Jar.len(self) int  ## How many cookies are stored.
    pub fn std.net.jar.Jar.clear(mut self) [mutates]  ## Forgets every cookie.
    pub fn std.net.jar.Jar.request(mut self, method str, url str, opts net.Options) net.Response ! net.NetError [clock fail io mutates net]  ## `net.request` with this jar's cookies attached and every response's
    pub fn std.net.jar.Jar.get(mut self, url str) net.Response ! net.NetError [clock fail io mutates net]  ## `GET url` through this jar, with the defaults of `net.get`.
pub fn std.net.jar.new() std.net.jar.Jar  ## An empty jar.
struct std.net.jar.Target
    scheme str
    host str
    path str
    authority str  ## Host and port as written, lowercase: what makes two URLs one origin.
fn std.net.jar.parse_target(url str) std.net.jar.Target
fn std.net.jar.default_path(path str) str  ## RFC 6265 5.1.4: the directory of a request path.
fn std.net.jar.is_ip(host str) bool
fn std.net.jar.domain_matches(host str, domain str) bool  ## RFC 6265 5.1.3.
fn std.net.jar.path_matches(request_path str, cookie_path str) bool  ## RFC 6265 5.1.4.
fn std.net.jar.month_number(name str) int
fn std.net.jar.days_from_civil(year int, month int, day int) int  ## Days since 1970-01-01 of a proleptic Gregorian date.
fn std.net.jar.parse_date(text str) int?  ## RFC 6265 5.1.1: a cookie date in any of the three common layouts, as unix
fn std.net.jar.site_of(host str) str  ## The site of a host: its last two labels (an IP address is its own site).
fn std.net.jar.resolve(base str, location str) str
```
