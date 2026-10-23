# js
entry: main in main.ail
tests: 6 (`ail test` lists them)
ail.pkg:
    package garlicjs

## (root)
```
fn main() ! [env fail fs io]
fn usage() [io]
fn run(str, str, bool) str ! syntax.JsError [fail]
fn test_dir(str) !bool [fail fs io]
struct RunResult
fn run_script(str, int, bool) RunResult [io]
fn test_runs(str) !bool [fail fs io]
PAGE_URL := "https://example.test/dir/page.html"
fn run_page(str, int) List[str] [io]
fn run_page_at(str, int, str) List[str] [io]
fn test_pages(str) !bool [fail fs io]
struct Case
fn load(str) !List[Case] [fail fs io]
fn load_known(str) !Map[str, str] [fail fs io]
fn fingerprint(str) str
fn run_suite(str, bool) !bool [fail fs io]
```

## syntax
```
pub struct syntax.Pos
pub struct syntax.Program
pub enum syntax.Stmt
pub struct syntax.VarDecl
pub struct syntax.Declarator
pub enum syntax.ForInit
pub enum syntax.ForLeft
pub struct syntax.Case
pub enum syntax.Expr
pub struct syntax.Quasi
pub enum syntax.Prop
pub enum syntax.Key
pub enum syntax.Pat
pub struct syntax.PatProp
pub struct syntax.Func
pub struct syntax.Class
pub struct syntax.ClassMember
pub fn syntax.expr_pos(syntax.Expr) syntax.Pos
pub fn syntax.pat_pos(syntax.Pat) syntax.Pos
pub fn syntax.expr_kind(syntax.Expr) str
pub fn syntax.pat_kind(syntax.Pat) str
pub fn syntax.expr_suspends(syntax.Expr) bool
pub fn syntax.any_suspends(List[syntax.Expr]) bool
pub fn syntax.pat_suspends(syntax.Pat) bool
pub fn syntax.stmt_suspends(syntax.Stmt) bool
pub fn syntax.block_suspends(List[syntax.Stmt]) bool
pub fn syntax.program_json(syntax.Program, bool) str
pub fn syntax.canonical_json(syntax.Program) str
pub error struct syntax.JsError
pub impl ToStr for syntax.JsError
pub enum syntax.Kind
pub struct syntax.Token
pub struct syntax.Lexer
pub fn syntax.new_lexer(str) syntax.Lexer
pub fn syntax.is_line_term(char) bool
pub fn syntax.id_start(char) bool
pub fn syntax.id_part(char) bool
pub fn syntax.tokenize(str) List[syntax.Token] ! syntax.JsError [fail]
pub fn syntax.token_line(syntax.Token) str
pub struct syntax.Parser
pub fn syntax.parse(str) syntax.Program ! syntax.JsError [fail]
pub fn syntax.number_to_string(float) str
pub fn syntax.quote(str) str
pub fn syntax.in_ranges(List[int], int) bool
```

