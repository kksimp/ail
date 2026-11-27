# compiler: std.args

## std.args
```
pub error struct std.args.ArgsError  ## What was wrong with the command line, with the usage text (std.args.6).
    pub why str
    pub usage str
pub impl ToStr for std.args.ArgsError
    pub fn to_str(self) str  ## What was wrong with the command line, with the usage text (std.args.6).
pub struct std.args.Spec  ## One accepted argument: made by `flag`, `opt`, `pos` or `rest`.
    pub kind std.args.SpecKind
    pub name str
    pub short str
    pub help str
    pub default str
    pub envvar str  ## The environment variable that supplies the value when the command line
    pub fn std.args.Spec.from_env(self, key str) std.args.Spec  ## The same spec, with its value taken from environment variable `key` when the
pub enum std.args.SpecKind  ## Which builder made a `Spec` (std.args.1-2, ADR 1068).
    Flag
    Opt
    Pos
    Rest
pub struct std.args.Args  ## What the command line said (std.args.4).
    values Map[str, str]
    flags Set[str]
    given Set[str]
    text str
    pub rest List[str]  ## The words left over after the named positionals, in order.
    pub fn std.args.Args.has(self, name str) bool  ## Whether option or positional `name` was given on the command line
    pub fn std.args.Args.opt(self, name str) str?  ## Option or positional `name` as a `str`, `none` when absent (std.args.8).
    pub fn std.args.Args.int(self, name str) int ! std.args.ArgsError [fail]  ## `name` as an `int`; text that isn't one is an `ArgsError` (std.args.8).
    pub fn std.args.Args.float(self, name str) float ! std.args.ArgsError [fail]  ## `name` as a `float` (std.args.8).
    pub fn std.args.Args.opt_int(self, name str) int?  ## `name` as an `int` when given and a number, else `none` (std.args.8).
    pub fn std.args.Args.opt_float(self, name str) float?  ## `name` as a `float` when given and a number, else `none` (std.args.8).
    pub fn std.args.Args.flag(self, name str) bool  ## Whether switch `name` was given (std.args.4).
    pub fn std.args.Args.get(self, name str) str  ## The value of option or positional `name`; its default when absent
pub fn std.args.flag(name str, short str, help str) std.args.Spec  ## A switch: `--NAME` or `-SHORT`; `short` may be "" (std.args.1).
pub fn std.args.opt(name str, short str, help str, default str) std.args.Spec  ## An option that takes a value: `--NAME V`, `--NAME=V` or `-SHORT V`;
pub fn std.args.pos(name str, help str) std.args.Spec  ## A required positional word, in the order the specs list them (std.args.2).
pub fn std.args.rest(name str, help str) std.args.Spec  ## Collects every positional word after the required ones into `rest`
pub struct std.args.Command  ## One subcommand: a name, a help line and its own specs (std.args.9).
    pub name str
    pub help str
    pub specs List[std.args.Spec]
pub struct std.args.Parsed  ## The subcommand that was chosen and what its words said (std.args.9).
    pub command str
    pub opts std.args.Args
pub fn std.args.command(name str, help str, specs List[std.args.Spec]) std.args.Command  ## A subcommand for `parse_commands` (std.args.9).
pub fn std.args.usage(name str, about str, specs List[std.args.Spec]) str  ## The usage text `--help` prints (std.args.5).
fn std.args.usage_text(name str, version str, about str, specs List[std.args.Spec]) str
pub fn std.args.commands_usage(name str, version str, about str, cmds List[std.args.Command]) str  ## The usage text for a tool with subcommands: one line per command (std.args.9).
fn std.args.find_long(specs List[std.args.Spec], name str) std.args.Spec?
fn std.args.find_short(specs List[std.args.Spec], short str) std.args.Spec?
pub fn std.args.parse(name str, about str, specs List[std.args.Spec], words List[str]) std.args.Args ! std.args.ArgsError [env fail io]  ## Reads `words` (usually `env.args()`) against `specs`. `--help` or `-h`
pub fn std.args.parse_versioned(name str, version str, about str, specs List[std.args.Spec], words List[str]) std.args.Args ! std.args.ArgsError [env fail io]  ## `parse` for a tool with a version: `--version` and `-V` print
pub fn std.args.parse_in(name str, version str, about str, specs List[std.args.Spec], words List[str], environ Map[str, str]) std.args.Args ! std.args.ArgsError [fail io]  ## `parse_versioned` reading environment fallbacks from `environ` instead of the
fn std.args.truthy(v str) bool
pub fn std.args.parse_commands(name str, version str, about str, cmds List[std.args.Command], words List[str]) std.args.Parsed ! std.args.ArgsError [env fail io]  ## Reads `words` for a tool with subcommands: the first word names the command
pub fn std.args.parse_commands_in(name str, version str, about str, cmds List[std.args.Command], words List[str], environ Map[str, str]) std.args.Parsed ! std.args.ArgsError [fail io]  ## `parse_commands` reading environment fallbacks from `environ` (std.args.10).
```
