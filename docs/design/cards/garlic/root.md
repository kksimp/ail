# garlic: (root)
entry: main in garlic.ail
tests: 86 (`ail test` lists them)
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
fn fetch_got(str, str, str, str, mut std.net.jar.Jar) Got [clock fs io mutates net]
struct Walked
fn walk(str, str, str, str, str, bool, bool, mut std.net.jar.Jar) Walked [clock io mutates net]
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
fn max_age(str) int
fn fetch_text(str, str, mut std.net.jar.Jar, mut Map[str, Cached]) str [clock fs io mutates net]
struct Loaded
fn load_doc(str, str, str, str, mut std.net.jar.Jar, mut Map[str, Cached]) Loaded [clock env fs io mutates net]
fn answer_asks(mut jsdom.Session, mut html.Doc, jsdom.Outcome, str, mut std.net.jar.Jar) jsdom.Outcome [clock io mutates net]
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
TOOLS_SRC := ...
fn tools_json() str
struct Gui
fn address(str) str [fs io]
MEASURE_RUNS := 5000
struct Garlic
impl std.app.App for Garlic
fn to_app(draw.Op, Gui) std.app.Op
GUI_W := 960.0
GUI_H := 600.0
GUI_CELL_W := 7.8
GUI_CELL_H := 16.0
fn app_main(Browser, bool) ! [app clock env fail fs io net proc spawn]
fn flag(List[str], str) str?
fn usage() [io]
VALUE_FLAGS := ...
fn target_of(List[str]) str
fn op_line(draw.Op) str
fn pixels_dump(mut Browser, float) [clock fs io mutates net]
fn main() ! [app clock env fail fs io net proc spawn]
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

trimmed to fit 2000 tokens (~1069); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
