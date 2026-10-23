# garlic: page

## page
```
pub struct page.Style  ## How a run is drawn. `link` is the link's number (1-based), 0 if none;
    pub bold bool
    pub italic bool
    pub code bool
    pub quiet bool
    pub link int
    pub heading int
    pub target int  ## The target's number (1-based) for focus, 0 if none.
    pub upper bool  ## `text-transform: uppercase`.
pub struct page.Run  ## Text drawn in one style on one row, starting at column `col`.
    pub col int
    pub text str
    pub style page.Style
pub struct page.Box  ## A perceivable element and where it was drawn. Rows and columns are
    pub id int
    pub role str
    pub name str
    pub href str
    pub level int
    pub link int  ## The link's number (1-based), 0 if not a link.
    pub value str  ## A form control's current value (`textbox`, `combobox`), or "".
    pub checked bool  ## Whether a `checkbox` or `radio` is checked.
    pub row int
    pub col int
    pub end_row int
    pub end_col int
pub struct page.Page  ## A laid-out page.
    pub url str
    pub title str
    pub width int
    pub rows List[List[page.Run]]
    pub boxes List[page.Box]
    pub links List[int]  ## For each link number n, `links[n - 1]` is its box's index.
    pub targets List[int]  ## The boxes an agent or the keyboard can act on (links and form
    pub changed List[int]  ## The ids of nodes changed since the page was parsed (`html.Doc.changed`).
    pub fn page.Page.row_text(self, r int) str  ## Row `r` as plain text.
    pub fn page.Page.text(self) str  ## The whole page as plain text, one row per line.
    pub fn page.Page.find(self, needle str, from int) int?  ## The first row at or after `from` whose text contains `needle`, ignoring
    pub fn page.Page.to_json(self, scroll int, height int, viewport_only bool) str  ## The page model as one JSON object (api 1): the viewport `height` rows
    pub fn page.Page.head_json(self, scroll int, height int) str  ## The model's members before `nodes`: where the page is and what changed.
    pub fn page.Page.node_objects(self, scroll int, height int) Map[int, str]  ## Each drawn box as its JSON object, by node id, in document order.
    pub fn page.Page.to_json_budget(self, scroll int, height int, max_chars int) str  ## The model in at most about `max_chars` characters: the viewport's nodes
pub page.HIDDEN := ...  ## Elements never drawn: metadata, scripts and templates.
pub page.BLOCKS := ...  ## Elements laid out as blocks; the rest flow inline.
pub fn page.input_type(d html.Doc, id int) str  ## The `type` of an `input`, lowercase; "text" when absent.
pub fn page.control_value(d html.Doc, id int) str  ## A form control's value: an input's `value`, a textarea's text (or the
pub fn page.layout(d html.Doc, url str, width int) page.Page  ## Lays `d` out at `width` columns; `url` resolves relative links. The
pub fn page.referrer(from str, to str) str  ## The `Referer` for a request to `to` from the page `from`: the whole
pub fn page.base_of(d html.Doc, url str) str  ## Where relative addresses in `d` start from: its first `<base href>`
pub fn page.layout_with(d html.Doc, url str, width int, sheet css.Sheet) page.Page  ## Lays `d` out with `sheet` (its linked style sheets) before its own.
pub fn page.quote(s str) str  ## `s` as a JSON string.
pub fn page.scheme(u str) str  ## The scheme of `u` (`https`), or "" when it has none (RFC 3986 3.1).
pub fn page.resolve(base str, rel str) str  ## `rel` resolved against `base` (RFC 3986 5.2): absolute URLs as they are,
pub fn page.remove_dots(p str) str  ## The path with `.` and `..` segments removed (RFC 3986 5.2.4); the query
```

trimmed to fit 2000 tokens (~1184); dropped: private items
