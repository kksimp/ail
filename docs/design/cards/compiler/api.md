# compiler: api

## api
```
pub api.API_VERSION := 1  ## The version of the output shape (api.out.1).
pub enum api.Form  ## How the outline is written.
    Plain  ## JSON on one line, the default.
    Pretty  ## JSON with a two-space indent (`--pretty`).
    Text  ## The compact outline (`--text`).
enum api.J  ## A JSON value whose objects keep their keys in the order given (api.out.2).
    Null
    Bool(b bool)
    Int(n int)
    Str(s str)
    Arr(items List[api.J])
    Obj(kvs List[api.Kv])
struct api.Kv
    key str
    val api.J
fn api.kv(key str, val api.J) api.Kv
fn api.opt_str(s str?) api.J
fn api.strs(xs List[str]) api.J
fn api.quote(s str) str
fn api.indent(depth int) str
fn api.write_json(j api.J, pretty bool, depth int) str
struct api.Item  ## A declaration in the outline, before it is written as JSON or text.
    line int
    col int
    kind str
    name str
    id str
    doc str?
    signature str
    extra List[api.Kv]  ## Keys after `signature`, in order.
    members List[api.Group]  ## Fields, variants or methods, each list under its key.
    fn api.Item.json(self) api.J
    fn api.Item.extra_strs(self, key str) List[str]  ## The strings of the array under `key` in `extra`.
    fn api.Item.has_extra(self, key str) bool
    fn api.Item.text(self, depth int, mut out List[str]) [mutates]
    fn api.Item.pos_key(self) int
struct api.Group
    key str
    items List[api.Item]
fn api.pad_left(n int, width int) str
fn api.spaces(n int) str
pub struct api.Facts  ## What the checker knows about the program, besides its declarations.
    pub effects List[List[str]]
    pub errors Map[int, List[str]]
struct api.Api
    prog ast.Program
    facts api.Facts
    fn_ids List[str]  ## The id of each function, by index (api.id.1-2).
    fn api.Api.fn_item(self, i int) api.Item
    fn api.Api.in_generated(self, im ast.ImplDecl, pos ast.Pos) bool  ## Whether `pos` is in the code a generator wrote into `im`.
    fn api.Api.impl_methods(self, im ast.ImplDecl) List[api.Item]
    fn api.Api.items(self) List[api.Item]  ## Every declaration of the program (not the prelude), nested: fields,
fn api.owner_name(prog ast.Program, f ast.FnDecl) str?
fn api.fn_id(prog ast.Program, f ast.FnDecl, qualified bool) str  ## `fn:name`, `method:Type.name`, `method:Trait.name`, or with `qualified`,
fn api.is_top(f ast.FnDecl) bool
fn api.is_impl(f ast.FnDecl) bool
fn api.new_api(prog ast.Program, facts api.Facts) api.Api
fn api.outside(pos ast.Pos) bool
fn api.receiver_text(r ast.Receiver) str
fn api.generics_text(gs List[ast.GenericParam]) str
fn api.owner_text(o ast.Owner) str
fn api.type_text(t ast.TypeRef) str  ## A type as written in source (type.opt.2, err.type.1-2, err.type.5, trait.any.1).
fn api.fn_signature(f ast.FnDecl) str  ## `fn name[T Ord](a int, mut b List[T]) T ! E`, as written; a top-level
fn api.char_text(c str) str
fn api.const_value(e ast.Expr) str  ## A constant's value when it is a literal, else `...`: the outline leaves
fn api.float_debug(x float) str  ## A float as Rust's `{:?}` writes it: `1.0`, `0.5`, `1e21`, `1e-7`.
fn api.field_item(owner str, f ast.Field) api.Item
fn api.variant_item(owner str, v ast.Variant) api.Item
fn api.type_extra(is_error bool, uniq bool) List[api.Kv]
fn api.type_prefix(is_error bool, uniq bool) str
fn api.cond_texts(cs List[ast.Contract], post bool) List[str]
fn api.trait_index(f ast.FnDecl) int?
fn api.receiver_of(f ast.FnDecl) str?
fn api.param_fields(p ast.Param) List[api.Kv]
fn api.take(mut methods Map[str, List[api.Item]], name str) List[api.Item] [mutates]
pub fn api.outline(prog ast.Program, facts api.Facts, path str, form api.Form) str  ## The outline of `prog` for the file `path`, in `form`; `symbols` and
```
