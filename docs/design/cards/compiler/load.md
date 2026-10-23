# compiler: load

## load
```
pub struct load.SelfDone  ## What the pass changed: the program with the rewritten functions, and its diagnostics.
    pub program ast.Program
    pub diags List[syntax.Diag]
pub fn load.rewrite_self(prog ast.Program) load.SelfDone
pub load.FILE_LINES := 1048576  ## Each file's lines are numbered from a multiple of this (modules.rs).
pub load.PRELUDE_LINE := 1073741824  ## Lines from here on belong to the prelude and the standard types.
pub error struct load.LoadError  ## Why a program can't be loaded: a file that can't be read, or no files.
    pub message str
pub struct load.Loaded  ## A loaded program with the diagnostics its loading found.
    pub program ast.Program
    pub diags List[syntax.Diag]
    pub texts List[str]  ## The source of each file of the program, in the order of `program.files`.
pub fn load.load(arg str, std str) load.Loaded ! load.LoadError [clock env fail fs io proc]  ## Read the program at `arg`, a file or a package directory, with every module
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
```

trimmed to fit 2000 tokens (~599); dropped: private items
