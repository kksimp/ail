# garlic: std.app

## std.app
```
pub enum std.app.Op  ## One drawing step of a `Canvas`.
pub enum std.app.Node  ## A piece of the screen.
pub struct std.app.Event  ## What happened.
pub struct std.app.Measure  ## A run of text whose advance width the host measures (std.app.13); `font`
pub error struct std.app.AppError  ## Why an app couldn't run (std.app.9): `no_host`, `start`, `version` or
pub fn std.app.label(str) std.app.Node  ## Static text.
pub fn std.app.column(List[std.app.Node]) std.app.Node  ## Children from top to bottom.
pub fn std.app.row(List[std.app.Node]) std.app.Node  ## Children from left to right.
pub fn std.app.button(str, str) std.app.Node  ## A push button.
pub fn std.app.field(str, str, str) std.app.Node  ## A one-line text field.
pub fn std.app.toggle(str, str, bool) std.app.Node  ## A check box.
pub fn std.app.canvas(str, float, float, List[std.app.Op]) std.app.Node  ## A drawing surface.
pub fn std.app.draw_text(float, float, str, int) std.app.Op  ## Plain monospace text of size 13 on a canvas.
pub fn std.app.draw_rect(float, float, float, float, int) std.app.Op  ## A filled rectangle on a canvas.
pub fn std.app.bitmap_data(List[int]) str  ## `R, G, B, A` bytes (each 0..=255) as the `data` of a `Bitmap`.
pub fn std.app.event(str) std.app.Event  ## An event of `kind` with every other field zero or empty.
pub fn std.app.press(str) std.app.Event  ## A button press, for tests.
pub fn std.app.edit(str, str) std.app.Event  ## A field edit, for tests.
pub fn std.app.key(str) std.app.Event  ## A key, for tests.
pub struct std.app.Inbox  ## Where tasks post events into a running app (std.app.14): `post` queues
pub fn std.app.external(str, str) std.app.Event  ## An `external` event from `source` with `data`, for `Inbox.post`.
pub trait std.app.App
pub fn std.app.outline(std.app.Node) str  ## The tree as indented text: what the headless renderer prints.
pub fn std.app.step[T std.app.App](mut T, std.app.Event) str [clock env fs io mutates net]  ## Feeds `ev` to `app` and returns the new outline: one step of a test.
pub fn std.app.event_from_json(Json) std.app.Event  ## The event a JSON object describes (std.app.8): members that are missing
pub fn std.app.frame[T std.app.App](T) str  ## The message that shows `a` (std.app.7): its title, focus and view tree.
pub fn std.app.script[T std.app.App](mut T, List[std.app.Event]) List[str] [clock env fs io mutates net]  ## Feeds each event to `a` in order, with no host (std.app.11), and returns
pub fn std.app.host_path() str ! std.app.AppError [env fail fs io]  ## The host program (std.app.10): `AIL_APP_HOST`, else the one built under
pub fn std.app.run[T std.app.App](mut T) !std.app.AppError [app clock env fail fs io mutates net proc spawn]  ## Runs `a` in a window (std.app.6-10): starts the host, sends the view
pub fn std.app.run_host[T std.app.App](mut T, str, List[str]) !std.app.AppError [app clock env fail fs io mutates net proc spawn]  ## Runs `a` as `run` does with the host program `cmd` and its arguments
pub fn std.app.new_inbox() std.app.Inbox  ## A new inbox for `run_with`: the program starts its tasks that post to it
pub fn std.app.run_with[T std.app.App](mut T, str, List[str], std.app.Inbox) !std.app.AppError [clock env fail fs io mutates net proc spawn]  ## Runs `a` as `run_host` does, with events from `inbox`, which the program
```

trimmed to fit 2000 tokens (~1237); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
