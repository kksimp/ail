# garlic: garlicjs.syntax
tests: 3 (`ail test` lists them)

## garlicjs.syntax
```
pub struct garlicjs.syntax.Pos  ## Where a node starts: line and column, from 1, and the offset into the
pub struct garlicjs.syntax.Program
pub enum garlicjs.syntax.Stmt
pub struct garlicjs.syntax.VarDecl  ## `var`, `let` or `const` and its declarators.
pub struct garlicjs.syntax.Declarator
pub enum garlicjs.syntax.ForInit
pub enum garlicjs.syntax.ForLeft
pub struct garlicjs.syntax.Case
pub enum garlicjs.syntax.Expr
pub struct garlicjs.syntax.Quasi  ## One literal piece of a template; `cooked` is none for an invalid escape
pub enum garlicjs.syntax.Prop
pub enum garlicjs.syntax.Key  ## A property name (§13.2.5 PropertyName).
pub enum garlicjs.syntax.Pat  ## A binding or assignment target (§8.6.2, §13.15.5).
pub struct garlicjs.syntax.PatProp
pub struct garlicjs.syntax.Func  ## A function, method, arrow or getter.
pub struct garlicjs.syntax.Class
pub struct garlicjs.syntax.ClassMember  ## A class element (§15.7).
pub fn garlicjs.syntax.expr_pos(garlicjs.syntax.Expr) garlicjs.syntax.Pos
pub fn garlicjs.syntax.pat_pos(garlicjs.syntax.Pat) garlicjs.syntax.Pos
pub fn garlicjs.syntax.expr_kind(garlicjs.syntax.Expr) str  ## The variant's name, to test a node's kind in one line.
pub fn garlicjs.syntax.pat_kind(garlicjs.syntax.Pat) str
pub fn garlicjs.syntax.expr_suspends(garlicjs.syntax.Expr) bool  ## Whether `yield` or `await` occurs in an expression, outside nested
pub fn garlicjs.syntax.any_suspends(List[garlicjs.syntax.Expr]) bool
pub fn garlicjs.syntax.pat_suspends(garlicjs.syntax.Pat) bool
pub fn garlicjs.syntax.stmt_suspends(garlicjs.syntax.Stmt) bool  ## Whether a statement may suspend: `yield` or `await` in it, outside
pub fn garlicjs.syntax.block_suspends(List[garlicjs.syntax.Stmt]) bool
pub fn garlicjs.syntax.program_json(garlicjs.syntax.Program, bool) str
pub fn garlicjs.syntax.canonical_json(garlicjs.syntax.Program) str  ## The canonical form, to compare two scripts that differ only in grouping.
pub error struct garlicjs.syntax.JsError  ## A SyntaxError: what is wrong and where, 1-based.
pub impl ToStr for garlicjs.syntax.JsError
pub enum garlicjs.syntax.Kind
pub struct garlicjs.syntax.Token  ## A token.
pub struct garlicjs.syntax.Lexer
pub fn garlicjs.syntax.new_lexer(str) garlicjs.syntax.Lexer
pub fn garlicjs.syntax.is_line_term(char) bool  ## Line terminators (§12.3): LF, CR, LS, PS.
pub fn garlicjs.syntax.id_start(char) bool  ## IdentifierStart (§12.7): ID_Start, `$`, `_`.
pub fn garlicjs.syntax.id_part(char) bool  ## IdentifierPart (§12.7): ID_Continue, `$`, ZWNJ, ZWJ.
pub fn garlicjs.syntax.tokenize(str) List[garlicjs.syntax.Token] ! garlicjs.syntax.JsError [fail]  ## Every token of `source`, with a `/` read as division after a value and
pub fn garlicjs.syntax.token_line(garlicjs.syntax.Token) str  ## A token as one line of `--tokens` output: `LINE:COL KIND text`.
pub struct garlicjs.syntax.Parser
pub fn garlicjs.syntax.parse(str) garlicjs.syntax.Program ! garlicjs.syntax.JsError [fail]  ## Parse a script (§16.1 Script).
pub fn garlicjs.syntax.number_to_string(float) str  ## Number::toString(x) in base 10 (ECMA-262 §6.1.6.1.20).
pub fn garlicjs.syntax.quote(str) str  ## `s` as a JSON string, quotes included.
pub fn garlicjs.syntax.in_ranges(List[int], int) bool  ## Whether `code` is in one of the ranges, by binary search.
```

trimmed to fit 2000 tokens (~1206); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
