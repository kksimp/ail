# compiler: fixrun

## fixrun
```
fixrun.MAX_ROUNDS := 5  ## Rounds of fix-and-recheck before giving up (fix.apply.2).
pub struct fixrun.Found  ## A diagnostic as `ail fix` sees it: its rule, position, whether it is a
    pub rule str
    pub pos ast.Pos
    pub warning bool
    pub fixes List[fix.Fix]
pub struct fixrun.Diagnosis  ## What a check of one file found: the diagnostics sorted by position with
    pub found List[fixrun.Found]
    pub report str
struct fixrun.Cut
    from int
    to int
    text str
struct fixrun.Chosen
    rule str
    pos ast.Pos
    title str
    cuts List[fixrun.Cut]
fn fixrun.line_starts(chars List[char]) List[int]  ## Character offset of the start of each line of `src`.
fn fixrun.offset(starts List[int], total int, p ast.Pos) int?
fn fixrun.choose(found List[fixrun.Found], src str) List[fixrun.Chosen]  ## The first fix of every diagnostic that has one, skipping a fix whose edits
fn fixrun.apply(src str, chosen List[fixrun.Chosen]) str  ## Apply the edits from last to first so earlier offsets stay valid.
fn fixrun.quote(s str) str
fixrun.FIX_USAGE := "usage: ail fix FILE [--dry-run] [--json]"  ## The usage text.
pub struct fixrun.Run  ## One `ail fix` run. The caller drives it, so this module needs neither the
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
    pub rules List[str]  ## The rule of every fix applied, in order (check.fix.1).
    pub sure_only bool  ## Apply only certain fixes of errors (fix.auto.1, ADR 1130).
pub fn fixrun.begin(args List[str]) fixrun.Run [fs io]  ## Read the command line (after the command word) and the file.
pub fn fixrun.step(mut r fixrun.Run, d fixrun.Diagnosis) bool [mutates]  ## Apply the first fix of each diagnostic in `d`, which is the check of the
pub fn fixrun.applied_text(r fixrun.Run) str  ## The `fixed PATH:LINE:COL: [RULE] TITLE` lines of the fixes applied so far, each
pub fn fixrun.canonicalize(mut r fixrun.Run) [mutates]  ## After the last round (fix.apply.3): a file that was canonical stays canonical.
pub fn fixrun.finish(mut r fixrun.Run, left fixrun.Diagnosis) int [fs io mutates]  ## Write the result (or show its diff) and print the summary; `left` is the
fn fixrun.emit(text str, to_err bool) [io]  ## `print!` / `eprint!`: the text as it is, which `print` would end with one more newline.
```
