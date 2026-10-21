# js: (root)
entry: main in main.ail
ail.pkg:
    package garlicjs

## (root)
```
fn main() ! [env fail fs io]
    fails with Absent | JsonError | fs.IoError
fn usage() [io]
fn run(mode str, source str, loc bool) str ! syntax.JsError [fail]  ## One file's output for `mode`.
fn test_dir(dir str) !bool [fail fs io]  ## Run every `NAME.js` in `dir`: its tree must equal `NAME.ast`, or its
    fails with fs.IoError
struct RunResult  ## What running a script did.
    lines List[str]  ## console.log's lines, when not echoed.
    failed bool
    message str  ## "Uncaught TypeError: ...", or the SyntaxError.
fn run_script(source str, max_steps int, echo bool) RunResult [io]  ## Parse and run one script in a new interpreter.
fn test_runs(dir str) !bool [fail fs io]  ## Run every `NAME.js` in `dir` and compare what it printed with `NAME.out`
    fails with fs.IoError
PAGE_URL := "https://example.test/dir/page.html"  ## The address test pages load at.
fn run_page(html str, max_steps int) List[str] [io]  ## Load a page and run its scripts, then a second of virtual time: what the
fn run_page_at(html str, max_steps int, url str) List[str] [io]
fn test_pages(dir str) !bool [fail fs io]  ## Run every `NAME.html` in `dir` against `NAME.out` (written by Chromium).
    fails with fs.IoError
struct Case  ## A packed test.
    name str
    source str
fn load(path str) !List[Case] [fail fs io]
    fails with Absent | JsonError | fs.IoError
fn load_known(path str) !Map[str, str] [fail fs io]  ## `known.txt`: `DIR/NAME reason`, one per line; `#` starts a comment.
    fails with fs.IoError
fn fingerprint(text str) str  ## FNV-1a over the text's code points: a short fingerprint of a tree.
fn run_suite(dir str, bless bool) !bool [fail fs io]  ## Run the suite in `dir`; print each surprise and a summary. True if none.
    fails with Absent | JsonError | fs.IoError
```