## vm
```
pub fn vm.utf16_less(str, str) bool
pub fn vm.element_interface(str) str
pub fn vm.new_vm() vm.Vm [mutates]
pub fn vm.arg_at(List[vm.Value], int) vm.Value
pub fn vm.native_is_ctor(str) bool
pub fn vm.is_array(vm.Value) bool
pub fn vm.num_of(vm.Value) float?
pub fn vm.parse_int(str, int) float
pub fn vm.parse_float(str) float
pub fn vm.uri_encode(str, str) str
pub fn vm.iso_date(float) str
pub fn vm.parse_date(str) float
pub struct vm.DomNode
pub struct vm.DomAttr
pub struct vm.Mutation
pub struct vm.HostRequest
pub struct vm.Listener
pub struct vm.Timer
pub fn vm.attr_of_node(vm.DomNode, str) str?
pub fn vm.node_id(vm.Value) int?
pub fn vm.collapse_space(str) str
pub fn vm.is_event_handler_name(str) bool
pub fn vm.form_encode(str) str
pub fn vm.url_part(str, str) str
pub fn vm.resolve_against(str, str) str
pub enum vm.Op
pub struct vm.Target
pub struct vm.Code
pub fn vm.compile_body(syntax.Func) vm.Code ! vm.Thrown [fail]
pub struct vm.Frame
pub struct vm.Completion
pub struct vm.Stop
pub struct vm.AgenRequest
pub fn vm.quote_js(str) str
pub struct vm.Vm
pub enum vm.Flow
pub vm.PRIVATE := "\u0000#"
pub fn vm.same_value_zero(vm.Value, vm.Value) bool
pub fn vm.init_binding(ref vm.Env, str, vm.Value, bool) [mutates]
pub fn vm.copy_env(ref vm.Env) ref vm.Env
pub fn vm.pattern_names(syntax.Pat) List[str]
pub fn vm.expr_text(syntax.Expr) str
pub fn vm.obj_of(vm.Value) ref vm.JsObject?
pub fn vm.str_of(vm.Value) str?
pub fn vm.well_known(str) str
pub vm.ITERATOR := "\u0000@w:Symbol.iterator"
pub vm.PI := 3.141592653589793
pub vm.LN2 := 0.6931471805599453
pub vm.LN10 := 2.302585092994046
pub fn vm.is_nan(float) bool
pub fn vm.is_finite(float) bool
pub fn vm.nan() float
pub fn vm.infinity() float
pub fn vm.fmod(float, float) float
pub fn vm.pow2(int) float
pub fn vm.exponent(float) int
pub fn vm.ulp(float) float
pub fn vm.trunc(float) float
pub fn vm.exp(float) float
pub fn vm.log(float) float
pub fn vm.log2(float) float
pub fn vm.log10(float) float
pub fn vm.pow(float, float) float
pub fn vm.sin(float) float
pub fn vm.cos(float) float
pub fn vm.tan(float) float
pub fn vm.atan(float) float
pub fn vm.atan2(float, float) float
pub fn vm.asin(float) float
pub fn vm.acos(float) float
pub fn vm.expm1(float) float
pub fn vm.log1p(float) float
pub fn vm.sinh(float) float
pub fn vm.cosh(float) float
pub fn vm.tanh(float) float
pub fn vm.cbrt(float) float
pub fn vm.decode_entities(str) str
pub struct vm.MoTarget
pub struct vm.Observer
pub vm.MO_MARK := "\u0000mutation-observers"
pub fn vm.to_fixed(float, int) str
pub fn vm.to_exponential(float, int?) str
pub fn vm.to_precision(float, int) str
pub fn vm.to_radix(float, int) str
pub fn vm.is_regexp(vm.Value) bool
pub fn vm.expand_replacement(str, str, str, str, List[str?]) str
pub fn vm.locale_number(float) str
pub fn vm.normalize_url(str) str
pub fn vm.form_decode(str) str
pub enum vm.Value
pub struct vm.Prop
pub struct vm.JsObject
pub enum vm.Callable
pub struct vm.ClassInfo
pub struct vm.Binding
pub struct vm.Env
pub error struct vm.Thrown
pub impl ToStr for vm.Thrown
pub fn vm.new_env(ref vm.Env?, bool) ref vm.Env
pub fn vm.data(vm.Value) vm.Prop
pub fn vm.hidden(vm.Value) vm.Prop
pub fn vm.is_obj(vm.Value) bool
pub fn vm.is_nullish(vm.Value) bool
pub fn vm.is_callable(vm.Value) bool
pub vm.SYMBOL := "\u0000@"
pub fn vm.is_symbol(str) bool
pub fn vm.symbol_description(str) str?
pub fn vm.symbol_text(str) str
pub fn vm.type_of(vm.Value) str
pub fn vm.truthy(vm.Value) bool
pub fn vm.string_to_number(str) float
pub fn vm.js_space(char) bool
pub fn vm.trim_js(str) str
pub fn vm.int_of(float) int
pub fn vm.as_index(float) int?
pub fn vm.index_of_key(str) int?
pub fn vm.to_int32(float) int
pub fn vm.to_uint32(float) int
pub fn vm.units(str) List[int]
pub fn vm.from_units(List[int]) str
pub fn vm.unit_len(str) int
```

trimmed to fit 4000 tokens (~2265); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
