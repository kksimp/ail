# compiler: load

## load
```
pub load.AUTO_ROUNDS := 5  ## Rounds of fix-and-check before the check reports what is left (fix.apply.2).
pub fn load.auto_fix_on(no_fix bool) bool [env]  ## Whether certain fixes are applied: not with `--no-fix`, nor when `AIL_FIX` is `0`
pub fn load.has_sure_fix(found List[load.Shown]) bool  ## Whether an error in `found` has a certain first fix.
pub fn load.auto_fix(files List[ast.SourceFile], found List[load.Shown], std str, skip_root bool) int [fs io]  ## Apply the certain first fix of each error in `found` (a check's diagnostics, at
pub fn load.certain_rounds(file str, std str) int [clock env fs io proc]  ## The certain fixes in the module files of the program rooted at `file`, round after round (at
pub fn load.effects_command(rest List[str], std str) int [clock env fs io proc]
pub fn load.fix_command(rest List[str], std str) int [clock env fs io proc]
pub struct load.Repair  ## What `check --fix` did (fix.check.1): the rules of the fixes applied, in order, and
    pub rules List[str]
    pub code int
pub fn load.fixed_line(rules List[str]) str  ## `fixed 3: expr.if.1, expr.name.1 x2` (fix.check.1): the rules in first-seen order with a count when more than one.
pub fn load.repair(path str, std str) load.Repair [clock env fs io proc]  ## Fixes `path` to a fixed point as `ail fix` does, writing it and printing nothing (fix.check.1).
pub fn load.header_command(rest List[str], std str) int [clock env fs io proc]
pub fn load.c_header(prog ast.Program, source str) str  ## The header text for the exported functions of `prog`, in source order.
pub load.FILE_LINES := 1048576  ## Each file's lines are numbered from a multiple of this (modules.rs).
pub load.PRELUDE_LINE := 1073741824  ## Lines from here on belong to the prelude and the standard types.
pub error struct load.LoadError  ## Why a program can't be loaded: a file that can't be read, or no files.
    pub message str
pub struct load.Loaded  ## A loaded program with the diagnostics its loading found.
    pub program ast.Program
    pub diags List[syntax.Diag]
    pub parsed int  ## How many of `diags`, from the first, are the lexer's and the parser's; the
    pub texts List[str]  ## The source of each file of the program, in the order of `program.files`.
pub fn load.load(arg str, std str) load.Loaded ! load.LoadError [clock env fail fs io proc]  ## Read the program at `arg`, a file or a package directory, with every module
pub fn load.load_derived(arg str, std str, user Map[str, str]) load.Loaded ! load.LoadError [clock env fail fs io proc]  ## `load`, with the texts the program's own derive generators wrote, by
pub fn load.load_with(arg str, std str, over Map[str, str]) load.Loaded ! load.LoadError [clock env fail fs io proc]  ## `load` with the text of some files given instead of read from disk, by
pub fn load.one_pass(loaded load.Loaded, semantic List[load.Shown], broken List[load.Shown]) List[load.Shown]  ## The checker's errors in `semantic` that survive the errors in `broken` (both
pub fn load.prove_command(rest List[str], std str) int [clock env fs io proc]
pub struct load.Shown  ## One diagnostic ready to print, from the loader's or the checker's.
    pub rule str
    pub pos ast.Pos
    pub message str
    pub help str?
    pub notes List[check.Note]
    pub warning bool
    pub fixes List[str]  ## The titles of the fixes the diagnostic offers.
    pub edits List[fix.Fix]  ## The edits of those fixes, positions as the checker made them; `report`
pub fn load.shown_syntax(d syntax.Diag) load.Shown  ## A syntax or loading diagnostic.
pub fn load.shown_check(d check.Diag) load.Shown  ## A checker diagnostic.
pub fn load.locate(files List[ast.SourceFile], at ast.Pos) (str, ast.Pos)  ## The file path and in-file position of `at`. Prelude positions stay as they
pub fn load.report(files List[ast.SourceFile], texts List[str], diags List[load.Shown], json bool) str  ## Report `diags` for the program `files` (sources `texts`, one per file):
pub fn load.report_next(files List[ast.SourceFile], texts List[str], diags List[load.Shown], json bool) str  ## `report` for `ail check` (check.out.2): each error also says what to run next.
pub load.IMPLICIT_STD := ...  ## The modules of std.use.1's index that are written in AIL: the name a file
pub fn load.is_implicit_use(u ast.Use) bool  ## Whether `u` is an import the loader added (std.use.4), not a written one.
pub fn load.may_name_std(text str) bool  ## Whether `text` may name one of the implicit modules, `NAME.`: the cheap
pub fn load.std_mentions(tokens List[syntax.Token], std str) List[int] [fs io]  ## The rows of `IMPLICIT_STD` that one file's tokens name as `NAME.MEMBER`
pub fn load.implicit_uses(rows List[int], line int) List[ast.Use]  ## One implicit import per row of `rows`, at column 0 of `line`, the first
pub fn load.keep_implicit(wanted List[ast.Use], written List[ast.Use], lines int) List[ast.Use]  ## The implicit imports of `wanted` that no written import of their file
```

trimmed to fit 2000 tokens (~1632); dropped: private items
