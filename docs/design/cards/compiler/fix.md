# compiler: fix

## fix
```
pub enum fix.Anchor  ## How an edit finds its position when the checker doesn't name a span.
    Exact  ## `pos..end` is the span (`old`, when set, is the whole word to find on the line).
    LineEnd  ## The end of the code on `pos.line`.
    AfterExpr  ## Just after the expression that starts at `pos`.
    TrimBefore  ## The character at `pos` and the blanks before it.
    Call  ## Rewrite the call `old(...)` at `pos` by the template in `text`.
    Brackets  ## Rewrite `old[args]` by the template in `text`.
    CallNamed  ## Name the arguments of the call `old(...)`; `text` lists the names, comma-separated.
    Chain  ## Split a chained comparison between its operators at `pos` and `end`.
    AfterBlock  ## Append the lines of `text` after the block starting at `pos`.
    Operands  ## Rewrite the binary expression whose operator `old` is at `pos`, by the template in `text`.
    CFor  ## A C-style `for` becomes a range loop.
    EmptyDecl  ## `xs: List[int] = []` as `xs := List[int]()`.
pub struct fix.Edit
    pub pos ast.Pos
    pub end ast.Pos
    pub old str?
    pub anchor fix.Anchor
    pub text str
    pub fn fix.Edit.to_json(self) str  ## An edit as the JSON object `ail check --json` writes once its span is settled.
pub struct fix.Fix
    pub title str
    pub edits List[fix.Edit]
    pub fn fix.Fix.to_json(self) str  ## A fix as `{"title":..,"edits":[..]}`.
fn fix.at(pos ast.Pos, old str?, anchor fix.Anchor, text str) fix.Edit
pub fn fix.replace(pos ast.Pos, old str, text str) fix.Edit  ## Replace the first whole-word `old` at or after `pos` on its line by `text`.
pub fn fix.insert(pos ast.Pos, text str) fix.Edit  ## Insert `text` at `pos`.
pub fn fix.at_line_end(pos ast.Pos, text str) fix.Edit  ## Insert `text` at the end of the code on `pos.line`.
pub fn fix.after_expr(pos ast.Pos, text str) fix.Edit  ## Insert `text` just after the expression that starts at `pos`.
pub fn fix.trim_before(pos ast.Pos) fix.Edit  ## Delete the character at `pos` and the blanks before it.
pub fn fix.span(pos ast.Pos, end ast.Pos, text str) fix.Edit  ## Replace the span `pos..end` with `text`.
pub fn fix.delete(pos ast.Pos, end ast.Pos) fix.Edit  ## Delete the span `pos..end`.
pub fn fix.call(pos ast.Pos, callee str, template str) fix.Edit  ## Rewrite the call `callee(...)` at `pos` by a template.
pub fn fix.call_with(pos ast.Pos, callee str, template str) fix.Edit  ## Rewrite `callee[args]` by a template.
pub fn fix.call_named(pos ast.Pos, callee str, names List[str]) fix.Edit  ## Name the arguments of the call `callee(...)`.
pub fn fix.chain(first ast.Pos, second ast.Pos) fix.Edit  ## Split a chained comparison between its two operators.
pub fn fix.after_block(pos ast.Pos, text str) fix.Edit  ## Append the lines `text` (each ending in a newline) after the block starting at `pos`.
pub fn fix.operands(pos ast.Pos, op str, template str) fix.Edit  ## Rewrite the binary expression whose operator `op` is at `pos` by a template.
pub fn fix.c_for(pos ast.Pos) fix.Edit  ## A C-style `for` at `pos` as a range loop.
pub fn fix.empty_decl(pos ast.Pos) fix.Edit  ## `xs: List[int] = []` at `pos` as `xs := List[int]()`.
```
