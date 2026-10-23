# compiler: fixrun

## fixrun
```
fn fixrun.unescape(s str) str
fn fixrun.escape(s str) str
fn fixrun.anchor_of(name str) str
fn fixrun.pos_of(line str, col str) ast.Pos
pub fn fixrun.settled(src str, dump str) str  ## One line per fix that settles: `LINE:COL:` of its diagnostic, then its edits.
fn fixrun.num_of(j Json, key str) int
fn fixrun.edits_of(f Json) List[fix.Edit]
pub fn fixrun.rust_diagnose(path str, src str, json bool) fixrun.Diagnosis [env fs io proc]  ## A check by the Rust tool, for `fixrun`: writes `src` to `path`, runs
pub fn fixrun.run_with_rust(args List[str]) int [env fs io proc]  ## `ail fix` with the Rust tool as the checker, for `tools/fix/parity.py`.
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
pub fn fixrun.begin(args List[str]) fixrun.Run [fs io]  ## Read the command line (after the command word) and the file.
pub fn fixrun.step(mut r fixrun.Run, d fixrun.Diagnosis) bool [mutates]  ## Apply the first fix of each diagnostic in `d`, which is the check of the
pub fn fixrun.canonicalize(mut r fixrun.Run) [mutates]  ## After the last round (fix.apply.3): a file that was canonical stays canonical.
pub fn fixrun.finish(mut r fixrun.Run, left fixrun.Diagnosis) int [fs io mutates]  ## Write the result (or show its diff) and print the summary; `left` is the
fn fixrun.emit(text str, to_err bool) [io]  ## `print!` / `eprint!`: the text as it is, which `print` would end with one more newline.
```
