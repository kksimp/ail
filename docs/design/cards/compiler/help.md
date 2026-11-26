# compiler: help

## help
```
struct help.Cmd  ## One command: its name, its group (an entry of `GROUPS`), a one-line summary, its usage line, and the rule IDs
    name str
    group str
    summary str
    usage str
    rules List[str]
help.GROUPS := ...  ## The groups, in the order `ail help` prints them (tool.help.1).
fn help.c(name str, group str, summary str, usage str, rules str) help.Cmd  ## A table row; `rules` is a comma list.
fn help.commands() List[help.Cmd]  ## The table (tool.help.1): every command a user types, once.
pub fn help.is_command(word str) bool  ## Is `word` a command of the table (tool.help.1)?
fn help.overview(cmds List[help.Cmd]) str  ## The overview (tool.help.1): one line per command under its group's heading.
fn help.detail(k help.Cmd) str  ## One command (tool.help.2): its usage line and its rules.
fn help.q(s str) str  ## A JSON string.
fn help.one_json(k help.Cmd) str  ## One command as a JSON object (tool.help.3).
fn help.table_json(cmds List[help.Cmd]) str  ## The whole table as one JSON array (tool.help.3).
pub fn help.command(args List[str]) int [io]  ## The exit status of `ail help ARGS` (also `--help`, `-h`, and no command at all).
fn help.alias_of(word str) str  ## The command another toolchain's word means (tool.help.4), else "".
fn help.edit_distance(a str, b str, cap int) int  ## The number of single-character edits between `a` and `b`, at most `cap + 1`.
fn help.nearest(cmds List[help.Cmd], word str) str  ## The command `word` most likely means (tool.help.4), else "": another toolchain's word for it, a command it
```
