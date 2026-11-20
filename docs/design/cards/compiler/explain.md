# compiler: explain

## explain
```
explain.UNSUPPORTED := "bootstrap.unsupported"  ## The code for an implementation limit; not a rule ID (spec README, rule 6).
explain.UNSUPPORTED_TEXT := ...
explain.SUBSET_RULE := "native.subset.1"  ## The rule whose explanation lists the constructs outside the native subset (ADR 0746).
explain.TEXT_CHARS := 400  ## Longest rule text printed in the text form, in characters.
explain.EXAMPLE_LINES := 4  ## Longest example printed in the text form (explain.cover.4).
struct explain.Chapter
    name str
    text str
struct explain.RuleRow
    rule str
    chapter str
    file str
struct explain.Adr
    id str
    title str
struct explain.Worked
    rejected str
    fixed str?
struct explain.Explanation
    rule str
    file str
    line int
    text str
    adrs List[explain.Adr]
    example str?
    rejected str?
    fixed str?
fn explain.root() str [env]  ## The repository directory.
fn explain.read_or_die(file str) str [fs io]
fn explain.chapters() List[explain.Chapter] [env fs io]  ## Every `docs/spec/*.md` chapter, sorted by name.
fn explain.cat(a str, b str) str
fn explain.clen(s str) int
fn explain.is_rule_id(s str) bool  ## Is `s` shaped like a rule ID, `chapter.section.n`?
fn explain.rule_at(line str) str?  ## The rule ID that starts `line`, if it starts one (`**chapter.section.n** — ...`).
fn explain.all_rows(chs List[explain.Chapter]) List[explain.RuleRow]
fn explain.all_rules(chs List[explain.Chapter]) List[str]
fn explain.fixable(rule str) bool [env fs io]  ## Does `ail fix` repair diagnostics of `rule`?
fn explain.hex_digit(n int) str
fn explain.json_str(v str) str
fn explain.emit(text str) [io]  ## `text` with the last `ch` characters or fewer: `s` as `print!` would write it.
fn explain.list(json bool) [env fs io]  ## `ail explain [--json]` (explain.list.1): every rule, its chapter key and
fn explain.shorten(text str) str  ## `text` cut after the last sentence within `TEXT_CHARS`.
fn explain.strip_links(text str) str  ## `[label](target)` as `label`: a link target costs tokens and an agent can't
fn explain.adrs_in(text str) List[explain.Adr] [env fs io]  ## The ADRs cited in `text` (`ADR NNNN`), with titles from the ADR index.
fn explain.block(lines List[str], at int) str  ## The lines of a code block that opens at `at`, up to the closing fence.
fn explain.worked_example(chs List[explain.Chapter], rule str) explain.Worked?  ## The first block tagged `error=RULE` anywhere in the specification, and the
fn explain.tagged_example(chs List[explain.Chapter], rule str) str?  ## The first block of the specification whose `rules=` tag names `rule`, when
fn explain.explanation(chs List[explain.Chapter], rule str) explain.Explanation? [env fs io]
fn explain.subset_constructs() List[List[str]] [env fs io]  ## The rows of the constructs table: (construct, owner) (ADR 0746).
fn explain.to_text(e explain.Explanation) str [env fs io]
fn explain.to_json(e explain.Explanation) str [env fs io]
fn explain.near(chs List[explain.Chapter], rule str) List[str]  ## Closest known rule IDs to a mistyped one, for the "did you mean" help.
explain.SUMMARY_CHARS := 100  ## Longest one-line summary in a family listing, in characters.
explain.LIST_MAX := 40  ## A prefix with more rules than this and no dot lists its families instead.
explain.WORD_MAX := 20  ## Most rules a word search prints.
struct explain.Brief  ## One rule in a listing: where it is and its text on one line.
    rule str
    file str
    line int
    text str
fn explain.briefs(chs List[explain.Chapter]) List[explain.Brief]  ## Every rule with its text joined onto one line, in spec order.
fn explain.summary(text str) str  ## The first sentence of `text`, cut to `SUMMARY_CHARS` at a space.
fn explain.name_char(code int) bool  ## A letter, digit, `_` or `.`: a character of a name in a code span.
fn explain.span_names(text str) List[str]  ## The names that open the code spans of `text`: `read_line() str?` gives `read_line`.
explain.PLAIN_NAMES := ...  ## Code-span names that say nothing about a family: keywords and the tool's name.
fn explain.family_name(n str) str  ## `n` as a family listing shows it: `r.read` as `read`; "" for a keyword, a
fn explain.plain_has(text str, word str) bool  ## Does `word` appear in `text` as a whole word, ignoring case?
fn explain.by_word(all List[explain.Brief], within str, word str) List[explain.Brief]  ## The rules under `within` (every rule when it is empty) that name `word` in
fn explain.query_of(arg str) str  ## `arg` as a family prefix or word: lower case, no trailing dot; "" when it
fn explain.print_rules(query str, hits List[explain.Brief], limit int, json bool) [io]  ## The rules found for `query`, one line each, `limit` at most (explain.find.1).
fn explain.print_sections(query str, hits List[explain.Brief], json bool) [io]  ## A large prefix (`std`, `expr`): its families, with the rule count and the
fn explain.discover(chs List[explain.Chapter], query str, json bool) bool [io]  ## `ail explain PREFIX|WORD` (explain.find.1): the rules of a family
fn explain.run(rule str, json bool) int [env fs io]
pub fn explain.command(args List[str]) int [env fs io]  ## `ail explain [RULE | FAMILY | WORD] [--json]`: the exit status.
```
