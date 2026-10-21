# compiler: check

## check
```
pub struct check.TParam
    pub name str
    pub bounds List[int]
pub struct check.Scoped
    pub name str
    pub id int
pub struct check.ImplInfo
    pub tr int?
    pub target check.Ty
    pub params List[int]
pub struct check.SigParam
    pub mutable bool
    pub ty check.Ty
pub struct check.Sig
    pub receiver bool?
    pub params List[check.SigParam]
    pub moves List[bool]
    pub ret check.Ty
    pub generics List[int]
    pub owner int
    pub declared int
pub struct check.Deferred
    pub ty check.Ty
    pub tr int
    pub pos ast.Pos
    pub what str
pub struct check.Checker
    pub prog ast.Program
    pub diags List[check.Diag]
    pub types Map[str, check.Def]
    pub traits Map[str, int]
    pub tparams List[check.TParam]
    pub struct_params List[List[int]]
    pub enum_params List[List[int]]
    pub list_param int
    pub trait_self List[int]
    pub supers List[List[int]]
    pub impl_info List[check.ImplInfo]
    pub impl_of Map[str, int]
    pub std_eq int
    pub std_ord int
    pub std_to_str int
    pub std_hash int
    pub std_ordering int
    pub std_drop int?
    pub field_types List[List[check.Ty]]
    pub variant_types List[List[List[check.Ty]]]
    pub fns Map[str, int]
    pub methods Map[str, int]
    pub sigs List[check.Sig]
    pub fn_scopes List[List[check.Scoped]]
    pub fn_self List[check.Ty?]
    pub inferred List[List[check.Ty]]
    pub consts Map[str, int]
    pub scope List[check.Scoped]
    pub self_ty check.Ty?
    pub deferring bool
    pub deferred List[check.Deferred]
pub fn check.new_checker(p ast.Program) check.Checker
pub fn check.check_declarations(p ast.Program) List[check.Diag]
pub check.COVERED_RULES := ...
pub struct check.Skip
    pub file str
    pub first int
    pub last int
pub fn check.skipped(p ast.Program) List[check.Skip]
pub struct check.Note
    pub pos ast.Pos
    pub message str
pub struct check.Diag
    pub rule str
    pub pos ast.Pos
    pub message str
    pub help str?
    pub notes List[check.Note]
    pub warning bool
    pub fn check.Diag.to_json(self, files List[ast.SourceFile]) str
pub fn check.new_diag(rule str, pos ast.Pos, message str) check.Diag
pub check.PRELUDE_FNS := ...
pub check.BUILTIN_TYPES := ...
pub check.SPEC_TYPES_NOT_YET := ...
pub check.GENERATABLE := ...
pub check.SIZED := ...
pub check.CONC_TYPES := ...
pub check.KNOWN_ATTRS := ...
pub check.STD_MODULES := ...
pub check.AIL_STD_MODULES := ...
pub check.PRELUDE_LINE := 1073741824
pub fn check.is_prelude_line(line int) bool
pub fn check.list_names(names List[str]) str
pub fn check.count(n int, noun str) str
pub fn check.was_were(n int) str
pub fn check.suggest(name str, candidates List[str]) str?
pub fn check.edit_distance(a str, b str) int
pub fn check.foreign_type_word(name str) str?
pub enum check.Ty
    Int
    Num(name str)
    Float
    Str
    Bytes
    Char
    Bool
    Unit
    Never
    Unknown
    Struct(index int, args List[check.Ty])
    Enum(index int, args List[check.Ty])
    ListOf(elem check.Ty)
    MapOf(key check.Ty, value check.Ty)
    SetOf(elem check.Ty)
    Opt(inner check.Ty)
    Res(ok check.Ty, errs check.Errs)
    Union(members List[check.Ty])
    Param(id int)
    AnyOf(trait_index int)
    RefOf(inner check.Ty)
    PtrOf(inner check.Ty)
    Conc(kind str, inner check.Ty)
    FnTy(params List[check.Ty], ret check.Ty)
pub enum check.Errs
    ErrSet(types List[check.Ty])
    Inferred(fn_index int)
pub enum check.Def
    DStruct(index int)
    DEnum(index int)
pub struct check.Bind
    pub param int
    pub ty check.Ty
pub fn check.same(a check.Ty, b check.Ty) bool
pub fn check.has_unknown(t check.Ty) bool
pub fn check.ty_key(t check.Ty) int
pub fn check.sort_errors(ts List[check.Ty]) List[check.Ty]
pub fn check.subst(t check.Ty, binds List[check.Bind]) check.Ty
pub fn check.subst_all(ts List[check.Ty], binds List[check.Bind]) List[check.Ty]
pub fn check.mentions(t check.Ty, p int) bool
pub fn check.con_of(t check.Ty) str?
pub fn check.args_of(t check.Ty) List[check.Ty]
```

trimmed to fit 2000 tokens (~1317); dropped: private items, doc comments
