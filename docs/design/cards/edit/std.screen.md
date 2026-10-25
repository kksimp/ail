# edit: std.screen

## std.screen
```
pub struct std.screen.Item  ## One thing on the screen. `id` is stable across frames (std.screen.5);
    pub id int
    pub role str
    pub text str
    pub color int
    pub x float
    pub y float
    pub w float
    pub h float
    pub enabled bool
    pub focused bool
    pub href str
    pub conf float
    pub why str
    pub fn std.screen.Item.line(self) str  ## The one-line answer for `it` (std.screen.2):
pub struct std.screen.Screen
    pub w float
    pub h float
    pub items List[std.screen.Item]
    pub fn std.screen.Screen.what_is_at(self, x float, y float) str  ## The topmost item at point (`x`, `y`): the last one drawn (std.screen.2).
    pub fn std.screen.Screen.find(self, role str, color str, text str, budget int) str  ## The items matching every non-empty criterion: `role`, colour name
    pub fn std.screen.Screen.describe(self, x float, y float, rw float, rh float, budget int) str  ## The items that overlap the region (std.screen.2).
    pub fn std.screen.Screen.changes_since(self, before std.screen.Screen, budget int) str  ## What differs from `before` (std.screen.8): items that appeared (`+`), went
pub fn std.screen.item(id int, role str, text str, color int, x float, y float, w float, h float) std.screen.Item  ## A plain item with the usual defaults: enabled, unfocused, certain.
pub fn std.screen.color_name(rgb int) str  ## `rgb` as a name from a fixed palette (std.screen.7), so a theme change or
fn std.screen.num(f float) str
fn std.screen.interactive(role str) bool
fn std.screen.rank(it std.screen.Item) int  ## How much `it` matters (std.screen.4): focused, then interactive, then
fn std.screen.inside(it std.screen.Item, x float, y float, w float, h float) bool
fn std.screen.fits(used int, line str, budget int) bool  ## Roughly four characters to a token.
pub fn std.screen.lines(items List[std.screen.Item], budget int) str  ## `items` as lines, most important first, in about `budget` tokens (0 for no
```
