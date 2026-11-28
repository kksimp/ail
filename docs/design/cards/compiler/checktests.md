# compiler: checktests

## checktests
```
checktests.LAST_LINE := 1073741824  ## Lines from here on are the prelude's, not the program's.
checktests.MAX_LINES := 10000  ## A program with more lines of its own than this is not built for its tests by a check (check.tests.5).
checktests.KEEP_KEYS := 4096  ## How many keys one program's cache file keeps: the newest first, so undoing an edit is a hit.
checktests.OUTSIDE := ...  ## The capabilities whose answers come from outside the program (check.tests.3).
struct checktests.Block  ## A top-level declaration of one of the program's own files, with the comment and attribute lines right above it.
    text str
    fx List[str]  ## The effects of the functions declared in it.
struct checktests.Case  ## One test `ail test` would run.
    name str
    block int
    outside bool  ## It reaches a capability of `OUTSIDE`, so no pass is stored for it.
    by_reach bool  ## A doc example: `outside` is read from what its declaration reaches.
struct checktests.Reach
    blocks List[checktests.Block]
    named Map[str, List[int]]  ## The blocks that declare each name.
    line_block List[List[int]]  ## For each file of the program, the block of each line: -1 above the first declaration. Empty for a std file.
    ambient str  ## The text no block holds (the lines above each file's first declaration, the manifest and the lock file).
    fn checktests.Reach.closure(self, start int, mut edges Map[int, List[int]]) List[int] [mutates]  ## The blocks `start` reaches, itself first (check.tests.2). `edges` keeps each block's neighbours between calls.
fn checktests.own_file(f ast.SourceFile) bool
fn checktests.file_at(files List[ast.SourceFile], line int) int
fn checktests.place(files List[ast.SourceFile], sizes List[int], pos ast.Pos) (int, int)?  ## The file and the 0-based line of `pos`; none outside the program's own files (`sizes`: their line counts).
fn checktests.lead_line(l str) bool
fn checktests.last_part(name str) str  ## `name` without the module it is qualified with: a module's `sum` is `shop.sum` in the program.
fn checktests.note_name(mut named Map[str, List[int]], name str, b int) [mutates]  ## Block `b` declares `name`. The name is kept as it is written at a use: without its module.
fn checktests.block_at(files List[ast.SourceFile], sizes List[int], line_block List[List[int]], pos ast.Pos) int  ## The block that holds `pos`, or -1.
fn checktests.graph(loaded load.Loaded, effects List[List[str]], root_dir str) checktests.Reach [fs io]  ## The declarations of the program's own files and the names that reach them.
fn checktests.words_of(text str) List[str]  ## Every run of letters, digits and `_` in `text`: comments and string contents too, which only widens the reach.
fn checktests.touches_outside(fx List[str]) bool
fn checktests.has_attr(f ast.FnDecl, name str) bool
fn checktests.used_modules(text str) List[str]  ## The modules whose tests `ail test` runs with the file's own: `use NAME` of one segment, not std (test.run.1).
fn checktests.cases(loaded load.Loaded, effects List[List[str]], g checktests.Reach, ri int) List[checktests.Case]  ## The tests of the program, as `ail test` names them, the file's own first.
fn checktests.test_file(given str) str [fs io]  ## The file `ail test` tests for the operand `given`: a file is itself, a directory is its `main.ail` or its
fn checktests.plain_path(p str) str
fn checktests.root_index(loaded load.Loaded, file str) int  ## The index in the program's files of the file under test, or -1.
fn checktests.compiler_stamp() str [env fs io]  ## What tells one compiler from the next: the running executable's size and time. Empty when it can't be read,
fn checktests.file_stamp(p str) str [fs io]  ## The path, size and time of the file `p`; empty when it can't be read.
fn checktests.cache_dir() str [env]  ## The build cache's directory (native.build.5), where the stored passes live too.
fn checktests.store(dir str, file str, text str) [fs io]
fn checktests.launcher_path() str [env fs io]  ## The `ail` to run the tests with: `$AIL`, else the launcher of this toolchain. Empty when there is none.
fn checktests.first_line(text str) str
pub fn checktests.run(loaded load.Loaded, given str, forced bool, known_fx List[List[str]]) int [env fs io proc]  ## Run the tests of the checked program `given` that have no stored pass and print the `tests:` line
```
