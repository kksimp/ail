# garlic
entry: main in garlic.ail
tests: 57 (`ail test` lists them)
ail.pkg:
    package garlic
    dep garlicjs path js

## (root)
```
KEYS := ...
BG := 1973790
FG := 13948116
QUIET_FG := 8750469
LINK_FG := 5676246
FOCUS_BG := 2510712
HEADING_FG := 14474410
CODE_FG := 13537656
CONTROL_FG := 5163440
STATUS_BG := 31436
STATUS_FG := 16777215
enum Prompt
struct Browser
fn options() net.Options [io net]
fn fetch(str, str, str) str [clock fs io net]
fn fetch_with(str, str, str, mut jsdom.Jar) str [clock fs io mutates net]
WINDOWS_1252 := ...
fn body_text(net.RawResponse) str
fn decode(bytes, str) str
fn charset_label(str) str
fn error_page(str, str) str
MAX_REFRESHES := 5
MAX_REFRESH_DELAY := 5
fn refresh_target(html.Doc, str, bool) str?
MAX_SHEETS := 8
fn linked_sheets(html.Doc, str) css.Sheet [fs io net]
fn fetch_text(str) str [fs io net]
struct Loaded
fn load_doc(str, str, str, mut jsdom.Jar) Loaded [clock env fs io mutates net]
fn script_url(str) str
fn local_url(str) str
fn edit_text(str, str) str?
fn form_fields(html.Doc, int, int) str
fn urlencode(str) str
fn utf8(int) List[int]
fn color(page.Style) int
fn window_main(mut Browser) ! [clock env fail fs io mutates net ui]
fn flag(List[str], str) str?
fn usage() [io]
fn main() ! [clock env fail fs io net ui]
fn tree(str) str
fn lay(str, int) page.Page
fn testdata() str [env fs io]
fn node_with(Browser, str, str, str) int
```

## css
```
pub struct css.Decl
pub struct css.Compound
pub struct css.AttrTest
pub struct css.Selector
pub struct css.Rule
pub struct css.Sheet
pub fn css.parse(str, int) css.Sheet
pub fn css.parse_in(str, int, str) css.Sheet
pub fn css.index(List[css.Rule]) css.Sheet
pub fn css.concat(css.Sheet, css.Sheet) css.Sheet
pub fn css.for_width(css.Sheet, int) css.Sheet
pub fn css.media_matches(str, int) bool
pub fn css.declarations(str) List[css.Decl]
pub fn css.selector(str) css.Selector?
pub fn css.matches(html.Doc, int, css.Selector) bool
pub fn css.cascade(html.Doc, int, css.Sheet) Map[str, str]
pub fn css.document_sheet(html.Doc) css.Sheet
```

## garlicjs.syntax
```
pub struct garlicjs.syntax.Pos
pub struct garlicjs.syntax.Program
pub enum garlicjs.syntax.Stmt
pub struct garlicjs.syntax.VarDecl
pub struct garlicjs.syntax.Declarator
pub enum garlicjs.syntax.ForInit
pub enum garlicjs.syntax.ForLeft
pub struct garlicjs.syntax.Case
pub enum garlicjs.syntax.Expr
pub struct garlicjs.syntax.Quasi
pub enum garlicjs.syntax.Prop
pub enum garlicjs.syntax.Key
pub enum garlicjs.syntax.Pat
pub struct garlicjs.syntax.PatProp
pub struct garlicjs.syntax.Func
pub struct garlicjs.syntax.Class
pub struct garlicjs.syntax.ClassMember
pub fn garlicjs.syntax.expr_pos(garlicjs.syntax.Expr) garlicjs.syntax.Pos
pub fn garlicjs.syntax.pat_pos(garlicjs.syntax.Pat) garlicjs.syntax.Pos
pub fn garlicjs.syntax.expr_kind(garlicjs.syntax.Expr) str
pub fn garlicjs.syntax.pat_kind(garlicjs.syntax.Pat) str
pub fn garlicjs.syntax.expr_suspends(garlicjs.syntax.Expr) bool
pub fn garlicjs.syntax.any_suspends(List[garlicjs.syntax.Expr]) bool
pub fn garlicjs.syntax.pat_suspends(garlicjs.syntax.Pat) bool
pub fn garlicjs.syntax.stmt_suspends(garlicjs.syntax.Stmt) bool
pub fn garlicjs.syntax.block_suspends(List[garlicjs.syntax.Stmt]) bool
pub fn garlicjs.syntax.program_json(garlicjs.syntax.Program, bool) str
pub fn garlicjs.syntax.canonical_json(garlicjs.syntax.Program) str
pub error struct garlicjs.syntax.JsError
pub impl ToStr for garlicjs.syntax.JsError
pub enum garlicjs.syntax.Kind
pub struct garlicjs.syntax.Token
pub struct garlicjs.syntax.Lexer
pub fn garlicjs.syntax.new_lexer(str) garlicjs.syntax.Lexer
pub fn garlicjs.syntax.is_line_term(char) bool
pub fn garlicjs.syntax.id_start(char) bool
pub fn garlicjs.syntax.id_part(char) bool
pub fn garlicjs.syntax.tokenize(str) List[garlicjs.syntax.Token] ! garlicjs.syntax.JsError [fail]
pub fn garlicjs.syntax.token_line(garlicjs.syntax.Token) str
pub struct garlicjs.syntax.Parser
pub fn garlicjs.syntax.parse(str) garlicjs.syntax.Program ! garlicjs.syntax.JsError [fail]
pub fn garlicjs.syntax.number_to_string(float) str
pub fn garlicjs.syntax.quote(str) str
pub fn garlicjs.syntax.in_ranges(List[int], int) bool
```

