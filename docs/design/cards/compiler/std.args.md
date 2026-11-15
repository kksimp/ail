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
pub fn std.args.usage(name str, about str, specs List[std.args.Spec]) str  ## The usage text `--help` prints (std.args.5).
fn std.args.find_long(specs List[std.args.Spec], name str) std.args.Spec?
fn std.args.find_short(specs List[std.args.Spec], short str) std.args.Spec?
pub fn std.args.parse(name str, about str, specs List[std.args.Spec], words List[str]) std.args.Args ! std.args.ArgsError [fail io]  ## Reads `words` (usually `env.args()`) against `specs`. `--help` or `-h`
```
