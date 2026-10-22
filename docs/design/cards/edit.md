# edit
entry: main in edit.ail
tests: 59 (`ail test` lists them)

## (root)
```
BG := 1973790
FG := 13948116
LINE_BG := 2763306
GUTTER_FG := 8750469
STATUS_BG := 31436
STATUS_FG := 16777215
FONT_SIZES := ...
PANE_BG := 2434342
TREE_W := 28
KEYWORD_FG := 5676246
TYPE_FG := 5163440
STRING_FG := 13537656
NUMBER_FG := 11914920
COMMENT_FG := 6986069
ATTR_FG := 12945088
OP_FG := 13948116
ERROR_BG := 5905693
WARN_BG := 5065245
SELECT_BG := 2510712
BRACKET_BG := 3887437
OCCUR_BG := 3422784
ERROR_FG := 16025457
WARN_FG := 13412096
FG_KINDS := ...
BG_KINDS := ...
UNDO_LIMIT := 200
INDENT := 4
struct TextEdit
impl Ord for TextEdit
struct Diag
struct OutlineItem
struct Snapshot
enum Prompt
struct Symbol
struct SymbolRef
struct Completion
impl Ord for Completion
enum Pane
struct Editor
fn splice(List[str], int, int, List[str]) List[str]
fn indent_of(str) int
BLOCK_WORDS := "fn if elif else while for match struct enum trait impl error"
fn opens_block(str) bool
struct Range
fn sel_bg(Range?, int, int, int) int
OPENERS := "([{\""
CLOSERS := ")]}\""
fn all_digits(str) bool
fn word_marks(List[char], str) List[bool]
struct ApiIndex
fn facts_of(Json) List[str]
fn parse_api(str) ApiIndex
fn typed_word(Command) str?
fn parse_type_name(str) str
fn parse_locals(Json) List[Completion]
enum Dir
fn is_word(char) bool
KEYWORDS := ...
fn quoted_end(List[char], int, bool) int
TRIPLE := "\"\"\""
fn triple_end(List[char], int) int
fn ends_in_triple(List[char], bool) bool
fn line_colors(List[char], bool) List[int]
fn colors(List[char]) List[int]
struct Span
struct Screen
fn diag_bg(List[Diag], int, int, int) int
enum Command
fn drop_last(str) str
fn find_in(List[char], List[char], int) int?
fn match_from(List[char], List[char], int, bool) int?
fn find_label(bool) str
fn on_off(bool) str
fn go_or_select(Dir, bool) Command
fn key_command(str, bool, bool, bool) Command?
fn ail_command() str [env fs io]
fn jstr(Json, str) str
fn jint(Json, str) int
fn jarr(Json, str) List[Json]
fn parse_diags(str) List[Diag]
fn parse_outline(str) List[OutlineItem]
fn typed(str, List[Command]) Editor
fn keys(str) List[Command]
DIAG_JSON := ...
API_SRC := ...
API_JSON := ...
fn indexed(List[Command]) Editor
fn press(str) ui.Event
ADD_REFS_JSON := ...
fn fg_at(Screen, int, int) int
LOCALS_JSON := ...
REFS_JSON := ...
TYPE_JSON := ...
fn draw(mut ui.Window, Screen) [io mutates ui]
fn draw_themed(mut ui.Window, Screen, Palette) [io mutates ui]
struct Palette
fn palette(Map[str, int]) Palette
fn settings_path() str? [env]
fn setting(str, str) str?
fn with_setting(str, str, str) str
fn saved_theme() str [env fs io]
fn save_theme(str) str [env fs io]
fn session_key(str) str
fn session_value(List[Editor], int) str
fn session_places(str) List[Loc]
enum Extra
fn extra_key(str, bool, bool, bool) Extra?
fn test_counts(str) str
struct Loc
fn number_at(List[char], int) Range?
fn output_location(str, str) Loc?
struct KeyRow
fn key_row(str, str, str, str) KeyRow
fn key_rows() List[KeyRow]
fn key_table() List[str]
fn key_event(str) ui.Event
fn pressed(ui.Event) str?
enum WsKey
fn workspace_key(str, bool, bool) WsKey?
struct Workspace
fn base_name(str) str
fn shown_path(str, str) str
struct RenamePlan
fn place(int, int, str) Range
fn valid_name(str) bool
struct TreeRow
struct Hit
fn search_text(str, str, str) List[Hit]
fn jtext(Json) str
fn json_pairs(Json) str
fn trace_hits(str, str, str) List[Hit]
fn failure_hits(str, str) List[Hit]
fn parse_coverage(str) Set[int]
struct Source
fn declared_at(List[Source], str) Loc?
fn card_hits(Json, List[Source]) List[Hit]
fn card_item(mut List[Hit], Json, List[Source], int, Loc) [mutates]
fn refs_hits(str, Json) List[Hit]
fn problem_hits(str, str, mut Set[str]) List[Hit] [mutates]
fn problems_title(List[Hit]) str
fn problem_key(Hit) str
fn list_files(str, int, int) List[str] [fs io]
fn fuzzy(str, str) bool
fn picker_list() std.ui.widgets.ListView
fn window_main(mut Editor, List[str]) [clock env fs io mutates proc ui]
fn font_flag(List[str]) int
fn print_screen(Editor, List[str]) [clock env fs io proc ui]
fn bench(mut Editor) [clock io mutates]
fn main() ! [clock env fail fs io proc ui]
```

## std.ui.widgets
```
pub struct std.ui.widgets.Rect
pub struct std.ui.widgets.Run
pub enum std.ui.widgets.Action
pub fn std.ui.widgets.color(Map[str, int], str) int
pub fn std.ui.widgets.draw_runs(mut ui.Window, List[std.ui.widgets.Run]) [io mutates ui]
pub fn std.ui.widgets.text_rows(List[std.ui.widgets.Run], int, int) List[str]
pub fn std.ui.widgets.fit(str, int) str
pub fn std.ui.widgets.fuzzy(str, str) bool
pub struct std.ui.widgets.TextField
pub struct std.ui.widgets.ListView
pub struct std.ui.widgets.Button
pub struct std.ui.widgets.Dialog
pub struct std.ui.widgets.Pane
pub struct std.ui.widgets.Node
pub struct std.ui.widgets.Tree
```

## themes
```
pub themes.NAMES := ...
pub fn themes.named(str) Map[str, int]?
```

trimmed to fit 4000 tokens (~1869); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
