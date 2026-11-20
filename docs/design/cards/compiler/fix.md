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
    pub sure bool
pub fn fix.replace(pos ast.Pos, old str, text str) fix.Edit
pub fn fix.insert(pos ast.Pos, text str) fix.Edit
pub fn fix.at_line_end(pos ast.Pos, text str) fix.Edit
pub fn fix.after_expr(pos ast.Pos, text str) fix.Edit
pub fn fix.trim_before(pos ast.Pos) fix.Edit
pub fn fix.span(pos ast.Pos, end ast.Pos, text str) fix.Edit
pub fn fix.delete(pos ast.Pos, end ast.Pos) fix.Edit
pub fn fix.call(pos ast.Pos, callee str, template str) fix.Edit
pub fn fix.call_with(pos ast.Pos, callee str, template str) fix.Edit
pub fn fix.call_named(pos ast.Pos, callee str, names List[str]) fix.Edit
pub fn fix.chain(first ast.Pos, second ast.Pos) fix.Edit
pub fn fix.after_block(pos ast.Pos, text str) fix.Edit
pub fn fix.operands(pos ast.Pos, op str, template str) fix.Edit
pub fn fix.c_for(pos ast.Pos) fix.Edit
pub fn fix.empty_decl(pos ast.Pos) fix.Edit
pub fn fix.rename_all(pos ast.Pos, old str, text str) fix.Edit
pub fn fix.char_literal(pos ast.Pos) fix.Edit
pub fn fix.join_swap(pos ast.Pos) fix.Edit
pub fn fix.int_text(pos ast.Pos) fix.Edit
pub fn fix.range_list(pos ast.Pos) fix.Edit
pub fn fix.method_decl(pos ast.Pos, name str, ty str) fix.Edit
pub fn fix.method_call(pos ast.Pos, name str) fix.Edit
pub fn fix.unbracket_one(pos ast.Pos) fix.Edit
pub fn fix.unparen(pos ast.Pos) fix.Edit
pub fn fix.paren_index(pos ast.Pos) fix.Edit
pub fn fix.char_code(pos ast.Pos) fix.Edit
pub fn fix.new_fix(title str, edits List[fix.Edit]) fix.Fix
pub fn fix.sure_fix(title str, edits List[fix.Edit]) fix.Fix
pub fn fix.resolve_method_decl(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_method_call(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_unbracket_one(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_unparen(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_paren_index(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_char_code(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_char_literal(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_range_list(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.split_lines(src str) List[List[char]]
pub fn fix.span_text(lines List[List[char]], pos ast.Pos, end ast.Pos) str
pub fn fix.expects_of(fixes List[fix.Fix], lines List[List[char]]) List[List[str]]
pub fn fix.token_end(lines List[List[char]], pos ast.Pos) ast.Pos
pub fn fix.resolve(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.settle_fixes(fixes List[fix.Fix], lines List[List[char]]) List[fix.Fix]
pub fn fix.is_word(c char) bool
pub fn fix.text_of(line List[char], a int, b int) str
pub fn fix.starts_at(line List[char], i int, pat List[char]) bool
pub fn fix.atom_after(line List[char], from int) int?
pub fn fix.piece(s str) str?
pub fn fix.resolve_operands(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_c_for(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_empty_decl(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.printf_text(args List[str]) str?
pub fn fix.resolve_join_swap(e fix.Edit, lines List[List[char]]) fix.Edit?
pub fn fix.resolve_int_text(e fix.Edit, lines List[List[char]]) fix.Edit?
pub struct fix.Written
    pub line int
    pub text str
pub fn fix.written_lines(f fix.Fix, lines List[List[char]]) List[fix.Written]?
pub fix.LITERAL_BRACE_FIX := "escape the literal brace"
pub fix.CLOSING_BRACE_FIX := "remove the backslash"
pub fn fix.sure_syntax(rule str) bool
pub fn fix.sure_syntax_fix(rule str, title str) bool
```

trimmed to fit 2000 tokens (~1366); dropped: private items, doc comments