## garlicjs.vm
```
pub fn garlicjs.vm.utf16_less(str, str) bool
pub fn garlicjs.vm.element_interface(str) str
pub fn garlicjs.vm.new_vm() garlicjs.vm.Vm [mutates]
pub fn garlicjs.vm.arg_at(List[garlicjs.vm.Value], int) garlicjs.vm.Value
pub fn garlicjs.vm.native_is_ctor(str) bool
pub fn garlicjs.vm.is_array(garlicjs.vm.Value) bool
pub fn garlicjs.vm.num_of(garlicjs.vm.Value) float?
pub fn garlicjs.vm.parse_int(str, int) float
pub fn garlicjs.vm.parse_float(str) float
pub fn garlicjs.vm.uri_encode(str, str) str
pub fn garlicjs.vm.iso_date(float) str
pub fn garlicjs.vm.parse_date(str) float
pub struct garlicjs.vm.DomNode
pub struct garlicjs.vm.DomAttr
pub struct garlicjs.vm.Mutation
pub struct garlicjs.vm.HostRequest
pub struct garlicjs.vm.Listener
pub struct garlicjs.vm.Timer
pub fn garlicjs.vm.attr_of_node(garlicjs.vm.DomNode, str) str?
pub fn garlicjs.vm.node_id(garlicjs.vm.Value) int?
pub fn garlicjs.vm.collapse_space(str) str
pub fn garlicjs.vm.is_event_handler_name(str) bool
pub fn garlicjs.vm.form_encode(str) str
pub fn garlicjs.vm.url_part(str, str) str
pub fn garlicjs.vm.resolve_against(str, str) str
pub enum garlicjs.vm.Op
pub struct garlicjs.vm.Target
pub struct garlicjs.vm.Code
pub fn garlicjs.vm.compile_body(garlicjs.syntax.Func) garlicjs.vm.Code ! garlicjs.vm.Thrown [fail]
pub struct garlicjs.vm.Frame
pub struct garlicjs.vm.Completion
pub struct garlicjs.vm.Stop
pub struct garlicjs.vm.AgenRequest
pub fn garlicjs.vm.quote_js(str) str
pub struct garlicjs.vm.Vm
pub enum garlicjs.vm.Flow
pub garlicjs.vm.PRIVATE := "\u0000#"
pub fn garlicjs.vm.same_value_zero(garlicjs.vm.Value, garlicjs.vm.Value) bool
pub fn garlicjs.vm.init_binding(ref garlicjs.vm.Env, str, garlicjs.vm.Value, bool) [mutates]
pub fn garlicjs.vm.copy_env(ref garlicjs.vm.Env) ref garlicjs.vm.Env
pub fn garlicjs.vm.pattern_names(garlicjs.syntax.Pat) List[str]
pub fn garlicjs.vm.expr_text(garlicjs.syntax.Expr) str
pub fn garlicjs.vm.obj_of(garlicjs.vm.Value) ref garlicjs.vm.JsObject?
pub fn garlicjs.vm.str_of(garlicjs.vm.Value) str?
pub fn garlicjs.vm.well_known(str) str
pub garlicjs.vm.ITERATOR := "\u0000@w:Symbol.iterator"
pub garlicjs.vm.PI := 3.141592653589793
pub garlicjs.vm.LN2 := 0.6931471805599453
pub garlicjs.vm.LN10 := 2.302585092994046
pub fn garlicjs.vm.is_nan(float) bool
pub fn garlicjs.vm.is_finite(float) bool
pub fn garlicjs.vm.nan() float
pub fn garlicjs.vm.infinity() float
pub fn garlicjs.vm.fmod(float, float) float
pub fn garlicjs.vm.pow2(int) float
pub fn garlicjs.vm.exponent(float) int
pub fn garlicjs.vm.ulp(float) float
pub fn garlicjs.vm.trunc(float) float
pub fn garlicjs.vm.exp(float) float
pub fn garlicjs.vm.log(float) float
pub fn garlicjs.vm.log2(float) float
pub fn garlicjs.vm.log10(float) float
pub fn garlicjs.vm.pow(float, float) float
pub fn garlicjs.vm.sin(float) float
pub fn garlicjs.vm.cos(float) float
pub fn garlicjs.vm.tan(float) float
pub fn garlicjs.vm.atan(float) float
pub fn garlicjs.vm.atan2(float, float) float
pub fn garlicjs.vm.asin(float) float
pub fn garlicjs.vm.acos(float) float
pub fn garlicjs.vm.expm1(float) float
pub fn garlicjs.vm.log1p(float) float
pub fn garlicjs.vm.sinh(float) float
pub fn garlicjs.vm.cosh(float) float
pub fn garlicjs.vm.tanh(float) float
pub fn garlicjs.vm.cbrt(float) float
pub fn garlicjs.vm.decode_entities(str) str
pub fn garlicjs.vm.to_fixed(float, int) str
pub fn garlicjs.vm.to_exponential(float, int?) str
pub fn garlicjs.vm.to_precision(float, int) str
pub fn garlicjs.vm.to_radix(float, int) str
pub fn garlicjs.vm.is_regexp(garlicjs.vm.Value) bool
pub fn garlicjs.vm.expand_replacement(str, str, str, str, List[str?]) str
pub fn garlicjs.vm.locale_number(float) str
pub fn garlicjs.vm.normalize_url(str) str
pub fn garlicjs.vm.form_decode(str) str
pub enum garlicjs.vm.Value
pub struct garlicjs.vm.Prop
pub struct garlicjs.vm.JsObject
pub enum garlicjs.vm.Callable
pub struct garlicjs.vm.ClassInfo
pub struct garlicjs.vm.Binding
pub struct garlicjs.vm.Env
pub error struct garlicjs.vm.Thrown
pub impl ToStr for garlicjs.vm.Thrown
pub fn garlicjs.vm.new_env(ref garlicjs.vm.Env?, bool) ref garlicjs.vm.Env
pub fn garlicjs.vm.data(garlicjs.vm.Value) garlicjs.vm.Prop
pub fn garlicjs.vm.hidden(garlicjs.vm.Value) garlicjs.vm.Prop
pub fn garlicjs.vm.is_obj(garlicjs.vm.Value) bool
pub fn garlicjs.vm.is_nullish(garlicjs.vm.Value) bool
pub fn garlicjs.vm.is_callable(garlicjs.vm.Value) bool
pub garlicjs.vm.SYMBOL := "\u0000@"
pub fn garlicjs.vm.is_symbol(str) bool
pub fn garlicjs.vm.symbol_description(str) str?
pub fn garlicjs.vm.symbol_text(str) str
pub fn garlicjs.vm.type_of(garlicjs.vm.Value) str
pub fn garlicjs.vm.truthy(garlicjs.vm.Value) bool
pub fn garlicjs.vm.string_to_number(str) float
pub fn garlicjs.vm.js_space(char) bool
pub fn garlicjs.vm.trim_js(str) str
pub fn garlicjs.vm.int_of(float) int
pub fn garlicjs.vm.as_index(float) int?
pub fn garlicjs.vm.index_of_key(str) int?
pub fn garlicjs.vm.to_int32(float) int
pub fn garlicjs.vm.to_uint32(float) int
pub fn garlicjs.vm.units(str) List[int]
pub fn garlicjs.vm.from_units(List[int]) str
pub fn garlicjs.vm.unit_len(str) int
```

