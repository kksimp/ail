# compiler: budget

## budget
```
pub struct budget.Budget  ## What `--budget` and `--from` asked for.
    pub tokens int
    pub from int
pub struct budget.Taken  ## The result of `take`: the budget (if asked for), or the error to print.
    pub found bool
    pub budget budget.Budget
    pub why str
struct budget.Record
    text str
    rank int
    kind str
    stderr bool
struct budget.Count
    kind str
    n int
budget.KINDS := ...  ## The rank order (budget.out.3): errors and traps, failed tests, warnings,
fn budget.rank_of(kind str) int
pub fn budget.default_for(command str) int  ## The default budget of each command, in estimated tokens (budget.out.5).
fn budget.all_digits(v str) bool
pub fn budget.take(mut args List[str]) budget.Taken [mutates]  ## Remove `--budget [N]` and `--from K` from `args` (the words before a
pub fn budget.estimate_tokens(s str) int  ## A cheap token estimate (api.card.3): a run of letters or digits costs one
fn budget.lines_inc(text str) List[str]  ## The lines of `text`, each with its newline (Rust's `split_inclusive`).
fn budget.chomp(line str) str
fn budget.continues(body str) bool
fn budget.records(text str, stderr bool, mut out List[budget.Record]) [mutates]  ## Split one stream into records (budget.out.2).
fn budget.num(s str) bool
fn budget.classify(line str) str  ## The kind of a record from its first line (budget.out.2).
fn budget.has_drive(line str) bool  ## A Windows path starts with a drive (`C:/dir/f.ail:2:5: ...`), whose colon is not a field separator.
fn budget.splitn(s str, sep str, n int) List[str]  ## At most `n` pieces; the last holds the rest.
fn budget.is_summary(line str) bool  ## `N passed, M failed` from `ail test`.
pub fn budget.apply(b budget.Budget, stdout str, stderr str, json bool) List[str]  ## The budgeted stdout and stderr (budget.out.1-4), as `(stdout, stderr)`.
fn budget.plural(k str, n int) str
fn budget.note(b budget.Budget, total int, shown int, full int, tokens int, counts List[budget.Count], json bool) str  ## The closing note (budget.out.4).
pub fn budget.run(b budget.Budget, exe str, args List[str], json bool) int [io proc]  ## Run `exe ARGS` as a child with the budget applied to its output
fn budget.emit(text str, to_err bool) [io]  ## `print!` / `eprint!`: the text as it is, which `print` would end with one more newline.
fn budget.without(xs List[str], i int, n int) List[str]  ## `xs` without the `n` items from index `i`.
struct budget.Pick  ## How many records are shown, and how many of them in full (from the start).
    shown int
    full int
fn budget.one_line(text str) str  ## The one-line form of a record: its first line.
fn budget.choose(recs List[budget.Record], from int, tokens int, used int) budget.Pick  ## The records from `from` that `tokens` covers, with `used` already spent
```
