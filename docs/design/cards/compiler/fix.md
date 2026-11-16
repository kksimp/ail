# compiler: fix

## fix
```
pub struct fix.Edit
pub struct fix.Fix
pub fn fix.replace(ast.Pos, str, str) fix.Edit  ## Replace the first whole-word occurrence of `old` at or after `pos` on its line.
pub fn fix.insert(ast.Pos, str) fix.Edit  ## Insert `text` at `pos`.
pub fn fix.at_line_end(ast.Pos, str) fix.Edit  ## Insert `text` at the end of the code on `pos.line`.
pub fn fix.after_expr(ast.Pos, str) fix.Edit  ## Insert `text` just after the expression that starts at `pos`.
pub fn fix.trim_before(ast.Pos) fix.Edit  ## Delete the character at `pos` and the blanks before it.
pub fn fix.span(ast.Pos, ast.Pos, str) fix.Edit  ## Replace the span `pos..end` with `text`.
pub fn fix.delete(ast.Pos, ast.Pos) fix.Edit  ## Delete the span `pos..end`.
pub fn fix.call(ast.Pos, str, str) fix.Edit  ## Rewrite the call `callee(...)` at `pos` by a template.
pub fn fix.call_with(ast.Pos, str, str) fix.Edit  ## Rewrite `callee[args]` by a template.
pub fn fix.call_named(ast.Pos, str, List[str]) fix.Edit  ## Name the arguments of the call `callee(...)`.
pub fn fix.chain(ast.Pos, ast.Pos) fix.Edit  ## Split a chained comparison between its two operators.
pub fn fix.after_block(ast.Pos, str) fix.Edit  ## Append the lines `text` (each ending in a newline) after the block starting at `pos`.
pub fn fix.operands(ast.Pos, str, str) fix.Edit  ## Rewrite `l op r` around the operator at `pos`; `text` is a template: `$l` `$r`
pub fn fix.c_for(ast.Pos) fix.Edit  ## `for (i := a; i < b; i += 1)` as `for i in a..b`, from the parenthesis at `pos`.
pub fn fix.empty_decl(ast.Pos) fix.Edit  ## `xs: List[int] = []` as `xs := List[int]()`, from the statement start.
pub fn fix.rename_all(ast.Pos, str, str) fix.Edit  ## Rename every use of the name `old` in the file, from the declaration at `pos`, to `text`.
pub fn fix.char_literal(ast.Pos) fix.Edit  ## The one-character string literal at `pos` as a character literal.
pub fn fix.join_swap(ast.Pos) fix.Edit  ## `sep.join(xs)` as `xs.join(sep)`, from the method name at `pos`.
pub fn fix.range_list(ast.Pos) fix.Edit  ## `(a..b).to_list()` as a comprehension, from the method name at `pos`.
pub fn fix.new_fix(str, List[fix.Edit]) fix.Fix
pub fn fix.sure_fix(str, List[fix.Edit]) fix.Fix  ## A certain fix (fix.auto.1, ADR 1130).
pub fn fix.resolve_char_literal(fix.Edit, List[List[char]]) fix.Edit?  ## `"c"` at `e.pos` as `'c'`: a plain character or one of the escapes `\n`, `\t`, `\\`.
pub fn fix.resolve_range_list(fix.Edit, List[List[char]]) fix.Edit?  ## `(a..b).to_list()` as `[i for i in a..b]`, from the method name at `e.pos`.
pub fn fix.split_lines(str) List[List[char]]  ## The source as lines of characters, as `settle` splits it (a final empty
pub fn fix.span_text(List[List[char]], ast.Pos, ast.Pos) str  ## The text of the span `pos..end` (fix.edit.7): the characters between the two positions, the lines joined
pub fn fix.expects_of(List[fix.Fix], List[List[char]]) List[List[str]]  ## The `expect` of every edit of every fix (fix.edit.7), computed where the lines are, so the lines are not passed on.
pub fn fix.token_end(List[List[char]], ast.Pos) ast.Pos  ## The exclusive end of the token starting at `pos`: a word, a number, a
pub fn fix.resolve(fix.Edit, List[List[char]]) fix.Edit?  ## Settle one edit.
pub fn fix.settle_fixes(List[fix.Fix], List[List[char]]) List[fix.Fix]  ## The fixes whose edits all settle, with the settled edits.
pub fn fix.is_word(char) bool
pub fn fix.text_of(List[char], int, int) str  ## `line[a..b]` as text.
pub fn fix.starts_at(List[char], int, List[char]) bool  ## Does `pat` occur in `line` at index `i`?
pub fn fix.atom_after(List[char], int) int?  ## End (exclusive) of the postfix expression that starts at `from`.
pub fn fix.piece(str) str?  ## An operand as a piece of an interpolated string.
pub fn fix.resolve_operands(fix.Edit, List[List[char]]) fix.Edit?
pub fn fix.resolve_c_for(fix.Edit, List[List[char]]) fix.Edit?
pub fn fix.resolve_empty_decl(fix.Edit, List[List[char]]) fix.Edit?
pub fn fix.printf_text(List[str]) str?  ## The arguments of a `printf(format, args...)` call as one interpolated
pub fn fix.resolve_join_swap(fix.Edit, List[List[char]]) fix.Edit?  ## `sep.join(xs)` as `xs.join(sep)` (expr.name.1, str has no `join`): `e.pos` is the method name.
pub struct fix.Written  ## One line as it reads after a fix: its number and its text.
pub fn fix.written_lines(fix.Fix, List[List[char]]) List[fix.Written]?  ## The lines that the settled fix `f` changes, as they read after it.
pub fix.LITERAL_BRACE_FIX := "escape the literal brace"  ## The fixes of `lex.str.2` and `lex.str.1` that are certain, told apart by their title (the rest of each rule's
pub fix.CLOSING_BRACE_FIX := "remove the backslash"
pub fn fix.sure_syntax(str) bool  ## Whether the parser's fix for `rule` is certain.
pub fn fix.sure_syntax_fix(str, str) bool  ## Whether the parser fix titled `title` for `rule` is certain: the rule's, or one of the two string fixes above.
```

trimmed to fit 2000 tokens (~1869); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
