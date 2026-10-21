# garlic: garlicjs.vm
tests: 3 (`ail test` lists them)

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

trimmed to fit 2000 tokens (~1887); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
