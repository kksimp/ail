# garlic: std.app

## std.app
```
pub enum std.app.Op  ## One drawing step of a `Canvas`. Coordinates are in points, colors `0xRRGGBB`.
    Str(x float, y float, text str, font str, size float, fg int, bold bool, italic bool, underline bool)  ## Text whose top-left corner is at `x`, `y`; `font` is `mono` or `sans`.
    Fill(x float, y float, w float, h float, color int)  ## A filled rectangle.
    Stroke(x1 float, y1 float, x2 float, y2 float, color int, width float)  ## A line of the given width.
    Pic(x float, y float, w float, h float, src str)  ## An image from a URL or file path, scaled to the rectangle.
    Clip(x float, y float, w float, h float)  ## Limits later steps to a rectangle, until the matching `Unclip`.
    Unclip
pub enum std.app.Node  ## A piece of the screen.
    Label(text str)  ## Static text.
    Button(id str, caption str)  ## A push button; its `id` comes back in a `press` event.
    Field(id str, value str, hint str)  ## A one-line text field; changes come back as `edit` events.
    Toggle(id str, caption str, on bool)  ## A check box; a click comes back as a `toggle` event.
    Column(kids List[std.app.Node])  ## Children from top to bottom.
    Row(kids List[std.app.Node])  ## Children from left to right.
    Spacer  ## Flexible empty space.
    Canvas(id str, w float, h float, ops List[std.app.Op])  ## A drawing surface of a size in points; it also sends `click`, `scroll`
fn std.app.member(name str, value Json) JsonMember  ## The JSON of `Op` and `Node` is written out, not derived: a derived impl
fn std.app.tagged(kind str, rest List[JsonMember]) Json
fn std.app.num(n float) Json
fn std.app.txt(s str) Json
impl ToJson for std.app.Op
    fn to_json(self) Json
impl ToJson for std.app.Node
    fn to_json(self) Json
pub struct std.app.Event  ## What happened. `kind` is `start`, `press`, `edit`, `submit`, `toggle`,
    pub kind str
    pub id str  ## The control's id (`press`, `edit`, `submit`, `toggle`, and the canvas
    pub text str  ## The new text (`edit`), the key's name as std.ui names it (`key`), or
    pub x float
    pub y float
    pub dx float  ## Scroll distance for `scroll`; the canvas size for `resize`.
    pub dy float
    pub cell_w float  ## For `resize`: the canvas's monospace cell size in points.
    pub cell_h float
    pub ctrl bool
    pub alt bool
    pub shift bool
    pub meta bool
pub error struct std.app.AppError  ## Why an app couldn't run (std.app.9): `no_host`, `start`, `version` or
    pub kind str
    pub message str
pub fn std.app.label(s str) std.app.Node  ## Static text.
pub fn std.app.column(kids List[std.app.Node]) std.app.Node  ## Children from top to bottom.
pub fn std.app.row(kids List[std.app.Node]) std.app.Node  ## Children from left to right.
pub fn std.app.button(id str, caption str) std.app.Node  ## A push button.
pub fn std.app.field(id str, value str, hint str) std.app.Node  ## A one-line text field.
pub fn std.app.toggle(id str, caption str, on bool) std.app.Node  ## A check box.
pub fn std.app.canvas(id str, w float, h float, ops List[std.app.Op]) std.app.Node  ## A drawing surface.
pub fn std.app.draw_text(x float, y float, s str, fg int) std.app.Op  ## Plain monospace text of size 13 on a canvas.
pub fn std.app.draw_rect(x float, y float, w float, h float, color int) std.app.Op  ## A filled rectangle on a canvas.
pub fn std.app.event(kind str) std.app.Event  ## An event of `kind` with every other field zero or empty.
pub fn std.app.press(id str) std.app.Event  ## A button press, for tests.
pub fn std.app.edit(id str, value str) std.app.Event  ## A field edit, for tests.
pub fn std.app.key(name str) std.app.Event  ## A key, for tests.
pub trait std.app.App
    fn update(mut self, ev std.app.Event) [mutates]  ## Changes the state for an event.
    fn view(self) std.app.Node  ## The screen for the current state.
    fn title(self) str  ## The window title.
    fn focus(self) str  ## The id of the control to give the keyboard focus; the host acts when
fn std.app.indent(depth int) str
pub fn std.app.outline(n std.app.Node) str  ## The tree as indented text: what the headless renderer prints.
fn std.app.outline_lines(n std.app.Node, depth int) List[str]
fn std.app.group(head str, kids List[std.app.Node], depth int) List[str]
fn std.app.op_text(op std.app.Op) str
pub fn std.app.step[T std.app.App](mut a T, ev std.app.Event) str [clock env fs io mutates net]  ## Feeds `ev` to `app` and returns the new outline: one step of a test.
pub fn std.app.event_from_json(j Json) std.app.Event  ## The event a JSON object describes (std.app.8): members that are missing
pub fn std.app.frame[T std.app.App](a T) str  ## The message that shows `a` (std.app.7): its title, focus and view tree.
pub fn std.app.script[T std.app.App](mut a T, events List[std.app.Event]) List[str] [clock env fs io mutates net]  ## Feeds each event to `a` in order, with no host (std.app.11), and returns
pub fn std.app.host_path() str ! std.app.AppError [env fail fs io]  ## The host program (std.app.10): `AIL_APP_HOST`, else the one built under
pub fn std.app.run[T std.app.App](mut a T) !std.app.AppError [app clock env fail fs io mutates net proc]  ## Runs `a` in a window (std.app.6-10): starts the host, sends the view
    fails with std.app.AppError
pub fn std.app.run_host[T std.app.App](mut a T, cmd str, args List[str]) !std.app.AppError [app clock env fail fs io mutates net proc]  ## Runs `a` as `run` does with the host program `cmd` and its arguments
    fails with std.app.AppError
fn std.app.gone(move c proc.Child) !std.app.AppError [fail io proc]
    fails with std.app.AppError
fn std.app.mismatch(move c proc.Child, line str) !std.app.AppError [fail io proc]
    fails with std.app.AppError
```
