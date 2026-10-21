# edit: std.ui.widgets

## std.ui.widgets
```
pub struct std.ui.widgets.Rect
    pub col int
    pub row int
    pub cols int
    pub rows int
pub struct std.ui.widgets.Run
    pub col int
    pub row int
    pub text str
    pub fg int
    pub bg int
    pub bold bool
pub enum std.ui.widgets.Action
    Ignored
    Changed
    Submit
    Cancel
pub fn std.ui.widgets.color(theme Map[str, int], kind str) int
pub fn std.ui.widgets.draw_runs(mut w ui.Window, runs List[std.ui.widgets.Run]) [io mutates ui]
pub fn std.ui.widgets.text_rows(runs List[std.ui.widgets.Run], cols int, rows int) List[str]
pub fn std.ui.widgets.fit(text str, n int) str
pub fn std.ui.widgets.fuzzy(item str, filter str) bool
pub struct std.ui.widgets.TextField
    pub label str
    pub text str
    pub cursor int
    pub fn std.ui.widgets.TextField.new(label str, text str) std.ui.widgets.TextField
    pub fn std.ui.widgets.TextField.on_event(mut self, ev ui.Event) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.TextField.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.TextField.caret_col(self, r std.ui.widgets.Rect) int
    pub fn std.ui.widgets.TextField.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
pub struct std.ui.widgets.ListView
    pub items List[str]
    pub filter str
    pub selected int
    pub top int
    pub empty str
    pub fn std.ui.widgets.ListView.new(items List[str]) std.ui.widgets.ListView
    pub fn std.ui.widgets.ListView.shown(self) List[str]
    pub fn std.ui.widgets.ListView.chosen(self) str?
    pub fn std.ui.widgets.ListView.on_event(mut self, ev ui.Event, r std.ui.widgets.Rect) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.ListView.scroll_to(mut self, rows int) [mutates]
    pub fn std.ui.widgets.ListView.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.ListView.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
pub struct std.ui.widgets.Button
    pub label str
    pub focused bool
    pub fn std.ui.widgets.Button.on_event(mut self, ev ui.Event, r std.ui.widgets.Rect) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.Button.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.Button.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
pub struct std.ui.widgets.Dialog
    pub message str
    pub choices List[str]
    pub selected int
    pub fn std.ui.widgets.Dialog.new(message str, choices List[str]) std.ui.widgets.Dialog
    pub fn std.ui.widgets.Dialog.on_event(mut self, ev ui.Event, r std.ui.widgets.Rect) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.Dialog.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.Dialog.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
pub struct std.ui.widgets.Pane
    pub lines List[str]
    pub top int
    pub cursor int
    pub fn std.ui.widgets.Pane.new(lines List[str]) std.ui.widgets.Pane
    pub fn std.ui.widgets.Pane.to_end(mut self, rows int) [mutates]
    pub fn std.ui.widgets.Pane.on_event(mut self, ev ui.Event, r std.ui.widgets.Rect) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.Pane.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.Pane.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
pub struct std.ui.widgets.Node
    pub label str
    pub depth int
    pub open bool
pub struct std.ui.widgets.Tree
    pub nodes List[std.ui.widgets.Node]
    pub selected int
    pub fn std.ui.widgets.Tree.new(nodes List[std.ui.widgets.Node]) std.ui.widgets.Tree
    pub fn std.ui.widgets.Tree.has_children(self, i int) bool
    pub fn std.ui.widgets.Tree.visible(self) List[int]
    pub fn std.ui.widgets.Tree.on_event(mut self, ev ui.Event, r std.ui.widgets.Rect) std.ui.widgets.Action [mutates]
    pub fn std.ui.widgets.Tree.render(self, r std.ui.widgets.Rect, theme Map[str, int]) List[std.ui.widgets.Run]
    pub fn std.ui.widgets.Tree.draw(self, mut w ui.Window, r std.ui.widgets.Rect, theme Map[str, int]) [io mutates ui]
```

trimmed to fit 2000 tokens (~1612); dropped: private items, doc comments
