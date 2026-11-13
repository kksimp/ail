# compiler: derive

## derive
```
pub fn derive.derive_to_json(t derive.TypeInfo) str ! derive.DeriveError [fail]  ## A struct becomes an object of its fields in declaration order; an enum
pub fn derive.json_members(fields List[derive.FieldInfo], prefix str, indent str, first str) str  ## `[JsonMember(...), ...]` for `fields` after the member text `first`, one
pub fn derive.json_write(type_text str, e str, depth int) str  ## An expression giving the `Json` of `e`, a `type_text`: lists and maps
pub fn derive.derive_from_json(t derive.TypeInfo) str ! derive.DeriveError [fail]  ## A struct reads each field from the member of its name; a field `T?` may
pub fn derive.json_fields(fields List[derive.FieldInfo], indent str) str ! derive.DeriveError [fail]  ## Statements reading each of `fields` from the object `j` into `v_NAME`.
pub fn derive.json_read(type_text str, src str, dest str, path str, indent str, depth int) str ! derive.DeriveError [fail]  ## Statements at `indent` that read a `type_text` from the `Json` expression
pub fn derive.json_tuple_parts(type_text str) List[str]?  ## The element types of a tuple type text `(A, B)`, split at the commas
pub fn derive.json_optional(inner str, src str, dest str, path str, indent str, depth int) str ! derive.DeriveError [fail]  ## Statements reading an optional `inner?` from the `Json?` expression `src`.
pub fn derive.derive_default(t derive.TypeInfo) str ! derive.DeriveError [fail]  ## A struct of each field's default (trait.assoc.1); an enum has no
pub fn derive.default_of(type_text str) str  ## The default value of a field's `type_text`.
pub fn derive.derive_from_args(t derive.TypeInfo) str ! derive.DeriveError [fail]  ## A `bool` field is a switch, `T?` of `str`, `int` or `float` an option, a
pub struct derive.TypeInfo  ## One struct or enum as declared (refl.info.1).
    pub name str
    pub id str
    pub module str
    pub kind derive.TypeKind
    pub is_error bool
    pub params List[str]
    pub fields List[derive.FieldInfo]
    pub variants List[derive.VariantInfo]
pub enum derive.TypeKind
    Struct
    Enum
pub struct derive.FieldInfo
    pub name str
    pub type_text str
    pub public bool
    pub doc str
pub struct derive.VariantInfo
    pub name str
    pub fields List[derive.FieldInfo]
pub error struct derive.DeriveError  ## Why a generator can't derive its trait for a type (ct.derive.4).
    pub message str
pub struct derive.Expansion  ## The text a generator wrote for one `impl`.
    pub module str  ## The `impl`'s module, empty for the root.
    pub label str  ## `TYPE.TRAIT`
    pub file int  ## The file of the `impl` and where it is, for a message.
    pub pos ast.Pos
    pub text str
    pub problem str  ## Why the generator refused the type; empty when `text` is the methods.
pub fn derive.expand(modules List[str], progs List[ast.Program]) List[derive.Expansion]  ## Every deriving `impl` of the program: `modules[k]` is the module of
pub fn derive.derive_default_skipping(t derive.TypeInfo, defaulted List[str]) str ! derive.DeriveError [fail]  ## `derive_default` for a struct whose fields may declare defaults
pub fn derive.show_type(t ast.TypeRef) str  ## A type as written, as `ail api` prints it.
pub struct derive.Job  ## One deriving `impl` of a trait the program declares with `@derive(f)`
    pub module str  ## The `impl`'s module, empty for the root.
    pub label str  ## `TYPE.TRAIT`
    pub file int  ## The file of the `impl` and where it is, for a message.
    pub pos ast.Pos
    pub generator str
    pub trait_module str  ## The module of the trait, where the generator is called from.
    pub info str  ## The `TypeInfo(...)` expression for the type (refl.info.1).
pub fn derive.user_jobs(modules List[str], progs List[ast.Program]) List[derive.Job]  ## The deriving `impl`s of user traits. An `impl` whose trait has no
pub fn derive.tidy(text str) str  ## `text` as the bootstrap's formatter leaves it for the shapes generators
pub struct derive.ObserveFacts  ## The fields of struct `name` marked `@observe`, in order; the variant names
    pub fields List[str]
    pub enums Map[str, List[str]]
    pub tools List[derive.ToolFact]
pub struct derive.ToolFact  ## A `@tool` method (attach.tool.1): its name, doc, parameters and result.
    pub name str
    pub doc str
    pub params List[str]
    pub types List[str]
    pub ret str  ## The success type as written, "" for none.
    pub fallible bool  ## Whether the method can fail (`!`).
pub fn derive.observe_facts(progs List[ast.Program], name str) derive.ObserveFacts
pub fn derive.derive_observe(t derive.TypeInfo, facts derive.ObserveFacts) str ! derive.DeriveError [fail]
```

trimmed to fit 2000 tokens (~1535); dropped: private items