## html
```
pub enum html.Kind
pub struct html.Attr
pub struct html.Node
pub enum html.Token
pub struct html.Doc
pub html.VOID := ...
pub fn html.is_space(char) bool
pub fn html.windows_1252(int) char
pub fn html.tokenize(str) List[html.Token]
pub fn html.build(List[html.Token]) html.Doc
pub fn html.parse(str) html.Doc
pub fn html.collapse(str) str
```

## jsdom
```
pub struct jsdom.Cookie
pub struct jsdom.Jar
pub fn jsdom.new_jar() jsdom.Jar
pub fn jsdom.host_of(str) str
pub fn jsdom.split_set_cookie(str) List[str]
pub struct jsdom.Outcome
pub fn jsdom.run(mut html.Doc, str, Map[str, str], Map[str, str], float, float, int) jsdom.Outcome [io mutates]
```

## page
```
pub struct page.Style
pub struct page.Run
pub struct page.Box
pub struct page.Page
pub fn page.input_type(html.Doc, int) str
pub fn page.control_value(html.Doc, int) str
pub fn page.layout(html.Doc, str, int) page.Page
pub fn page.layout_with(html.Doc, str, int, css.Sheet) page.Page
pub fn page.quote(str) str
pub fn page.scheme(str) str
pub fn page.resolve(str, str) str
pub fn page.remove_dots(str) str
```

trimmed to fit 4000 tokens (~3790); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
