# compiler: api

## api
```
pub fn api.ask_command(args List[str], std str) int [clock env fs io proc]  ## `ail ask PROGRAM QUESTION...`; the exit status: 0 answered, 1 nothing
pub fn api.estimate_tokens(s str) int  ## A cheap token estimate (api.card.3): a run of letters or digits costs one
pub fn api.card_command(rest List[str], std str) int [clock env fs io proc]  ## `ail api --card ...`; the exit status. `rest` holds the arguments, `--card`
pub fn api.context_command(args List[str], std str) int [clock env fs io proc]  ## `ail context PROGRAM SYMBOL [--budget N] [--json]`; the exit status: 0 the
pub fn api.public_decls(prog ast.Program, facts api.Facts, whole bool) List[api.Decl]  ## The public declarations of `prog` (api.diff.2).
pub fn api.render_diff(old str, new str, a List[api.Decl], b List[api.Decl], json bool, budget int) (str, int)  ## The report of `a` (OLD) against `b` (NEW), and how many declarations
pub fn api.diff_command(args List[str], std str) int [clock env fs io proc]  ## `ail api --diff ...` with its arguments after `--diff`; the exit status:
pub fn api.errors_command(rest List[str], std str) int [clock env fs io proc]
pub struct api.Impact  ## What an edit touches.
    pub changed List[str]  ## Declarations whose text changed, or that are new.
    pub affected List[str]  ## Functions that are, or reach, a changed declaration.
    pub tests List[str]  ## The affected `@test` functions, in source order.
    pub effects List[api.EffectChange]  ## Functions whose effects differ, with the effects before and after.
    pub untested List[str]  ## Changed functions that no `@test` function reaches (ADR 1024), in source order.
    pub via List[(str, str)]  ## For each affected function, the declaration it names or calls on the way to
    pub reaches List[(str, List[str])]  ## For each affected test, the changed declarations it reaches.
    pub partial bool  ## The new version has check errors, so the impact is read from the graph as far as it resolves.
pub struct api.EffectChange
    pub id str
    pub old List[str]
    pub new List[str]
pub fn api.impact_between(old load.Loaded, old_analysis check.Analysis, new load.Loaded, new_analysis check.Analysis) api.Impact  ## What changed between `old` and `new` can reach (api.impact.1-3).
pub fn api.impact_text(i api.Impact) str  ## The impact as text lines (api.impact.4).
pub fn api.impact_json(i api.Impact) str  ## The impact as a JSON object (api.impact.4).
pub api.API_VERSION := 1  ## The version of the output shape (api.out.1).
pub enum api.Form  ## How the outline is written.
    Plain  ## JSON on one line, the default.
    Pretty  ## JSON with a two-space indent (`--pretty`).
    Text  ## The compact outline (`--text`).
pub struct api.Facts  ## What the checker knows about the program, besides its declarations.
    pub effects List[List[str]]
    pub errors Map[int, List[str]]
    pub symbols Map[int, str]
    pub weak_symbols Map[int, str]
pub fn api.outline(prog ast.Program, facts api.Facts, path str, form api.Form, symbols bool) str  ## The outline of `prog` for the file `path`, in `form`; `symbols` and
pub fn api.outline_expanded(prog ast.Program, facts api.Facts, texts List[str], path str, form api.Form, symbols bool) str  ## `outline` with `--expanded` (ct.gen.4); `texts` are the program's file sources.
pub fn api.package_outline(prog ast.Program, facts api.Facts, dir str, form api.Form, symbols bool) str  ## The outline of the package `dir` in `form`.
pub fn api.package_command(dir str, form api.Form, std str, symbols bool) int [clock env fs io proc]  ## `ail api DIR`; the exit status. Doc comments are attached file by file.
pub fn api.query_command(rest List[str], std str) int [clock env fs io proc]  ## `ail api --locals|--type|--refs [DIR] FILE LINE COL [--pretty]`; the exit
pub struct api.Ref  ## One use: the id of the declaration and where the name is written.
    pub id str
    pub pos ast.Pos
    pub within int  ## The index of the function whose body or signature has it, `-1` outside one.
    pub label bool  ## A field named by a label in a struct literal or variant (api.ask.2).
pub fn api.references(prog ast.Program, symbols Map[int, str], weak Map[int, str], own_file bool) List[api.Ref]  ## Every reference in `prog`, by position then id, without the prelude and
pub struct api.Found  ## Every reference of `prog` unsorted, without the prelude, and the value
    pub refs List[api.Ref]
    pub names List[(ast.Pos, str)]
pub fn api.collect_refs(prog ast.Program, symbols Map[int, str], weak Map[int, str]) api.Found
pub fn api.review_command(args List[str], std str) int [clock env fs io proc]  ## `ail review ARGS`; the exit status: 0 the report was printed, 1 with `--deny-new`
pub fn api.semantic_diff_command(args List[str], std str) int [clock env fs io proc]  ## `ail diff ARGS`; the exit status: 0 when nothing changed, 1 when something did, 2 for an error.
```

trimmed to fit 2000 tokens (~1657); dropped: private items
