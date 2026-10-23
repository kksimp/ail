# compiler: fix

## fix
```
pub fn fix.settled(src str, dump str) str  ## One line per fix that settles: `LINE:COL:` of its diagnostic, then its edits.
pub fn fix.rust_diagnose(path str, src str, json bool) fix.Diagnosis [env fs io proc]  ## A check by the Rust tool, for `fixrun`: writes `src` to `path`, runs
pub fn fix.run_with_rust(args List[str]) int [env fs io proc]  ## `ail fix` with the Rust tool as the checker, for `tools/fix/parity.py`.
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
pub fn fix.new_fix(title str, edits List[fix.Edit]) fix.Fix
pub struct fix.Found  ## A diagnostic as `ail fix` sees it: its rule, position, whether it is a
    pub rule str
    pub pos ast.Pos
    pub warning bool
    pub fixes List[fix.Fix]
pub struct fix.Diagnosis  ## What a check of one file found: the diagnostics sorted by position with
    pub found List[fix.Found]
    pub report str
pub struct fix.Run  ## One `ail fix` run. The caller drives it, so this module needs neither the
    pub code int  ## Exit status once the run is over (`begin` failed), else -1.
    pub path str
    pub src str
    original str
    dry bool
    json bool
    canonical bool
    applied int
    rounds int
    out str
pub fn fix.begin(args List[str]) fix.Run [fs io]  ## Read the command line (after the command word) and the file.
pub fn fix.step(mut r fix.Run, d fix.Diagnosis) bool [mutates]  ## Apply the first fix of each diagnostic in `d`, which is the check of the
pub fn fix.canonicalize(mut r fix.Run) [mutates]  ## After the last round (fix.apply.3): a file that was canonical stays canonical.
pub fn fix.finish(mut r fix.Run, left fix.Diagnosis) int [fs io mutates]  ## Write the result (or show its diff) and print the summary; `left` is the
pub fn fix.split_lines(src str) List[List[char]]  ## The source as lines of characters, as `settle` splits it (a final empty
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
```

trimmed to fit 2000 tokens (~1699); dropped: private items
