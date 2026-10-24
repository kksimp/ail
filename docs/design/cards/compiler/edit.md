# compiler: edit

## edit
```
edit.USAGE := ...
pub fn edit.command(args List[str], std str) int [clock env fs io proc]  ## `ail edit PATH [--dry-run] [--diff] [--if-clean] [--test] [--json] < EDITS`
fn edit.plural(n int, w str) str
fn edit.is_dir(path str) bool [fs io]
fn edit.files_of(path str) List[str] [fs io]  ## The files of the program at `path`: the file, or every `.ail` file of a package.
fn edit.write_file(path str, text str) [fs io]  ## Write `text` to a temporary file beside `path` and rename it over, so a reader sees the old
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
struct edit.Reader
    cs List[char]
    at int
    err str
pub fn edit.parse_json(text str) edit.Parsed
fn edit.skip_ws(mut r edit.Reader) [mutates]
fn edit.fail(mut r edit.Reader, msg str) edit.J [mutates]
fn edit.read_value(mut r edit.Reader) edit.J [mutates]
pub struct edit.Work  ## The working copy: each file's text, keyed by path (in path order).
    pub root str
    pub package bool
    pub paths List[str]
    pub texts List[str]
    pub err str
    pub std str  ## The directory of the standard modules, for the check a rename needs.
    fn edit.Work.index(self, file str) int
    pub fn edit.Work.module(self, file str) str
    fn edit.Work.fail(mut self, msg str) [mutates]
    fn edit.Work.find(mut self, id str) edit.Hit [mutates]  ## The file and declaration an id names (edit.id.1-2); `file` is -1 when none.
    fn edit.Work.target(mut self, op edit.J) int [mutates]  ## The file an insert or import goes to (edit.op.3, edit.op.6); -1 on failure.
    fn edit.Work.declared_at(mut self, id str) edit.Spot [mutates]
struct edit.Hit  ## One declaration found by `find`: where, and which.
    file int
    decl edit.Decl
fn edit.bare_name(id str) str
fn edit.parent_of(path str) str
fn edit.str_op(op edit.J, key str) str
pub struct edit.Hunked  ## Apply a hunk (edit.op.7) to lines `from..to` of `src`: the `-` and context
    pub text str
    pub err str
fn edit.text_of(l str) str
pub fn edit.hunk(src str, from int, to int, lines List[str]) edit.Hunked
pub struct edit.Compact  ## The compact form of the edits (edit.in.3): hunks under an id line, and
    pub ops List[edit.J]
    pub err str
pub fn edit.parse_compact(input str) edit.Compact
fn edit.hunk_op(has_id bool, id str, lines List[str]) edit.J
fn edit.starts_directive(s str) bool
fn edit.first_word(l str) str
pub fn edit.apply(mut work edit.Work, op edit.J, path str) [clock env fs io mutates proc]  ## Apply one operation (edit.op.1-7); sets `work.err` on failure.
fn edit.need(mut work edit.Work, op edit.J, kind str, key str) str [mutates]
fn edit.text_at(cs List[char], i int, s str) bool  ## Whether `s` is at character `i` of `cs`.
fn edit.ident_char(c char) bool
fn edit.is_ident(s str) bool
fn edit.name_column(head str, d edit.Decl) int  ## The 1-based column of a declaration's name on its line, `0` when it isn't there.
struct edit.Spot  ## Where `id` is declared: the file, 1-based line and column, and the name.
    file int
    line int
    col int
    name str
fn edit.rename(mut work edit.Work, id str, to str) [clock env fs io mutates proc]  ## `{"op":"rename","id":ID,"to":NAME}` (edit.op.5): the declaration and every
pub struct edit.Decl  ## A top-level declaration found in a file's text (edit.id.1).
    pub id str
    pub name str  ## The bare name a rename changes.
    pub start int  ## 0-based: first line (doc comment and attributes included), the
    pub head int
    pub sig_end int
    pub end int
fn edit.word_char(c char) bool
pub fn edit.word(s str) str  ## The leading identifier characters of `s`.
fn edit.chars_len(s str) int
fn edit.skip(s str, n int) str  ## `s` without its first `n` characters.
pub fn edit.indent_of(line str) int  ## Leading spaces of a line.
fn edit.starts_blank(line str) bool
pub struct edit.Declared  ## The id and name a column-0 line declares, if any (edit.id.1); an empty id means none.
    pub id str
    pub name str
pub fn edit.declares(line str) edit.Declared
pub fn edit.decls(src str) List[edit.Decl]  ## The declarations of `src`, in order.
fn edit.starts_with_blank(l str) bool
fn edit.trailing_blank(line str) bool
pub fn edit.signature_end(lines List[str], i int) int  ## The last line of the signature that starts on line `i`: where its
pub fn edit.reindent(text str, by int) List[str]  ## `text` without its common indentation, indented by `by` spaces.
pub fn edit.splice(src str, from int, to int, new List[str]) str  ## `src` with lines `from..to` replaced by `new`.
pub fn edit.module_of(root str, file str) str  ## `module` is the module path of `file` inside package `root`: `geo` for `root/geo/a.ail`.
pub fn edit.qualified(id str, module str) str  ## `fn:origin` in module `geo` is `fn:geo.origin` (api.pkg.1).
```
