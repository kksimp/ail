# garlic: (root)
entry: main in garlic.ail
tests: 106 (`ail test` lists them)
ail.pkg:
    package garlic
    dep garlicjs path js
    dep image path ../pdf/image caps fs

## (root)
```
KEYS := ...
enum Prompt
struct Browser
struct Kept
MAX_KEPT := 16
fn options() net.Options [io net]
fn image_options() net.Options [io net]
fn fetch(str, str, str) str [clock fs io net]
fn fetch_with(str, str, str, mut std.net.jar.Jar) str [clock fs io mutates net]
struct Got
MAX_HOPS := 10
MAX_REPLIES := 32
LOG_KEEP := 20
MAX_WAIT_ROUNDS := 10
fn fetch_got(str, str, str, str, bool, bool, mut std.net.jar.Jar, mut Map[str, Cached]) Got [clock fs io mutates net]
fn fresh_until(Map[str, str], int, int) int
fn keep_cached(str, Walked, int, mut Map[str, Cached]) [mutates]
struct Walked
fn walk(str, str, str, str, str, bool, bool, Map[str, str], mut std.net.jar.Jar) Walked [clock io mutates net]
WINDOWS_1252 := ...
fn body_text(net.RawResponse) str
fn decode(bytes, str) str
fn charset_label(str) str
fn error_page(str, str) str
MAX_REFRESHES := 5
MAX_REFRESH_DELAY := 5
fn refresh_target(html.Doc, str, bool) str?
MAX_SHEETS := 8
fn linked_sheets(html.Doc, str, mut std.net.jar.Jar, mut Map[str, Cached]) css.Sheet [clock fs io mutates net]
fn data_bytes(str) bytes
fn data_text(str) str
struct Cached
MAX_CACHED := 64
MAX_CACHED_TEXT := 2000000
fn max_age(str) int
fn fetch_text(str, str, mut std.net.jar.Jar, mut Map[str, Cached]) str [clock fs io mutates net]
struct Loaded
fn load_doc(str, str, str, str, bool, mut std.net.jar.Jar, mut Map[str, Cached], mut Map[int, ws.Sock]) Loaded [clock env fs io mutates net random]
fn serve_page(mut jsdom.Session, mut html.Doc, jsdom.Outcome, str, mut std.net.jar.Jar, mut Map[int, ws.Sock]) jsdom.Outcome [clock io mutates net random]
fn quiet() jsdom.Outcome
fn socket_op(mut jsdom.Session, mut html.Doc, jsdom.SockOp, str, mut std.net.jar.Jar, mut Map[int, ws.Sock]) jsdom.Outcome [clock io mutates net random]
fn deliver_received(mut jsdom.Session, mut html.Doc, mut Map[int, ws.Sock], int) jsdom.Outcome [io mutates net random]
fn origin_of(str) str
fn script_reply(jsdom.Ask, str, mut std.net.jar.Jar) str [clock io mutates net]
fn script_url(str) str
fn local_url(str) str
fn edit_text(str, str) str?
fn form_fields(html.Doc, int, int) str
fn urlencode(str) str
fn utf8(int) List[int]
fn rpc_id(Json) str
struct Guard
fn host_of(str) str
fn sends(str) bool
fn rpc_reply(str, str) str
fn look_answer(std.screen.Screen, Json) str
TOOLS_SRC := ...
fn tools_json() str
struct Gui
fn address(str) str [fs io]
SEARCH_ENGINES := ...
fn is_query(str) bool
fn search_url(str) str
MEASURE_RUNS := 5000
fn id_tag(float, float, int) List[draw.Op]
struct Garlic
WATCH_TOOLS := ...
fn watched_text(str) str [clock env fs io net random]
fn theme() tabstrip.Theme
impl std.app.App for Garlic
fn to_app(draw.Op, Gui) std.app.Op
GUI_W := 960.0
GUI_H := 600.0
GUI_CELL_W := 7.8
GUI_CELL_H := 16.0
fn app_main(Browser, bool, bool, str) ! [app clock env fail fs io net net.serve proc random spawn]
fn ticker(std.app.Inbox, Atomic[int]) [clock]
fn serve(net.Server, std.app.Inbox, attach.Bridge, str) [io net]
fn attached_main(mut Garlic, str) ! [clock env fail fs io mutates net net.serve proc random spawn]
fn flag(List[str], str) str?
fn usage() [io]
VALUE_FLAGS := ...
fn target_of(List[str]) str
fn op_line(draw.Op) str
fn pixels_dump(mut Browser, float) [clock fs io mutates net]
fn main() ! [app clock env fail fs io net net.serve proc random spawn]
fn tree(str) str
fn lay(str, int) page.Page
fn testdata() str [env fs io]
fn show_ops(List[draw.Op]) List[str]
fn resized(float, float) std.app.Event
fn clicked(float, float) std.app.Event
fn node_with(Browser, str, str, str) int
fn pixel_text(List[draw.Op]) List[str]
fn pixels(str, float) gfx.Scene
fn bench_run(int, str, html.Doc, css.Sheet) int
fn bench(List[str]) [clock fs io]
fn bitmap_data_lens(List[std.app.Op]) List[List[int]]
fn flat(gfx.Scene) List[str]
```

trimmed to fit 2000 tokens (~1481); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
