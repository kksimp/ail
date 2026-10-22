# garlic: (root)
entry: main in garlic.ail
tests: 63 (`ail test` lists them)
ail.pkg:
    package garlic
    dep garlicjs path js
    dep image path ../pdf/image caps fs

## (root)
```
KEYS := ...  ## The keys, as `--keys` prints them.
enum Prompt  ## What the bottom row is doing: showing the status, or reading a URL, a
struct Browser  ## The browser: the page shown (its tree and layout), where it is scrolled,
fn options() net.Options [io net]  ## The headers Garlic sends: who it is, and that it wants HTML.
fn image_options() net.Options [io net]  ## The headers for a picture: any image type.
fn fetch(str, str, str) str [clock fs io net]  ## The text of `url`: `http` and `https` through `net.request` (which
fn fetch_with(str, str, str, mut std.net.jar.Jar) str [clock fs io mutates net]  ## `fetch`, sending `cj`'s cookies for the site and keeping the ones the
WINDOWS_1252 := ...  ## The labels that name windows-1252, which HTML reads `iso-8859-1` and
fn body_text(net.RawResponse) str  ## A response body as text: its `Content-Encoding` (gzip or deflate)
fn decode(bytes, str) str  ## `data` as text in the charset that `content_type` names, or failing that
fn charset_label(str) str  ## The value of the `charset=` in `s`, lowercased and unquoted; "" if none.
fn error_page(str, str) str
MAX_REFRESHES := 5  ## At most this many hops (a `<meta http-equiv=refresh>` or a script
MAX_REFRESH_DELAY := 5  ## A refresh with a longer delay is left alone, as the page asks to be read.
fn refresh_target(html.Doc, str, bool) str?  ## Where the page's `<meta http-equiv=refresh content="N; url=...">` sends
MAX_SHEETS := 8  ## At most this many linked style sheets are fetched per page.
fn linked_sheets(html.Doc, str) css.Sheet [fs io net]  ## The page's linked style sheets, fetched and parsed in document order.
fn fetch_text(str) str [fs io net]  ## The text of a sub-resource, or "" when it can't be fetched.
struct Loaded  ## A loaded page: where it ended up and its tree.
fn load_doc(str, str, str, mut std.net.jar.Jar) Loaded [clock env fs io mutates net]  ## Fetches and parses `target`, then runs its scripts (unless `--no-js`),
fn script_url(str) str  ## The address scripts see: a local file gets a file:// URL.
fn local_url(str) str
fn edit_text(str, str) str?  ## `text` after a key typed into a prompt: Backspace deletes, a character
fn form_fields(html.Doc, int, int) str  ## The form's fields, URL-encoded (`application/x-www-form-urlencoded`):
fn urlencode(str) str  ## `s` encoded for a form: letters, digits and `*-._` as they are, space as
fn utf8(int) List[int]  ## The UTF-8 bytes of code point `k`.
struct Gui  ## The native window's side of the browser: the host's cell size in points
fn address(str) str [fs io]  ## What a typed address means: a URL as it is, a path to a file that exists
struct Garlic  ## The native window as a `std.app` app (docs/design/garlic-browser.md,
impl std.app.App for Garlic
fn to_app(draw.Op, Gui) std.app.Op  ## A drawing operation of the page, as a `std.app` canvas operation.
GUI_W := 960.0  ## The native window: the page area starts at `GUI_W` by `GUI_H` points with
GUI_H := 600.0
GUI_CELL_W := 7.8
GUI_CELL_H := 16.0
fn app_main(Browser, bool) ! [app clock env fail fs io net proc]
fn flag(List[str], str) str?  ## The value after flag `name` (`--width 60`), or none.
fn usage() [io]
fn main() ! [app clock env fail fs io net proc]
fn tree(str) str  ## The tree of `src` in the html5lib-tests format.
fn lay(str, int) page.Page
fn testdata() str [env fs io]  ## `tools/garlic/testdata`, from the repository root or `bootstrap/`.
fn show_ops(List[draw.Op]) List[str]  ## The ops of a display list as `text@x,y` and `rect@x,y wxh` lines.
fn resized(float, float) std.app.Event
fn clicked(float, float) std.app.Event
fn node_with(Browser, str, str, str) int  ## The id of the first element `tag` whose `attr` is `value` in `b`'s page.
fn pixel_text(List[draw.Op]) List[str]  ## The text of the proportional `Str` operations in `ops`, as `text@x,y`.
fn pixels(str, float) gfx.Scene
fn bench_run(int, str, html.Doc, css.Sheet) int
fn bench(List[str]) [clock fs io]  ## `garlic --bench FILE [ROUNDS]`: `args` are the words after `--bench`.
fn bitmap_data_lens(List[std.app.Op]) List[List[int]]
```

trimmed to fit 2000 tokens (~1521); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
