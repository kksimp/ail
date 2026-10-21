# js: syntax
tests: 3 (`ail test` lists them)

## syntax
```
pub struct syntax.Pos  ## Where a node starts: line and column, from 1, and the offset into the
pub struct syntax.Program
pub enum syntax.Stmt
pub struct syntax.VarDecl  ## `var`, `let` or `const` and its declarators.
pub struct syntax.Declarator
pub enum syntax.ForInit
pub enum syntax.ForLeft
pub struct syntax.Case
pub enum syntax.Expr
pub struct syntax.Quasi  ## One literal piece of a template; `cooked` is none for an invalid escape
pub enum syntax.Prop
pub enum syntax.Key  ## A property name (§13.2.5 PropertyName).
pub enum syntax.Pat  ## A binding or assignment target (§8.6.2, §13.15.5).
pub struct syntax.PatProp
pub struct syntax.Func  ## A function, method, arrow or getter.
pub struct syntax.Class
pub struct syntax.ClassMember  ## A class element (§15.7).
pub fn syntax.expr_pos(syntax.Expr) syntax.Pos
pub fn syntax.pat_pos(syntax.Pat) syntax.Pos
pub fn syntax.expr_kind(syntax.Expr) str  ## The variant's name, to test a node's kind in one line.
pub fn syntax.pat_kind(syntax.Pat) str
pub fn syntax.expr_suspends(syntax.Expr) bool  ## Whether `yield` or `await` occurs in an expression, outside nested
pub fn syntax.any_suspends(List[syntax.Expr]) bool
pub fn syntax.pat_suspends(syntax.Pat) bool
pub fn syntax.stmt_suspends(syntax.Stmt) bool  ## Whether a statement may suspend: `yield` or `await` in it, outside
pub fn syntax.block_suspends(List[syntax.Stmt]) bool
pub fn syntax.program_json(syntax.Program, bool) str
pub fn syntax.canonical_json(syntax.Program) str  ## The canonical form, to compare two scripts that differ only in grouping.
pub error struct syntax.JsError  ## A SyntaxError: what is wrong and where, 1-based.
pub impl ToStr for syntax.JsError
pub enum syntax.Kind
pub struct syntax.Token  ## A token.
pub struct syntax.Lexer
pub fn syntax.new_lexer(str) syntax.Lexer
pub fn syntax.is_line_term(char) bool  ## Line terminators (§12.3): LF, CR, LS, PS.
pub fn syntax.id_start(char) bool  ## IdentifierStart (§12.7): ID_Start, `$`, `_`.
pub fn syntax.id_part(char) bool  ## IdentifierPart (§12.7): ID_Continue, `$`, ZWNJ, ZWJ.
pub fn syntax.tokenize(str) List[syntax.Token] ! syntax.JsError [fail]  ## Every token of `source`, with a `/` read as division after a value and
pub fn syntax.token_line(syntax.Token) str  ## A token as one line of `--tokens` output: `LINE:COL KIND text`.
pub struct syntax.Parser
pub fn syntax.parse(str) syntax.Program ! syntax.JsError [fail]  ## Parse a script (§16.1 Script).
pub fn syntax.number_to_string(float) str  ## Number::toString(x) in base 10 (ECMA-262 §6.1.6.1.20).
pub fn syntax.quote(str) str  ## `s` as a JSON string, quotes included.
pub fn syntax.in_ranges(List[int], int) bool  ## Whether `code` is in one of the ranges, by binary search.
```

trimmed to fit 2000 tokens (~1010); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
