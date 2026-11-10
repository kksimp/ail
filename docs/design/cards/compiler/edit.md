# compiler: edit

## edit
```
pub fn edit.command(args List[str], std str) int [clock env fs io proc]  ## `ail edit PATH [--dry-run] [--diff] [--if-clean] [--test] [--json] < EDITS`
pub enum edit.J  ## The JSON of an edit list (edit.in.1): a small parser with the messages of
    Null
    Bool(b bool)
    Num(text str)
    Str(s str)
    Arr(xs List[edit.J])
    Obj(keys List[str], vals List[edit.J])
    pub fn edit.J.get(self, key str) edit.J?  ## The member of an object.
    pub fn edit.J.str_of(self, key str) str?  ## The string member of an object.
pub struct edit.Parsed  ## The result of parsing: `err` is empty when the parse worked.
    pub value edit.J
    pub err str
pub fn edit.parse_json(text str) edit.Parsed
pub struct edit.Work  ## The working copy: each file's text, keyed by path (in path order).
    pub root str
    pub package bool
    pub paths List[str]
    pub texts List[str]
    pub err str
    pub std str  ## The directory of the standard modules, for the check a rename needs.
    pub fn edit.Work.module(self, file str) str
pub struct edit.Hunked  ## Apply a hunk (edit.op.7) to lines `from..to` of `src`: the `-` and context
    pub text str
    pub err str
pub fn edit.hunk(src str, from int, to int, lines List[str]) edit.Hunked
pub struct edit.Compact  ## The compact form of the edits (edit.in.3): hunks under an id line, and
    pub ops List[edit.J]
    pub err str
pub fn edit.parse_compact(input str) edit.Compact
pub fn edit.apply(mut work edit.Work, op edit.J, path str) [clock env fs io mutates proc]  ## Apply one operation (edit.op.1-7); sets `work.err` on failure.
pub struct edit.Decl  ## A top-level declaration found in a file's text (edit.id.1).
    pub id str
    pub name str  ## The bare name a rename changes.
    pub start int  ## 0-based: first line (doc comment and attributes included), the
    pub head int
    pub sig_end int
    pub end int
pub fn edit.word(s str) str  ## The leading identifier characters of `s`.
pub fn edit.indent_of(line str) int  ## Leading spaces of a line.
pub struct edit.Declared  ## The id and name a column-0 line declares, if any (edit.id.1); an empty id means none.
    pub id str
    pub name str
pub fn edit.declares(line str) edit.Declared
pub fn edit.decls(src str) List[edit.Decl]  ## The declarations of `src`, in order.
pub fn edit.signature_end(lines List[str], i int) int  ## The last line of the signature that starts on line `i`: where its
pub fn edit.reindent(text str, by int) List[str]  ## `text` without its common indentation, indented by `by` spaces.
pub fn edit.splice(src str, from int, to int, new List[str]) str  ## `src` with lines `from..to` replaced by `new`.
pub fn edit.module_of(root str, file str) str  ## `module` is the module path of `file` inside package `root`: `geo` for `root/geo/a.ail`.
pub fn edit.qualified(id str, module str) str  ## `fn:origin` in module `geo` is `fn:geo.origin` (api.pkg.1).
```

trimmed to fit 2000 tokens (~1007); dropped: private items
