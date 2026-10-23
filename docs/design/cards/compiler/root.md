# compiler: (root)
entry: main in main.ail

## (root)
```
fn main() ! [clock env fail fs io proc]
BUILD_USAGE := ...
fn positive(str) bool  ## A positive whole number (run.limit.4).
fn host_target() str [io proc]  ## The target `ail build` makes by default: the host's, from `uname`.
fn build_cli(List[str]) [clock env fs io proc]  ## `ail build FILE [-o OUT] [--target T] [--timeout MS] [--max-memory MB]
fn compiler_dir() str [env io]  ## The `compiler/` directory: `AIL_COMPILER`, or `compiler` under `AIL_ROOT`.
fn compile_to(str, str, str, str, str) [clock env fs io proc]  ## Build `file` for `target` (the host's when empty) into `out` (by default
fn cached_copy(str, str, str) bool [fs io proc]  ## Copy the cached executable `exe` to `out`; false when there is none
fn compile_fresh(str, str, str, str, str) [clock env fs io proc]  ## `compile_to` without the cache.
fn check_dump(str, bool) ! [env fail fs io]  ## The checker's diagnostics for one dump, sorted by position as `ail check`
fn runtime(str, str) !ast.Program [fail fs io]  ## The native runtime in directory `dir`, parsed as one program.
fn runtime_with(str, str, str, str) !ast.Program [fail fs io]  ## The runtime with `ail build`'s default time limit in milliseconds and
fn source_text(str) !str [fail fs io]  ## The text of a program: a file, or all the files of a package directory.
fn imported_text(str, str, str) str [fs io]  ## `text` (the program's own source) followed by the text of every module it
fn executable_hex(mir.Module, str, str) str [io]  ## The executable of lowered program `m` for `target` as hex text (native.build.3).
fn runtime_syntax(str, str) [io]  ## The lexer's and the parser's diagnostics for one runtime file; any ends the run.
fn lower_file(str, ast.Program?, str) !mir.Module [clock env fail fs io proc]  ## Parse a program and the modules it imports, and lower them to MIR
fn mentions_prelude(List[mir.Source]) bool  ## Whether a source names a module of the prelude written in AIL: the test
fn manifest_dep(str, str) str? [fs io]  ## The directory of dependency `name` in the manifest `ROOT/ail.pkg`:
fn std_file(str, str) str  ## The file of standard module `imp` (`std.csv`, `std.net.http`) under `std`.
fn derived_sources() !Derived [env fail fs io]  ## The methods derive generators wrote (ct.gen.1), which `ail build` puts
fn derive_builtin(List[mir.Source]) List[mir.Source] [io]  ## The methods of the `impl`s the built-in generators derive (ADR 0850) that
fn derive_user(List[mir.Source], ast.Program?, str) List[mir.Source] [clock env fs io proc]  ## The methods of the `impl`s of traits with a user-written `@derive(f)`
fn run_generators(List[mir.Source], List[derive.Job], ast.Program?) List[str] [clock env fs io proc]  ## The text each job's generator wrote, in order, or `@@err MESSAGE`.
fn derive_failure(str) str  ## "`Trait` can't be derived for `Type`" for the label `Type.Trait`.
fn remove_quietly(str) [fs io]
fn user_main_renamed(mir.Source) mir.Source  ## `s` with its function `main` of the root module renamed, so the generator
struct Derived  ## What `ail build` passes besides the program's files.
fn derived_source(str, str, List[str]) mir.Source [io]  ## The generated text `body` of the `impl` `label` (`TYPE.TRAIT`) in
fn uses_regex(ast.Program) bool  ## Whether the program has a `use` that ends in `regex` (`use std.regex`, `use app.regex`).
fn uses_ui(ast.Program) bool  ## Whether the program has a `use` that ends in `ui` (`use std.ui`).
fn mentions_ui(str) bool  ## Whether `text` names `ui.` as a word: not the end of a longer name
fn parse_source(str, str) !mir.Source [fail fs io]  ## One file, parsed; syntax errors are printed and the run fails.
fn parse_module(str, List[str]) !List[mir.Source] [fail fs io]  ## The files of one module, parsed together: a generic name declared in one
fn parse_text(str, str, str) mir.Source [io]  ## Text from `path`, parsed; syntax errors are printed and the run fails.
fn parse_with(str, str, str, Set[str]) mir.Source [io]
CACHE_KEEP := 64  ## Entries the cache keeps; the least recently used go first (native.build.5).
fn cache_dir() str [env]  ## The build cache directory: `AIL_CACHE_DIR`, `$XDG_CACHE_HOME/ail/build`,
fn tree_stamp(str, str) str [fs io]  ## One line per `.ail` file under `root` (path and the SHA-256 of its text),
fn cache_key(str, str, str, str) str [env fs io]  ## The key of building `file` for `target`: everything that decides the
fn cache_trim(str) [fs io proc]  ## Keep the newest `CACHE_KEEP` executables of `dir` (`ls -t` lists newest first;
fn cli_prepare(str) str [clock env fs io proc]  ## The cached executable of `file` for the host, built first when missing.
fn cli_prepare_cmd(List[str]) [clock env fs io proc]
fn cli_run(List[str]) [clock env fs io proc]  ## `run FILE [ARGS...]`: the program's output and exit status are passed on.
fn chop(str) str  ## `s` without one final newline.
fn load_dump(List[str]) [clock env fs io proc]  ## `load FILE...`: the loaded program as `ail api --ast --program` prints it.
fn check_cmd(List[str]) [clock env fs io proc]  ## `check-program [--json] FILE|DIR`: `ail check` on the native front end.
fn api_cmd(List[str]) [clock env fs io proc]  ## `api [--outline] FILE [--text | --pretty]`: the outline of one file
```

trimmed to fit 2000 tokens (~1871); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence)
