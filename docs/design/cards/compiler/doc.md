# compiler: doc

## doc
```
pub fn doc.trimmed_line(raw str) str  ## The source line without a trailing `\r` and leading blanks.
pub fn doc.doc_text(line str) str  ## The text of a `##` line after the marker: one space dropped, trailing blanks gone.
pub fn doc.by_line(src str) Map[int, str]  ## The doc comment ending on the line before each line, by that line's number.
fn doc.doc_at(docs Map[int, str], pos ast.Pos) str?  ## The doc of the declaration at `pos`, unless it is in the prelude.
pub fn doc.attach(prog ast.Program, src str) ast.Program  ## Fill the `doc` of every declaration in the program (not the prelude)
pub fn doc.attach_lines(prog ast.Program, docs Map[int, str]) ast.Program  ## Like `attach` for docs already keyed by program line, as in a package of
pub struct doc.DocTest  ## One example.
    pub name str  ## `doc:Type.method` (test.doc.3).
    pub fn_name str  ## The generated function's name.
    pub expected List[str]  ## The lines `# => text` asked for (test.doc.2); `has_expected` is whether there are any.
    pub has_expected bool
    pub code List[doc.Line]  ## The code with its line in the original file.
    pub fence int  ## The line of the opening fence.
pub struct doc.Line
    pub no int
    pub text str
fn doc.declared(prog ast.Program) Map[int, str]  ## The name of each documented declaration, by its line.
fn doc.add(mut names Map[int, str], pos ast.Pos, name str) [mutates]
pub fn doc.collect(src str, prog ast.Program) List[doc.DocTest]  ## The ```` ```ail ```` blocks of every doc comment, in source order
fn doc.blocks(run List[doc.Line]) List[doc.DocTest]  ## The runnable blocks of one doc comment.
pub struct doc.Augmented  ## The source with each example appended as a test (test.doc.1).
    pub text str
    pub origin List[int]  ## The original line of every line of `text`; 0 for the original's own.
pub fn doc.augment(src str, tests List[doc.DocTest]) doc.Augmented
pub fn doc.check_output(t doc.DocTest, output str) str  ## The text a doc example's output must equal, or the failure (test.doc.2).
fn doc.debug_list(xs List[str]) str  ## A list of strings as Rust's `{:?}` writes it.
pub fn doc.expected_lines(t doc.DocTest) List[int]  ## The lines of its `# =>` expectations (snap.update.4).
pub fn doc.line_of(t doc.DocTest) int  ## Where a failing example is reported: its opening fence.
```
