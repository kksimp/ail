# compiler: (root)
entry: main in main.ail

## (root)
```
fn main() ! [clock env fail fs io proc]
    fails with JsonError | fs.IoError
BUILD_USAGE := ...
fn positive(v str) bool  ## A positive whole number (run.limit.4).
fn host_target() str [io proc]  ## The target `ail build` makes by default: the host's, from `uname`.
fn build_cli(rest List[str]) [clock env fs io proc]  ## `ail build FILE [-o OUT] [--target T] [--timeout MS] [--max-memory MB]
fn compiler_dir() str [env io]  ## The `compiler/` directory: `AIL_COMPILER`, or `compiler` under `AIL_ROOT`.
fn compile_to(file str, out_arg str, target_arg str, ms str, mb str) [clock env fs io proc]  ## Build `file` for `target` (the host's when empty) into `out` (by default
fn check_dump(path str, many bool) ! [env fail fs io]  ## The checker's diagnostics for one dump, sorted by position as `ail check`
    fails with JsonError | fs.IoError
fn runtime(dir str, path str) !ast.Program [fail fs io]  ## The native runtime in directory `dir`, parsed as one program.
    fails with fs.IoError
fn runtime_with(dir str, path str, ms str, mb str) !ast.Program [fail fs io]  ## The runtime with `ail build`'s default time limit in milliseconds and
    fails with fs.IoError
fn source_text(path str) !str [fail fs io]  ## The text of a program: a file, or all the files of a package directory.
    fails with fs.IoError
fn executable_hex(m mir.Module, path str, target str) str [io]  ## The executable of lowered program `m` for `target` as hex text (native.build.3).
fn runtime_syntax(path str, text str) [io]  ## The lexer's and the parser's diagnostics for one runtime file; any ends the run.
fn lower_file(path str, rt ast.Program?, std str) !mir.Module [clock env fail fs io proc]  ## Parse a program and the modules it imports, and lower them to MIR
    fails with fs.IoError
fn mentions_prelude(srcs List[mir.Source]) bool  ## Whether a source names a module of the prelude written in AIL: the test
fn manifest_dep(root str, name str) str? [fs io]  ## The directory of dependency `name` in the manifest `ROOT/ail.pkg`:
fn std_file(std str, imp str) str  ## The file of standard module `imp` (`std.csv`, `std.net.http`) under `std`.
fn derived_sources() !Derived [env fail fs io]  ## The methods derive generators wrote (ct.gen.1), which `ail build` puts
    fails with fs.IoError
fn derive_builtin(srcs List[mir.Source]) List[mir.Source] [io]  ## The methods of the `impl`s the built-in generators derive (ADR 0850) that
fn derive_user(srcs List[mir.Source], rt ast.Program?, std str) List[mir.Source] [clock env fs io proc]  ## The methods of the `impl`s of traits with a user-written `@derive(f)`
fn run_generators(srcs List[mir.Source], jobs List[derive.Job], rt ast.Program?) List[str] [clock env fs io proc]  ## The text each job's generator wrote, in order, or `@@err MESSAGE`.
fn derive_failure(label str) str  ## "`Trait` can't be derived for `Type`" for the label `Type.Trait`.
fn remove_quietly(path str) [fs io]
fn user_main_renamed(s mir.Source) mir.Source  ## `s` with its function `main` of the root module renamed, so the generator
struct Derived  ## What `ail build` passes besides the program's files.
    sources List[mir.Source]
    prelude str?
fn derived_source(module str, label str, body List[str]) mir.Source [io]  ## The generated text `body` of the `impl` `label` (`TYPE.TRAIT`) in
fn uses_regex(prog ast.Program) bool  ## Whether the program has a `use` that ends in `regex` (`use std.regex`, `use app.regex`).
fn parse_source(module str, path str) !mir.Source [fail fs io]  ## One file, parsed; syntax errors are printed and the run fails.
    fails with fs.IoError
fn parse_module(module str, paths List[str]) !List[mir.Source] [fail fs io]  ## The files of one module, parsed together: a generic name declared in one
    fails with fs.IoError
fn parse_text(module str, path str, text str) mir.Source [io]  ## Text from `path`, parsed; syntax errors are printed and the run fails.
fn parse_with(module str, path str, text str, names Set[str]) mir.Source [io]
CACHE_KEEP := 64  ## Entries the cache keeps; the least recently used go first (native.build.5).
fn cache_dir() str [env]  ## The build cache directory: `AIL_CACHE_DIR`, `$XDG_CACHE_HOME/ail/build`,
fn tree_stamp(tag str, root str) str [fs io]  ## One line per `.ail` file under `root` (path and the SHA-256 of its text),
fn cache_key(file str, target str) str [env fs io]  ## The key of building `file` for `target`: everything that decides the
fn cache_trim(dir str) [fs io proc]  ## Keep the newest `CACHE_KEEP` executables of `dir` (`ls -t` lists newest first;
fn cli_prepare(file str) str [clock env fs io proc]  ## The cached executable of `file` for the host, built first when missing.
fn cli_prepare_cmd(rest List[str]) [clock env fs io proc]
fn cli_run(rest List[str]) [clock env fs io proc]  ## `run FILE [ARGS...]`: the program's output and exit status are passed on.
fn chop(s str) str  ## `s` without one final newline.
fn load_dump(files List[str]) [env fs io]  ## `load FILE...`: the loaded program as `ail api --ast --program` prints it.
fn check_cmd(rest List[str]) [env fs io]  ## `check-program [--json] FILE|DIR`: `ail check` on the native front end. The
```
