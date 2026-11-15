# compiler: fix

## fix
```
pub struct fix.Edit
    pub pos ast.Pos
    pub end ast.Pos
    pub old str?
    pub anchor str
    pub text str
pub struct fix.Fix
    pub title str
    pub edits List[fix.Edit]
pub fn fix.replace(pos ast.Pos, old str, text str) fix.Edit  ## Replace the first whole-word occurrence of `old` at or after `pos` on its line.
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
pub fn fix.operands(pos ast.Pos, op str, template str) fix.Edit  ## Rewrite `l op r` around the operator at `pos`; `text` is a template: `$l` `$r`
pub fn fix.c_for(pos ast.Pos) fix.Edit  ## `for (i := a; i < b; i += 1)` as `for i in a..b`, from the parenthesis at `pos`.
pub fn fix.empty_decl(pos ast.Pos) fix.Edit  ## `xs: List[int] = []` as `xs := List[int]()`, from the statement start.
pub fn fix.rename_all(pos ast.Pos, old str, text str) fix.Edit  ## Rename every use of the name `old` in the file, from the declaration at `pos`, to `text`.
pub fn fix.char_literal(pos ast.Pos) fix.Edit  ## The one-character string literal at `pos` as a character literal.
pub fn fix.range_list(pos ast.Pos) fix.Edit  ## `(a..b).to_list()` as a comprehension, from the method name at `pos`.
pub fn fix.new_fix(title str, edits List[fix.Edit]) fix.Fix
pub fn fix.resolve_char_literal(e fix.Edit, lines List[List[char]]) fix.Edit?  ## `"c"` at `e.pos` as `'c'`: a plain character or one of the escapes `\n`, `\t`, `\\`.
pub fn fix.resolve_range_list(e fix.Edit, lines List[List[char]]) fix.Edit?  ## `(a..b).to_list()` as `[i for i in a..b]`, from the method name at `e.pos`. The
pub fn fix.split_lines(src str) List[List[char]]  ## The source as lines of characters, as `settle` splits it (a final empty
pub fn fix.span_text(lines List[List[char]], pos ast.Pos, end ast.Pos) str  ## The text of the span `pos..end` (fix.edit.7): the characters between the two positions, the lines joined
pub fn fix.expects_of(fixes List[fix.Fix], lines List[List[char]]) List[List[str]]  ## The `expect` of every edit of every fix (fix.edit.7), computed where the lines are, so the lines are not passed on.
pub fn fix.token_end(lines List[List[char]], pos ast.Pos) ast.Pos  ## The exclusive end of the token starting at `pos`: a word, a number, a
pub fn fix.resolve(e fix.Edit, lines List[List[char]]) fix.Edit?  ## Settle one edit.
pub fn fix.settle_fixes(fixes List[fix.Fix], lines List[List[char]]) List[fix.Fix]  ## The fixes whose edits all settle, with the settled edits.
pub fn fix.is_word(c char) bool
pub fn fix.text_of(line List[char], a int, b int) str  ## `line[a..b]` as text.
pub fn fix.starts_at(line List[char], i int, pat List[char]) bool  ## Does `pat` occur in `line` at index `i`?
pub fn fix.atom_after(line List[char], from int) int?  ## End (exclusive) of the postfix expression that starts at `from`.
pub fn fix.piece(s str) str?  ## An operand as a piece of an interpolated string.
pub fn fix.resolve_operands(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_c_for(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_empty_decl(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.printf_text(args List[str]) str?  ## The arguments of a `printf(format, args...)` call as one interpolated
pub struct fix.Written  ## One line as it reads after a fix: its number and its text.
    pub line int
    pub text str
pub fn fix.written_lines(f fix.Fix, lines List[List[char]]) List[fix.Written]?  ## The lines that the settled fix `f` changes, as they read after it. `none` when
```

trimmed to fit 2000 tokens (~1697); dropped: private items
